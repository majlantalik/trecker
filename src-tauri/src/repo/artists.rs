//! Artists to check. The catalog is `artists`, your list is `user_artists`, as releases.

use super::{map_err, new_id, now, push_id_list};
use crate::db::LOCAL_USER_ID;
use crate::domain::*;
use crate::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use std::collections::HashMap;

const ARTIST_COLUMNS: &str = "
    a.id                    AS id,
    a.musicbrainz_artist_id AS musicbrainz_artist_id,
    a.name                  AS name,
    a.disambiguation        AS disambiguation,
    a.artist_type           AS artist_type,
    a.country               AS country,
    a.begin_year            AS begin_year,
    a.end_year              AS end_year,
    a.image_url             AS image_url,
    ua.status               AS status,
    ua.verdict              AS verdict,
    ua.note                 AS note,
    ua.checked_at           AS checked_at,
    ua.created_at           AS created_at
";

const FROM: &str = " FROM user_artists ua JOIN artists a ON a.id = ua.artist_id WHERE ua.user_id = ";

fn read_row(row: &sqlx::sqlite::SqliteRow) -> AppResult<Artist> {
    let status: String = row.try_get("status").map_err(map_err)?;
    let verdict: Option<String> = row.try_get("verdict").map_err(map_err)?;
    Ok(Artist {
        id: row.try_get("id").map_err(map_err)?,
        musicbrainz_artist_id: row.try_get("musicbrainz_artist_id").map_err(map_err)?,
        name: row.try_get("name").map_err(map_err)?,
        disambiguation: row.try_get("disambiguation").map_err(map_err)?,
        artist_type: row.try_get("artist_type").map_err(map_err)?,
        country: row.try_get("country").map_err(map_err)?,
        begin_year: row.try_get("begin_year").map_err(map_err)?,
        end_year: row.try_get("end_year").map_err(map_err)?,
        image_url: row.try_get("image_url").map_err(map_err)?,
        genres: Vec::new(),
        links: Vec::new(),
        // CHECK constraints hold both columns to these values.
        status: if status == "CHECKED" { ArtistStatus::Checked } else { ArtistStatus::ToCheck },
        verdict: match verdict.as_deref() {
            Some("LIKED") => Some(ArtistVerdict::Liked),
            Some("NOT_FOR_ME") => Some(ArtistVerdict::NotForMe),
            _ => None,
        },
        note: row.try_get("note").map_err(map_err)?,
        checked_at: row.try_get("checked_at").map_err(map_err)?,
        created_at: row.try_get("created_at").map_err(map_err)?,
    })
}

/// Reads rows and attaches genres and links, two queries for the whole list.
async fn hydrate(pool: &SqlitePool, rows: Vec<sqlx::sqlite::SqliteRow>) -> AppResult<Vec<Artist>> {
    let mut artists: Vec<Artist> = rows.iter().map(read_row).collect::<AppResult<_>>()?;
    if artists.is_empty() {
        return Ok(artists);
    }
    let ids: Vec<String> = artists.iter().map(|a| a.id.clone()).collect();

    let mut q: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT artist_id, name FROM artist_genres WHERE artist_id IN (");
    push_id_list(&mut q, &ids);
    q.push(") ORDER BY position");
    let mut genres: HashMap<String, Vec<String>> = HashMap::new();
    for row in q.build().fetch_all(pool).await.map_err(map_err)? {
        let id: String = row.try_get("artist_id").map_err(map_err)?;
        genres.entry(id).or_default().push(row.try_get("name").map_err(map_err)?);
    }

    // rowid keeps the order the links were written in, which is the order they are shown.
    let mut q: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT artist_id, kind, url FROM artist_links WHERE artist_id IN (");
    push_id_list(&mut q, &ids);
    q.push(") ORDER BY rowid");
    let mut links: HashMap<String, Vec<ArtistLink>> = HashMap::new();
    for row in q.build().fetch_all(pool).await.map_err(map_err)? {
        let id: String = row.try_get("artist_id").map_err(map_err)?;
        links.entry(id).or_default().push(ArtistLink {
            kind: row.try_get("kind").map_err(map_err)?,
            url: row.try_get("url").map_err(map_err)?,
        });
    }

    for a in &mut artists {
        a.genres = genres.remove(&a.id).unwrap_or_default();
        a.links = links.remove(&a.id).unwrap_or_default();
    }
    Ok(artists)
}

/// Your list: artists still to check first, newest added first, then checked artists,
/// most recently checked first.
pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Artist>> {
    let sql = format!(
        "SELECT {ARTIST_COLUMNS}{FROM}? \
         ORDER BY CASE ua.status WHEN 'TO_CHECK' THEN 0 ELSE 1 END, \
         COALESCE(ua.checked_at, ua.created_at) DESC, ua.id"
    );
    let rows = sqlx::query(&sql).bind(LOCAL_USER_ID).fetch_all(pool).await.map_err(map_err)?;
    hydrate(pool, rows).await
}

pub async fn get(pool: &SqlitePool, artist_id: &str) -> AppResult<Artist> {
    let sql = format!("SELECT {ARTIST_COLUMNS}{FROM}? AND a.id = ?");
    let rows = sqlx::query(&sql)
        .bind(LOCAL_USER_ID)
        .bind(artist_id)
        .fetch_all(pool)
        .await
        .map_err(map_err)?;
    hydrate(pool, rows).await?.pop().ok_or(AppError::NotFound("artist"))
}

/// The artist on your list with this MusicBrainz id, if there is one. Adding someone twice
/// returns them without asking MusicBrainz again.
pub async fn find_tracked(pool: &SqlitePool, musicbrainz_artist_id: &str) -> AppResult<Option<Artist>> {
    let sql = format!("SELECT {ARTIST_COLUMNS}{FROM}? AND a.musicbrainz_artist_id = ?");
    let rows = sqlx::query(&sql)
        .bind(LOCAL_USER_ID)
        .bind(musicbrainz_artist_id.trim())
        .fetch_all(pool)
        .await
        .map_err(map_err)?;
    Ok(hydrate(pool, rows).await?.pop())
}

/// Stores a looked-up artist and puts them on your list.
///
/// The catalog row is written fresh from the lookup, replacing an older one for the same
/// artist. Your tracking row is created only if it does not exist, so adding someone already
/// on your list keeps your note and whether you have checked them.
pub async fn add(pool: &SqlitePool, meta: ArtistMetadata, note: Option<String>) -> AppResult<Artist> {
    if meta.name.trim().is_empty() || meta.musicbrainz_artist_id.trim().is_empty() {
        return Err(AppError::Invalid("an artist needs a name and a MusicBrainz id".into()));
    }
    let stamp = now();
    let mut tx = pool.begin().await.map_err(map_err)?;

    let existing: Option<String> = sqlx::query_scalar("SELECT id FROM artists WHERE musicbrainz_artist_id = ?")
        .bind(meta.musicbrainz_artist_id.trim())
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_err)?;

    let artist_id = match existing {
        Some(id) => {
            sqlx::query(
                "UPDATE artists SET name = ?, disambiguation = ?, artist_type = ?, country = ?, \
                 begin_year = ?, end_year = ?, image_url = COALESCE(?, image_url), updated_at = ? \
                 WHERE id = ?",
            )
            .bind(meta.name.trim())
            .bind(&meta.disambiguation)
            .bind(&meta.artist_type)
            .bind(&meta.country)
            .bind(meta.begin_year)
            .bind(meta.end_year)
            .bind(&meta.image_url)
            .bind(&stamp)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
            id
        }
        None => {
            let id = new_id();
            sqlx::query(
                "INSERT INTO artists (id, musicbrainz_artist_id, name, disambiguation, artist_type, \
                 country, begin_year, end_year, image_url, created_at, updated_at) \
                 VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            )
            .bind(&id)
            .bind(meta.musicbrainz_artist_id.trim())
            .bind(meta.name.trim())
            .bind(&meta.disambiguation)
            .bind(&meta.artist_type)
            .bind(&meta.country)
            .bind(meta.begin_year)
            .bind(meta.end_year)
            .bind(&meta.image_url)
            .bind(&stamp)
            .bind(&stamp)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
            id
        }
    };

    sqlx::query("DELETE FROM artist_genres WHERE artist_id = ?")
        .bind(&artist_id)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    let mut seen = std::collections::HashSet::new();
    for (position, genre) in meta.genres.iter().map(|g| g.trim()).filter(|g| !g.is_empty()).enumerate() {
        if !seen.insert(genre.to_lowercase()) {
            continue;
        }
        sqlx::query("INSERT OR IGNORE INTO artist_genres (artist_id, name, position) VALUES (?,?,?)")
            .bind(&artist_id)
            .bind(genre)
            .bind(position as i64)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    }

    sqlx::query("DELETE FROM artist_links WHERE artist_id = ?")
        .bind(&artist_id)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    for link in &meta.links {
        sqlx::query("INSERT OR IGNORE INTO artist_links (artist_id, kind, url) VALUES (?,?,?)")
            .bind(&artist_id)
            .bind(&link.kind)
            .bind(&link.url)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    }

    sqlx::query(
        "INSERT INTO user_artists (id, artist_id, user_id, note, created_at, updated_at) \
         VALUES (?,?,?,?,?,?) ON CONFLICT (artist_id, user_id) DO NOTHING",
    )
    .bind(new_id())
    .bind(&artist_id)
    .bind(LOCAL_USER_ID)
    .bind(note.as_deref().map(str::trim).filter(|n| !n.is_empty()))
    .bind(&stamp)
    .bind(&stamp)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;

    tx.commit().await.map_err(map_err)?;
    get(pool, &artist_id).await
}

pub async fn update(pool: &SqlitePool, artist_id: &str, req: ArtistUpdateRequest) -> AppResult<Artist> {
    let current = get(pool, artist_id).await?;
    let stamp = now();

    let note = match req.note {
        Some(n) => Some(n.trim().to_string()).filter(|n| !n.is_empty()),
        None => current.note.clone(),
    };

    let (status, verdict, checked_at) = match req.status {
        None => (current.status, current.verdict, current.checked_at.clone()),
        Some(ArtistStatus::ToCheck) => (ArtistStatus::ToCheck, None, None),
        // Changing the verdict of someone already checked keeps the date they were checked.
        Some(ArtistStatus::Checked) => (
            ArtistStatus::Checked,
            req.verdict,
            current.checked_at.clone().filter(|_| current.status == ArtistStatus::Checked).or(Some(stamp.clone())),
        ),
    };

    sqlx::query(
        "UPDATE user_artists SET note = ?, status = ?, verdict = ?, checked_at = ?, updated_at = ? \
         WHERE artist_id = ? AND user_id = ?",
    )
    .bind(note)
    .bind(status.as_str())
    .bind(verdict.map(ArtistVerdict::as_str))
    .bind(checked_at)
    .bind(&stamp)
    .bind(artist_id)
    .bind(LOCAL_USER_ID)
    .execute(pool)
    .await
    .map_err(map_err)?;

    get(pool, artist_id).await
}

/// Takes an artist off your list. The catalog row stays, as a release's does.
pub async fn delete(pool: &SqlitePool, artist_id: &str) -> AppResult<()> {
    let done = sqlx::query("DELETE FROM user_artists WHERE artist_id = ? AND user_id = ?")
        .bind(artist_id)
        .bind(LOCAL_USER_ID)
        .execute(pool)
        .await
        .map_err(map_err)?;
    if done.rows_affected() == 0 {
        return Err(AppError::NotFound("artist"));
    }
    Ok(())
}

/// Which of these albums you already track, by release group id.
pub async fn library_matches(
    pool: &SqlitePool,
    release_group_ids: &[String],
) -> AppResult<HashMap<String, LibraryMatch>> {
    let mut matches = HashMap::new();
    // SQLite allows 32,766 bound variables; a discography is capped at 300 albums.
    for chunk in release_group_ids.chunks(500) {
        let mut q: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT r.musicbrainz_release_group_id AS group_id, r.id AS release_id, ur.status, ur.rating \
             FROM releases r JOIN user_releases ur ON ur.release_id = r.id \
             WHERE ur.user_id = ",
        );
        q.push_bind(LOCAL_USER_ID).push(" AND r.musicbrainz_release_group_id IN (");
        push_id_list(&mut q, chunk);
        q.push(")");
        for row in q.build().fetch_all(pool).await.map_err(map_err)? {
            let status: String = row.try_get("status").map_err(map_err)?;
            matches.insert(
                row.try_get("group_id").map_err(map_err)?,
                LibraryMatch {
                    release_id: row.try_get("release_id").map_err(map_err)?,
                    status: if status == "LISTENED" { ReleaseStatus::Listened } else { ReleaseStatus::Queued },
                    rating: row.try_get("rating").map_err(map_err)?,
                },
            );
        }
    }
    Ok(matches)
}
