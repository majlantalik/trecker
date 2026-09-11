#!/usr/bin/env python3
"""One-time import from the Postgres web app into the local SQLite database.

Reads through `docker compose exec db psql`, so it needs the old stack running but no
backend, no login and no network. Standard library only.

    docker compose up -d db
    python3 scripts/import-from-postgres.py --dry-run
    python3 scripts/import-from-postgres.py

The two schemas are deliberately near-identical, because the SQLite one was written from
the Postgres one. What actually differs, and therefore what this script converts:

    UUID        -> TEXT, unchanged in appearance
    TIMESTAMPTZ -> TEXT, normalised to RFC3339 in UTC
    BOOLEAN     -> INTEGER, psql emits 't'/'f'
    NULL        -> psql CSV emits an empty field, which is not the same as ''

`user_releases.user_id` is rewritten to the single local user. If the old database held
more than one account, pass --user to choose which one to import; importing several
people's tracking rows into one local library would silently merge them.

NOT YET RUN AGAINST REAL DATA. It was written against the schema rather than against a
populated database, so check the --dry-run counts before trusting it.
"""

from __future__ import annotations

import argparse
import csv
import io
import os
import re
import sqlite3
import subprocess
import sys
from pathlib import Path

LOCAL_USER_ID = "00000000-0000-0000-0000-000000000001"

DEFAULT_SQLITE = Path.home() / ".local/share/cz.mtulek.trecker/trecker.db"


def load_env(path: Path) -> dict[str, str]:
    env: dict[str, str] = {}
    if not path.exists():
        return env
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        env[k.strip()] = v.strip().strip('"').strip("'")
    return env


def psql(sql: str, user: str, database: str) -> str:
    """Run one statement in the compose db service and return stdout."""
    cmd = [
        "docker", "compose", "exec", "-T", "db",
        "psql", "-U", user, "-d", database,
        "--no-align", "--tuples-only", "--field-separator=,",
        "-c", sql,
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.exit(f"psql failed:\n{proc.stderr.strip()}")
    return proc.stdout


def copy_csv(table_sql: str, user: str, database: str) -> list[dict[str, str | None]]:
    out = psql(f"COPY ({table_sql}) TO STDOUT WITH (FORMAT csv, HEADER true)", user, database)
    rows: list[dict[str, str | None]] = []
    for row in csv.DictReader(io.StringIO(out)):
        # csv gives '' for both NULL and the empty string; for these columns only NULL
        # is meaningful, and no column in this schema stores a deliberate empty string.
        rows.append({k: (v if v != "" else None) for k, v in row.items()})
    return rows


def to_rfc3339(value: str | None) -> str | None:
    """Postgres prints '2026-09-11 20:15:00+02'. SQLite stores '2026-09-11T18:15:00Z'."""
    if not value:
        return None
    v = value.strip().replace(" ", "T", 1)
    m = re.search(r"([+-])(\d{2})(?::?(\d{2}))?$", v)
    if m:
        # Shift to UTC so every stored timestamp is directly comparable as a string.
        from datetime import datetime, timedelta, timezone

        base = v[: m.start()]
        base = base.split(".")[0]
        sign = 1 if m.group(1) == "+" else -1
        offset = timedelta(hours=int(m.group(2)), minutes=int(m.group(3) or 0)) * sign
        dt = datetime.fromisoformat(base).replace(tzinfo=timezone.utc) - offset
        return dt.strftime("%Y-%m-%dT%H:%M:%SZ")
    if v.endswith("Z"):
        return v
    return v.split(".")[0] + "Z"


def to_bool(value: str | None) -> int:
    return 1 if value in ("t", "true", "TRUE", "1") else 0


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--sqlite", type=Path, default=DEFAULT_SQLITE)
    ap.add_argument("--env", type=Path, default=Path(".env"))
    ap.add_argument("--user", help="email of the account to import; required if several exist")
    ap.add_argument("--dry-run", action="store_true", help="report counts, write nothing")
    ap.add_argument("--force", action="store_true", help="import even if the target is not empty")
    args = ap.parse_args()

    env = load_env(args.env)
    pg_user = env.get("POSTGRES_USER") or os.environ.get("POSTGRES_USER")
    pg_db = env.get("POSTGRES_DB") or os.environ.get("POSTGRES_DB")
    if not pg_user or not pg_db:
        sys.exit(f"POSTGRES_USER / POSTGRES_DB not found in {args.env} or the environment")

    if not args.sqlite.exists():
        sys.exit(f"{args.sqlite} does not exist. Launch Trecker once so it creates the schema.")

    # Pick the account whose tracking rows become the local library.
    accounts = copy_csv("SELECT id, email FROM users ORDER BY created_at", pg_user, pg_db)
    if not accounts:
        sys.exit("no users in the source database; nothing to import")
    if args.user:
        match = [a for a in accounts if a["email"] == args.user]
        if not match:
            sys.exit(f"no account {args.user!r}. Found: {', '.join(a['email'] or '?' for a in accounts)}")
        account = match[0]
    elif len(accounts) == 1:
        account = accounts[0]
    else:
        sys.exit(
            "several accounts exist; choose one with --user:\n  "
            + "\n  ".join(a["email"] or "?" for a in accounts)
        )
    print(f"source account: {account['email']}")

    releases = copy_csv(
        "SELECT id, artist, title, release_year, album_art_url, country, "
        "spotify_id, musicbrainz_id, created_at FROM releases",
        pg_user, pg_db,
    )
    genres = copy_csv("SELECT id, name FROM genres", pg_user, pg_db)
    release_genres = copy_csv(
        "SELECT release_id, genre_id FROM release_genres", pg_user, pg_db
    )
    links = copy_csv(
        "SELECT release_id, service, url FROM release_streaming_links", pg_user, pg_db
    )
    tracking = copy_csv(
        "SELECT id, release_id, status, rating, did_not_finish, date_listened, notes, "
        f"discovery_link, created_at FROM user_releases WHERE user_id = '{account['id']}'",
        pg_user, pg_db,
    )

    print(
        f"  releases {len(releases)}  genres {len(genres)}  "
        f"release_genres {len(release_genres)}  streaming_links {len(links)}  "
        f"tracking {len(tracking)}"
    )

    if args.dry_run:
        print("dry run, nothing written")
        return

    con = sqlite3.connect(args.sqlite, timeout=15)
    con.execute("PRAGMA foreign_keys = ON")

    existing = con.execute("SELECT COUNT(*) FROM releases").fetchone()[0]
    if existing and not args.force:
        sys.exit(f"{args.sqlite} already holds {existing} releases; pass --force to import anyway")

    with con:
        con.executemany(
            "INSERT OR IGNORE INTO releases "
            "(id, artist, title, release_year, album_art_url, country, spotify_id, "
            " musicbrainz_id, created_at) VALUES (?,?,?,?,?,?,?,?,?)",
            [
                (
                    r["id"], r["artist"], r["title"],
                    int(r["release_year"]) if r["release_year"] else None,
                    r["album_art_url"], r["country"], r["spotify_id"], r["musicbrainz_id"],
                    to_rfc3339(r["created_at"]),
                )
                for r in releases
            ],
        )
        con.executemany(
            "INSERT OR IGNORE INTO genres (id, name) VALUES (?,?)",
            [(int(g["id"]), g["name"]) for g in genres],
        )
        con.executemany(
            "INSERT OR IGNORE INTO release_genres (release_id, genre_id) VALUES (?,?)",
            [(g["release_id"], int(g["genre_id"])) for g in release_genres],
        )
        con.executemany(
            "INSERT OR IGNORE INTO release_streaming_links (release_id, service, url) VALUES (?,?,?)",
            [(l["release_id"], l["service"], l["url"]) for l in links],
        )
        con.executemany(
            "INSERT OR IGNORE INTO user_releases "
            "(id, release_id, user_id, status, rating, did_not_finish, date_listened, "
            " notes, discovery_link, created_at, updated_at) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            [
                (
                    t["id"], t["release_id"], LOCAL_USER_ID,
                    t["status"] or "QUEUED",
                    float(t["rating"]) if t["rating"] else None,
                    to_bool(t["did_not_finish"]),
                    to_rfc3339(t["date_listened"]),
                    t["notes"], t["discovery_link"],
                    to_rfc3339(t["created_at"]),
                    # No history to draw on, so the import time is the honest answer.
                    to_rfc3339(t["created_at"]),
                )
                for t in tracking
            ],
        )

    final_releases = con.execute("SELECT COUNT(*) FROM releases").fetchone()[0]
    final_tracking = con.execute("SELECT COUNT(*) FROM user_releases").fetchone()[0]
    indexed = con.execute("SELECT COUNT(*) FROM releases_fts").fetchone()[0]
    print(f"imported: {final_releases} releases, {final_tracking} tracked, {indexed} indexed for search")
    if indexed != final_releases:
        print("WARNING: the search index is out of step with the catalog", file=sys.stderr)
    con.close()


if __name__ == "__main__":
    main()
