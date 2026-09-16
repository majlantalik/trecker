//! Port of `ReleaseService` and `GenreService`.

use super::filter::{self, fts_query, push_filters, push_order_by};
use super::{hydrate, map_err, new_id, now, push_id_list, RELEASE_COLUMNS};
use crate::db::LOCAL_USER_ID;
use crate::domain::{
    PageResponse, Release, ReleaseFilterParams, ReleaseRequest, ReleaseStatus, ReleaseUpdateRequest,
    ResolvedMetadata, UnlinkedRelease,
};
use crate::error::{AppError, AppResult};
use crate::library::{ExportedRelease, ImportMode};
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
    let existing = find_catalog(&mut *tx, req.musicbrainz_release_group_id.as_deref()).await?;

    let release_id = match existing {
        Some(id) => id,
        None => {
            let id = new_id();
            // ON CONFLICT DO NOTHING replaces catching DataIntegrityViolationException and
            // retrying the lookup. Two commands can still interleave here, so the losing
            // insert affects no rows and we fall back to the row the winner created.
            let inserted = sqlx::query(
                "INSERT INTO releases (id, artist, title, release_year, album_art_url, country, \
                 musicbrainz_release_group_id, created_at) VALUES (?,?,?,?,?,?,?,?) \
                 ON CONFLICT DO NOTHING",
            )
            .bind(&id)
            .bind(req.artist.trim())
            .bind(req.title.trim())
            .bind(req.release_year)
            .bind(&req.album_art_url)
            .bind(&req.country)
            .bind(blank_to_none(&req.musicbrainz_release_group_id))
            .bind(&stamp)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

            if inserted.rows_affected() == 0 {
                find_catalog(&mut *tx, req.musicbrainz_release_group_id.as_deref())
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
                r.musicbrainz_release_group_id \
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
                musicbrainz_release_group_id: row.try_get("musicbrainz_release_group_id").map_err(map_err)?,
            })
        })
        .collect()
}

/// The MusicBrainz release group behind a tracked release, for refreshing it by id.
/// None for a release typed in by hand; an error only if the release is not tracked.
pub async fn release_group_id(pool: &SqlitePool, release_id: &str) -> AppResult<Option<String>> {
    let row: Option<Option<String>> = sqlx::query_scalar(
        "SELECT r.musicbrainz_release_group_id FROM user_releases ur \
         JOIN releases r ON r.id = ur.release_id WHERE ur.user_id = ? AND r.id = ?",
    )
    .bind(LOCAL_USER_ID)
    .bind(release_id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    row.ok_or(AppError::NotFound("release"))
}

/// Sets the MusicBrainz release group a tracked release is, once the person has chosen it.
///
/// The id is UNIQUE in the catalog. When another release you track already has it, that is
/// the same album twice, and this refuses rather than guessing whose rating and notes win.
/// When the row holding it is one you no longer track, a leftover from a delete, the id
/// moves here: that row stays, but nothing about your library depends on it.
pub async fn link_release_group(pool: &SqlitePool, release_id: &str, group_id: &str) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(map_err)?;

    // Tracked, or not found. The same check `release_group_id` makes.
    let tracked: Option<String> = sqlx::query_scalar(
        "SELECT id FROM user_releases WHERE release_id = ? AND user_id = ?",
    )
    .bind(release_id)
    .bind(LOCAL_USER_ID)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_err)?;
    tracked.ok_or(AppError::NotFound("release"))?;

    if let Some(holder) = find_catalog(&mut tx, Some(group_id)).await? {
        if holder != release_id {
            let other: Option<(String, String)> = sqlx::query_as(
                "SELECT r.artist, r.title FROM releases r JOIN user_releases ur ON ur.release_id = r.id \
                 WHERE r.id = ? AND ur.user_id = ?",
            )
            .bind(&holder)
            .bind(LOCAL_USER_ID)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_err)?;

            if let Some((artist, title)) = other {
                return Err(AppError::AlreadyInLibrary { release_id: holder, artist, title });
            }
            sqlx::query("UPDATE releases SET musicbrainz_release_group_id = NULL WHERE id = ?")
                .bind(&holder)
                .execute(&mut *tx)
                .await
                .map_err(map_err)?;
        }
    }

    sqlx::query("UPDATE releases SET musicbrainz_release_group_id = ? WHERE id = ?")
        .bind(group_id)
        .bind(release_id)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;

    tx.commit().await.map_err(map_err)
}

/// Tracked releases with no MusicBrainz id, in the order they were added.
pub async fn unlinked(pool: &SqlitePool) -> AppResult<Vec<UnlinkedRelease>> {
    let rows: Vec<(String, String, String, Option<i32>)> = sqlx::query_as(
        "SELECT r.id, r.artist, r.title, r.release_year FROM user_releases ur \
         JOIN releases r ON r.id = ur.release_id \
         WHERE ur.user_id = ? AND r.musicbrainz_release_group_id IS NULL \
         ORDER BY ur.created_at, ur.id",
    )
    .bind(LOCAL_USER_ID)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;

    Ok(rows
        .into_iter()
        .map(|(id, artist, title, release_year)| UnlinkedRelease { id, artist, title, release_year })
        .collect())
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

/// Finds the catalog row for an album by its MusicBrainz release group, if it has one.
///
/// Keyed by group rather than by release so that two different pressings of one album,
/// which a search can easily return on two separate adds, land on one catalog row.
async fn find_catalog(
    conn: &mut sqlx::SqliteConnection,
    musicbrainz_release_group_id: Option<&str>,
) -> AppResult<Option<String>> {
    let Some(id) = musicbrainz_release_group_id.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    sqlx::query_scalar("SELECT id FROM releases WHERE musicbrainz_release_group_id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_err)
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

// ---------------------------------------------------------------- export and import

/// A tracked release plus the catalog identifier an export carries.
///
/// `Release` deliberately does not expose the release group id to the frontend, but the file
/// format needs it: it is the only key that survives a move to another machine.
pub struct ExportRow {
    pub release: Release,
    pub musicbrainz_release_group_id: Option<String>,
}

/// Every tracked release, unpaginated, in the order a person would want to read.
///
/// Catalog rows you no longer track are not here. Keeping them after a delete is an
/// implementation detail of the two-table split, not part of your library.
pub async fn export_all(pool: &SqlitePool) -> AppResult<Vec<ExportRow>> {
    let sql = format!(
        "SELECT {RELEASE_COLUMNS}, r.musicbrainz_release_group_id AS musicbrainz_release_group_id \
         FROM user_releases ur JOIN releases r ON r.id = ur.release_id \
         WHERE ur.user_id = ? \
         ORDER BY r.artist COLLATE NOCASE, r.title COLLATE NOCASE, ur.id"
    );
    let rows = sqlx::query(&sql)
        .bind(LOCAL_USER_ID)
        .fetch_all(pool)
        .await
        .map_err(map_err)?;

    // Read before `hydrate` consumes the rows. It preserves order, so zipping is safe.
    let ids: Vec<Option<String>> = rows
        .iter()
        .map(|r| r.try_get("musicbrainz_release_group_id").map_err(map_err))
        .collect::<AppResult<_>>()?;

    Ok(hydrate(pool, rows)
        .await?
        .into_iter()
        .zip(ids)
        .map(|(release, musicbrainz_release_group_id)| ExportRow {
            release,
            musicbrainz_release_group_id,
        })
        .collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportOutcome {
    Added,
    Overwritten,
    Skipped,
}

/// Writes one release from a file, in one transaction.
///
/// Identity follows `docs/export-format.md`: the MusicBrainz id first, then artist and
/// title compared case-insensitively. The second rule is a heuristic and can be wrong,
/// which is why it lives here, where the import report makes the result visible, and not
/// inside `create`.
pub async fn import_one(
    pool: &SqlitePool,
    r: &ExportedRelease,
    mode: ImportMode,
) -> AppResult<ImportOutcome> {
    let mut tx = pool.begin().await.map_err(map_err)?;
    let stamp = now();

    let catalog_id = match find_catalog(&mut tx, r.musicbrainz_id.as_deref()).await? {
        Some(id) => Some(id),
        None => find_by_name(&mut tx, &r.artist, &r.title).await?,
    };

    let tracked: Option<String> = match &catalog_id {
        Some(id) => sqlx::query_scalar(
            "SELECT id FROM user_releases WHERE release_id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(LOCAL_USER_ID)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_err)?,
        None => None,
    };

    if tracked.is_some() && mode == ImportMode::Skip {
        // Dropping the transaction rolls it back, so a skip writes nothing at all.
        return Ok(ImportOutcome::Skipped);
    }

    let status = filter::status_str(crate::library::status_of(r));
    let did_not_finish = i32::from(r.did_not_finish);

    let release_id = match catalog_id {
        Some(id) => {
            // Only an overwrite rewrites the catalog. When this is a new tracking row on a
            // catalog entry that already exists, the entry stays as it is: it is shared,
            // content-addressed data, and the same reasoning applies as in `create`, which
            // also leaves a matched catalog row alone.
            if mode == ImportMode::Overwrite {
                sqlx::query(
                    "UPDATE releases SET artist = ?, title = ?, release_year = ?, \
                     album_art_url = ?, country = ?, musicbrainz_release_group_id = ? WHERE id = ?",
                )
                .bind(&r.artist)
                .bind(&r.title)
                .bind(r.release_year)
                .bind(&r.album_art_url)
                .bind(&r.country)
                .bind(&r.musicbrainz_id)
                .bind(&id)
                .execute(&mut *tx)
                .await
                .map_err(map_err)?;

                set_genres(&mut tx, &id, &r.genres).await?;
                set_links(&mut tx, &id, &to_map(&r.streaming_links)).await?;
            }
            id
        }
        None => {
            let id = new_id();
            sqlx::query(
                "INSERT INTO releases (id, artist, title, release_year, album_art_url, \
                 country, musicbrainz_release_group_id, created_at) VALUES (?,?,?,?,?,?,?,?)",
            )
            .bind(&id)
            .bind(&r.artist)
            .bind(&r.title)
            .bind(r.release_year)
            .bind(&r.album_art_url)
            .bind(&r.country)
            .bind(&r.musicbrainz_id)
            .bind(&stamp)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;

            set_genres(&mut tx, &id, &r.genres).await?;
            set_links(&mut tx, &id, &to_map(&r.streaming_links)).await?;
            id
        }
    };

    let outcome = if tracked.is_some() {
        // An overwrite replaces every field, a null included. That is the whole difference
        // from skip: half-replacing would be the merge mode the format refuses to offer.
        sqlx::query(
            "UPDATE user_releases SET status = ?, rating = ?, did_not_finish = ?, \
             date_listened = ?, notes = ?, discovery_link = ?, \
             created_at = COALESCE(?, created_at), updated_at = ? \
             WHERE release_id = ? AND user_id = ?",
        )
        .bind(status)
        .bind(r.rating)
        .bind(did_not_finish)
        .bind(&r.date_listened)
        .bind(&r.notes)
        .bind(&r.discovery_link)
        .bind(&r.added_at)
        .bind(&stamp)
        .bind(&release_id)
        .bind(LOCAL_USER_ID)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        ImportOutcome::Overwritten
    } else {
        sqlx::query(
            "INSERT INTO user_releases (id, release_id, user_id, status, rating, \
             did_not_finish, date_listened, notes, discovery_link, created_at, updated_at) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(new_id())
        .bind(&release_id)
        .bind(LOCAL_USER_ID)
        .bind(status)
        .bind(r.rating)
        .bind(did_not_finish)
        .bind(&r.date_listened)
        .bind(&r.notes)
        .bind(&r.discovery_link)
        // A file without an addedAt was written by hand. Today is the honest answer.
        .bind(r.added_at.clone().unwrap_or_else(|| stamp.clone()))
        .bind(&stamp)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        ImportOutcome::Added
    };

    tx.commit().await.map_err(map_err)?;
    Ok(outcome)
}

/// The fallback identity rule: same artist, same title, ignoring case.
///
/// `COLLATE NOCASE` folds ASCII only, so "BJÖRK" and "Björk" are two releases to this
/// query. That is the same limitation the genre lookup and every text sort already carry,
/// and matching more loosely here would merge records rather than order them.
async fn find_by_name(
    conn: &mut sqlx::SqliteConnection,
    artist: &str,
    title: &str,
) -> AppResult<Option<String>> {
    sqlx::query_scalar(
        "SELECT id FROM releases WHERE artist = ? COLLATE NOCASE AND title = ? COLLATE NOCASE \
         ORDER BY created_at LIMIT 1",
    )
    .bind(artist.trim())
    .bind(title.trim())
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_err)
}

fn to_map(links: &std::collections::BTreeMap<String, String>) -> HashMap<String, String> {
    links.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}
