//! Port of `ReleaseService` and `GenreService`.

use super::filter::{self, fts_query, push_filters, push_order_by};
use super::{hydrate, map_err, new_id, now, push_id_list, RELEASE_COLUMNS};
use crate::db::LOCAL_USER_ID;
use crate::domain::*;
use crate::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use std::collections::HashMap;

pub async fn list(pool: &SqlitePool, p: &ReleaseFilterParams) -> AppResult<PageResponse<Release>> {
    let size = p.size.unwrap_or(20).max(1);
    let page = p.page.unwrap_or(0);

    let mut count_q: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT COUNT(*) FROM user_releases ur JOIN releases r ON r.id = ur.release_id \
         WHERE ur.user_id = ",
    );
    count_q.push_bind(LOCAL_USER_ID);
    push_filters(&mut count_q, p);
    let total_elements: i64 = count_q
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(map_err)?;

    let mut q: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT ");
    q.push(RELEASE_COLUMNS)
        .push(" FROM user_releases ur JOIN releases r ON r.id = ur.release_id WHERE ur.user_id = ")
        .push_bind(LOCAL_USER_ID);
    push_filters(&mut q, p);
    push_order_by(&mut q, p)?;
    q.push(" LIMIT ")
        .push_bind(size as i64)
        .push(" OFFSET ")
        .push_bind((page as i64) * (size as i64));

    let rows = q.build().fetch_all(pool).await.map_err(map_err)?;
    let content = hydrate(pool, rows).await?;

    let total_pages = ((total_elements as f64) / (size as f64)).ceil() as u32;
    Ok(PageResponse {
        content,
        total_elements: total_elements as u64,
        total_pages,
        number: page,
        size,
        first: page == 0,
        last: page + 1 >= total_pages.max(1),
    })
}

pub async fn get(pool: &SqlitePool, release_id: &str) -> AppResult<Release> {
    let sql = format!(
        "SELECT {RELEASE_COLUMNS} FROM user_releases ur JOIN releases r ON r.id = ur.release_id \
         WHERE ur.user_id = ? AND r.id = ?"
    );
    let rows = sqlx::query(&sql)
        .bind(LOCAL_USER_ID)
        .bind(release_id)
        .fetch_all(pool)
        .await
        .map_err(map_err)?;

    hydrate(pool, rows)
        .await?
        .pop()
        .ok_or(AppError::NotFound("release"))
}

pub async fn random_queued(pool: &SqlitePool) -> AppResult<Release> {
    let sql = format!(
        "SELECT {RELEASE_COLUMNS} FROM user_releases ur JOIN releases r ON r.id = ur.release_id \
         WHERE ur.user_id = ? AND ur.status = 'QUEUED' ORDER BY RANDOM() LIMIT 1"
    );
    let rows = sqlx::query(&sql)
        .bind(LOCAL_USER_ID)
        .fetch_all(pool)
        .await
        .map_err(map_err)?;

    hydrate(pool, rows)
        .await?
        .pop()
        .ok_or(AppError::NotFound("queued release"))
}

pub async fn create(pool: &SqlitePool, req: ReleaseRequest) -> AppResult<Release> {
    if req.artist.trim().is_empty() || req.title.trim().is_empty() {
        return Err(AppError::Invalid("artist and title are required".into()));
    }

    let mut tx = pool.begin().await.map_err(map_err)?;
    let stamp = now();

    // Find-or-create the catalog row, exactly as ReleaseService.create does.
    let existing = find_catalog(&mut *tx, req.spotify_id.as_deref(), req.musicbrainz_id.as_deref()).await?;

    let release_id = match existing {
        Some(id) => id,
        None => {
            let id = new_id();
            // ON CONFLICT DO NOTHING replaces catching DataIntegrityViolationException and
            // retrying the lookup. Two commands can still interleave here, so the losing
            // insert affects no rows and we fall back to the row the winner created.
            let inserted = sqlx::query(
                "INSERT INTO releases (id, artist, title, release_year, album_art_url, country, \
                 spotify_id, musicbrainz_id, created_at) VALUES (?,?,?,?,?,?,?,?,?) \
                 ON CONFLICT DO NOTHING",
            )
            .bind(&id)
            .bind(req.artist.trim())
            .bind(req.title.trim())
            .bind(req.release_year)
            .bind(&req.album_art_url)
            .bind(&req.country)
            .bind(blank_to_none(&req.spotify_id))
            .bind(blank_to_none(&req.musicbrainz_id))
            .bind(&stamp)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

            if inserted.rows_affected() == 0 {
                find_catalog(&mut *tx, req.spotify_id.as_deref(), req.musicbrainz_id.as_deref())
                    .await?
                    .ok_or_else(|| AppError::Internal("catalog insert conflicted but no row found".into()))?
            } else {
                if let Some(g) = &req.genres {
                    set_genres(&mut tx, &id, g).await?;
                }
                if let Some(links) = &req.streaming_links {
                    set_links(&mut tx, &id, links).await?;
                }
                id
            }
        }
    };

    // Idempotent: tracking the same release twice returns what is already there rather
    // than failing on the unique constraint.
    let already: Option<String> =
        sqlx::query_scalar("SELECT id FROM user_releases WHERE release_id = ? AND user_id = ?")
            .bind(&release_id)
            .bind(LOCAL_USER_ID)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_err)?;

    if already.is_none() {
        sqlx::query(
            "INSERT INTO user_releases (id, release_id, user_id, status, discovery_link, \
             created_at, updated_at) VALUES (?,?,?,'QUEUED',?,?,?)",
        )
        .bind(new_id())
        .bind(&release_id)
        .bind(LOCAL_USER_ID)
        .bind(&req.discovery_link)
        .bind(&stamp)
        .bind(&stamp)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    }

    tx.commit().await.map_err(map_err)?;
    get(pool, &release_id).await
}

pub async fn update(pool: &SqlitePool, release_id: &str, req: ReleaseUpdateRequest) -> AppResult<Release> {
    let mut tx = pool.begin().await.map_err(map_err)?;

    let tracking: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT id, date_listened FROM user_releases WHERE release_id = ? AND user_id = ?",
    )
    .bind(release_id)
    .bind(LOCAL_USER_ID)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_err)?;

    let (_tracking_id, current_listened) = tracking.ok_or(AppError::NotFound("release"))?;

    // Catalog fields. As in the Java, a None means "leave alone", so a field cannot be
    // cleared through this path. Preserved deliberately rather than silently widened.
    let mut cat: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE releases SET ");
    let mut cat_sep = cat.separated(", ");
    let mut cat_any = false;
    if let Some(v) = &req.artist {
        cat_sep.push("artist = ").push_bind_unseparated(v.clone());
        cat_any = true;
    }
    if let Some(v) = &req.title {
        cat_sep.push("title = ").push_bind_unseparated(v.clone());
        cat_any = true;
    }
    if let Some(v) = req.release_year {
        cat_sep.push("release_year = ").push_bind_unseparated(v);
        cat_any = true;
    }
    if let Some(v) = &req.album_art_url {
        cat_sep.push("album_art_url = ").push_bind_unseparated(v.clone());
        cat_any = true;
    }
    if let Some(v) = &req.country {
        cat_sep.push("country = ").push_bind_unseparated(v.clone());
        cat_any = true;
    }
    if cat_any {
        cat.push(" WHERE id = ").push_bind(release_id);
        cat.build().execute(&mut *tx).await.map_err(map_err)?;
    }

    if let Some(g) = &req.genres {
        set_genres(&mut tx, release_id, g).await?;
    }
    if let Some(links) = &req.streaming_links {
        set_links(&mut tx, release_id, links).await?;
    }

    // Tracking fields.
    let mut date_listened = req.date_listened.clone();
    let mut status = req.status;
    if status == Some(ReleaseStatus::Listened)
        && current_listened.is_none()
        && date_listened.is_none()
    {
        // Marking something listened without saying when means now.
        date_listened = Some(now());
    }

    let mut tr: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE user_releases SET ");
    let mut tr_sep = tr.separated(", ");
    tr_sep.push("updated_at = ").push_bind_unseparated(now());
    if let Some(v) = status.take() {
        tr_sep
            .push("status = ")
            .push_bind_unseparated(filter::status_str(v));
    }
    if let Some(v) = &req.discovery_link {
        tr_sep.push("discovery_link = ").push_bind_unseparated(v.clone());
    }
    if let Some(v) = req.rating {
        tr_sep.push("rating = ").push_bind_unseparated(v);
    }
    if let Some(v) = req.did_not_finish {
        tr_sep.push("did_not_finish = ").push_bind_unseparated(i32::from(v));
    }
    if let Some(v) = &req.notes {
        tr_sep.push("notes = ").push_bind_unseparated(v.clone());
    }
    if let Some(v) = &date_listened {
        tr_sep.push("date_listened = ").push_bind_unseparated(v.clone());
    }
    tr.push(" WHERE release_id = ")
        .push_bind(release_id)
        .push(" AND user_id = ")
        .push_bind(LOCAL_USER_ID);
    tr.build().execute(&mut *tx).await.map_err(map_err)?;

    tx.commit().await.map_err(map_err)?;
    get(pool, release_id).await
}

/// Removes the tracking row only. The catalog row stays, which is the whole point of the
/// split: the album still exists, you just are not tracking it.
pub async fn delete(pool: &SqlitePool, release_id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM user_releases WHERE release_id = ? AND user_id = ?")
        .bind(release_id)
        .bind(LOCAL_USER_ID)
        .execute(pool)
        .await
        .map_err(map_err)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("release"));
    }
    Ok(())
}

/// Catalog autocomplete. This is where FTS5 earns its place: prefix matching ranked by
/// relevance, rather than the substring LIKE the Java used ordered by insertion date.
pub async fn search_catalog(pool: &SqlitePool, q: &str, limit: u32) -> AppResult<Vec<ResolvedMetadata>> {
    let Some(match_expr) = fts_query(q) else {
        return Ok(vec![]);
    };
    let limit = limit.clamp(1, 20) as i64;

    let rows = sqlx::query(
        "SELECT r.id, r.artist, r.title, r.release_year, r.album_art_url, r.country, \
                r.spotify_id, r.musicbrainz_id \
         FROM releases_fts f JOIN releases r ON r.rowid = f.rowid \
         WHERE releases_fts MATCH ? ORDER BY rank LIMIT ?",
    )
    .bind(&match_expr)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;

    if rows.is_empty() {
        return Ok(vec![]);
    }

    let ids: Vec<String> = rows
        .iter()
        .map(|r| r.try_get::<String, _>("id").map_err(map_err))
        .collect::<AppResult<_>>()?;

    let mut gq: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT rg.release_id, g.name FROM release_genres rg \
         JOIN genres g ON g.id = rg.genre_id WHERE rg.release_id IN (",
    );
    push_id_list(&mut gq, &ids);
    gq.push(") ORDER BY g.name");
    let mut genres: HashMap<String, Vec<String>> = HashMap::new();
    for row in gq.build().fetch_all(pool).await.map_err(map_err)? {
        genres
            .entry(row.try_get("release_id").map_err(map_err)?)
            .or_default()
            .push(row.try_get("name").map_err(map_err)?);
    }

    let mut lq: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT release_id, service, url FROM release_streaming_links WHERE release_id IN (",
    );
    push_id_list(&mut lq, &ids);
    lq.push(")");
    let mut links: HashMap<String, HashMap<String, String>> = HashMap::new();
    for row in lq.build().fetch_all(pool).await.map_err(map_err)? {
        let id: String = row.try_get("release_id").map_err(map_err)?;
        links.entry(id).or_default().insert(
            row.try_get("service").map_err(map_err)?,
            row.try_get("url").map_err(map_err)?,
        );
    }

    rows.iter()
        .map(|row| {
            let id: String = row.try_get("id").map_err(map_err)?;
            Ok(ResolvedMetadata {
                artist: row.try_get("artist").map_err(map_err)?,
                title: row.try_get("title").map_err(map_err)?,
                release_year: row.try_get("release_year").map_err(map_err)?,
                album_art_url: row.try_get("album_art_url").map_err(map_err)?,
                country: row.try_get("country").map_err(map_err)?,
                genres: genres.get(&id).cloned().unwrap_or_default(),
                streaming_links: links.get(&id).cloned().unwrap_or_default(),
                spotify_id: row.try_get("spotify_id").map_err(map_err)?,
                musicbrainz_id: row.try_get("musicbrainz_id").map_err(map_err)?,
            })
        })
        .collect()
}

pub async fn list_genres(pool: &SqlitePool) -> AppResult<Vec<String>> {
    sqlx::query_scalar("SELECT name FROM genres ORDER BY name COLLATE NOCASE")
        .fetch_all(pool)
        .await
        .map_err(map_err)
}

/// Countries that actually appear in the library.
///
/// Scoped to tracked releases rather than the whole catalog: offering a filter for a
/// country you own nothing from can only ever return an empty list. Includes queued
/// releases, unlike the stats breakdowns, because the library filter covers both.
pub async fn list_countries(pool: &SqlitePool) -> AppResult<Vec<String>> {
    sqlx::query_scalar(
        "SELECT DISTINCT r.country FROM user_releases ur \
         JOIN releases r ON r.id = ur.release_id \
         WHERE ur.user_id = ? AND r.country IS NOT NULL AND TRIM(r.country) != '' \
         ORDER BY r.country COLLATE NOCASE",
    )
    .bind(LOCAL_USER_ID)
    .fetch_all(pool)
    .await
    .map_err(map_err)
}

// ---------------------------------------------------------------- helpers

async fn find_catalog(
    conn: &mut sqlx::SqliteConnection,
    spotify_id: Option<&str>,
    musicbrainz_id: Option<&str>,
) -> AppResult<Option<String>> {
    // Spotify first, then MusicBrainz, matching findExistingCatalogRelease.
    // The column name comes from this fixed pair, never from input.
    for (column, value) in [("spotify_id", spotify_id), ("musicbrainz_id", musicbrainz_id)] {
        let Some(v) = value.map(str::trim).filter(|v| !v.is_empty()) else {
            continue;
        };
        let found: Option<String> =
            sqlx::query_scalar(&format!("SELECT id FROM releases WHERE {column} = ?"))
                .bind(v)
                .fetch_optional(&mut *conn)
                .await
                .map_err(map_err)?;
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

/// Replaces a release's genres wholesale, creating any that do not exist yet.
/// `findOrCreate` in the Java, plus the join-table rewrite the JPA cascade did implicitly.
async fn set_genres(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    release_id: &str,
    names: &[String],
) -> AppResult<()> {
    sqlx::query("DELETE FROM release_genres WHERE release_id = ?")
        .bind(release_id)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;

    for raw in names {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        // Case-insensitive find, matching findByNameIgnoreCase, so "Jazz" and "jazz" do
        // not become two genres.
        let existing: Option<i64> =
            sqlx::query_scalar("SELECT id FROM genres WHERE name = ? COLLATE NOCASE")
                .bind(name)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_err)?;

        let genre_id = match existing {
            Some(id) => id,
            None => sqlx::query_scalar("INSERT INTO genres (name) VALUES (?) RETURNING id")
                .bind(name)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_err)?,
        };

        sqlx::query("INSERT OR IGNORE INTO release_genres (release_id, genre_id) VALUES (?, ?)")
            .bind(release_id)
            .bind(genre_id)
            .execute(&mut **tx)
            .await
            .map_err(map_err)?;
    }
    Ok(())
}

async fn set_links(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    release_id: &str,
    links: &HashMap<String, String>,
) -> AppResult<()> {
    sqlx::query("DELETE FROM release_streaming_links WHERE release_id = ?")
        .bind(release_id)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;

    for (service, url) in links {
        sqlx::query(
            "INSERT OR REPLACE INTO release_streaming_links (release_id, service, url) VALUES (?,?,?)",
        )
        .bind(release_id)
        .bind(service)
        .bind(url)
        .execute(&mut **tx)
        .await
        .map_err(map_err)?;
    }
    Ok(())
}

/// An empty external id is not an id. Storing `''` would make two unrelated releases
/// collide on the UNIQUE constraint; NULL does not collide with NULL.
fn blank_to_none(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}
