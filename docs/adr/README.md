# Architecture decision records

One file per decision that would be expensive to reverse: undoing it would force a schema
migration, a file format version bump or a module rewrite, and the reason is not obvious
from the code.

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-local-first-tauri-app.md) | Local-first Tauri app with a Rust core | Accepted |
| [0002](0002-sqlite-with-catalog-and-tracking-split.md) | SQLite, keeping the catalog and tracking split | Accepted |
| [0003](0003-streaming-services-are-link-only.md) | Streaming services are link-only | Accepted |
| [0004](0004-release-group-is-album-identity.md) | The MusicBrainz release group is an album's identity | Accepted |
| [0005](0005-library-export-format.md) | Library export format | Accepted |
| [0006](0006-sync-deferred-backend-frozen.md) | Sync is deferred, and the old backend is frozen | Accepted |
| [0007](0007-artists-to-check.md) | Artists to check are a second kind of entity | Accepted |

## What does not belong here

- **Rules for whoever edits the code**, such as collation, the sort whitelist or pragma
  checks. Those live in `CLAUDE.md`.
- **Choices that are cheap to reverse**, such as the LTO profile. A code comment covers
  them.
- **Specifications.** `docs/export-format.md` is one. ADR 0005 records the decisions
  behind it and points there for the detail.

## Writing one

Copy the shape of an existing record: context, decision, alternatives considered,
consequences, status. Keep it to about a page.

A record is never edited to say something different. When a decision is overturned, write
a new ADR, set the old one's status to `Superseded by NNNN`, and leave its text alone. The
old reasoning is the part worth keeping.
