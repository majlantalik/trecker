//! Repository layer. Replaces `ReleaseService`, `GenreService` and `StatsService`.

pub mod filter;
pub mod releases;
pub mod stats;

#[cfg(test)]
mod integration;

use crate::domain::{Release, ReleaseStatus};
use crate::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Row, Sqlite, SqliteExecutor};
use std::collections::HashMap;

pub fn map_err(e: sqlx::Error) -> AppError {
    AppError::Internal(e.to_string())
}

/// The columns every release query selects, in the order `hydrate` expects.
///
/// `id` is the catalog release id, not the tracking row id, because that is what the
/// frontend holds and sends back to get, update and delete. `created_at` comes from the
/// tracking row, not the catalog: it is when *you* added it, not when the album entered
/// the shared catalog. Both of these match `ReleaseResponse.from`.
pub const RELEASE_COLUMNS: &str = "
    r.id              AS id,
    r.artist          AS artist,
    r.title           AS title,
    r.release_year    AS release_year,
    r.album_art_url   AS album_art_url,
    r.country         AS country,
    ur.status         AS status,
    ur.discovery_link AS discovery_link,
    ur.rating         AS rating,
    ur.did_not_finish AS did_not_finish,
    ur.date_listened  AS date_listened,
    ur.notes          AS notes,
    ur.created_at     AS created_at
";

/// A release without its two child collections attached yet.
struct Partial {
    release: Release,
}

fn read_row(row: &sqlx::sqlite::SqliteRow) -> AppResult<Partial> {
    let status: String = row.try_get("status").map_err(map_err)?;
    let status = match status.as_str() {
        "LISTENED" => ReleaseStatus::Listened,
        // The column has a CHECK constraint, so anything else means the file was edited
        // by hand. Treat it as queued rather than failing the whole page.
        _ => ReleaseStatus::Queued,
    };

    Ok(Partial {
        release: Release {
            id: row.try_get("id").map_err(map_err)?,
            artist: row.try_get("artist").map_err(map_err)?,
            title: row.try_get("title").map_err(map_err)?,
            release_year: row.try_get("release_year").map_err(map_err)?,
            album_art_url: row.try_get("album_art_url").map_err(map_err)?,
            status,
            discovery_link: row.try_get("discovery_link").map_err(map_err)?,
            streaming_links: HashMap::new(),
            country: row.try_get("country").map_err(map_err)?,
            rating: row.try_get("rating").map_err(map_err)?,
            did_not_finish: row.try_get::<i64, _>("did_not_finish").map_err(map_err)? != 0,
            date_listened: row.try_get("date_listened").map_err(map_err)?,
            notes: row.try_get("notes").map_err(map_err)?,
            created_at: row.try_get("created_at").map_err(map_err)?,
            genres: Vec::new(),
        },
    })
}

/// Attaches genres and streaming links to a page of releases.
///
/// Two extra queries for the whole page rather than two per release. The JPA version used
/// eager fetching to the same end; here it has to be explicit.
pub async fn hydrate<'e, E>(executor: E, rows: Vec<sqlx::sqlite::SqliteRow>) -> AppResult<Vec<Release>>
where
    E: SqliteExecutor<'e> + Copy,
{
    let mut releases: Vec<Release> = rows
        .iter()
        .map(|r| read_row(r).map(|p| p.release))
        .collect::<AppResult<_>>()?;

    if releases.is_empty() {
        return Ok(releases);
    }

    let ids: Vec<String> = releases.iter().map(|r| r.id.clone()).collect();

    let mut genre_q: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT rg.release_id, g.name FROM release_genres rg \
         JOIN genres g ON g.id = rg.genre_id WHERE rg.release_id IN (",
    );
    push_id_list(&mut genre_q, &ids);
    genre_q.push(") ORDER BY g.name");

    let mut genres: HashMap<String, Vec<String>> = HashMap::new();
    for row in genre_q.build().fetch_all(executor).await.map_err(map_err)? {
        let id: String = row.try_get("release_id").map_err(map_err)?;
        let name: String = row.try_get("name").map_err(map_err)?;
        genres.entry(id).or_default().push(name);
    }

    let mut link_q: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT release_id, service, url FROM release_streaming_links WHERE release_id IN (",
    );
    push_id_list(&mut link_q, &ids);
    link_q.push(")");

    let mut links: HashMap<String, HashMap<String, String>> = HashMap::new();
    for row in link_q.build().fetch_all(executor).await.map_err(map_err)? {
        let id: String = row.try_get("release_id").map_err(map_err)?;
        let service: String = row.try_get("service").map_err(map_err)?;
        let url: String = row.try_get("url").map_err(map_err)?;
        links.entry(id).or_default().insert(service, url);
    }

    for r in &mut releases {
        r.genres = genres.remove(&r.id).unwrap_or_default();
        r.streaming_links = links.remove(&r.id).unwrap_or_default();
    }
    Ok(releases)
}

pub fn push_id_list(qb: &mut QueryBuilder<'_, Sqlite>, ids: &[String]) {
    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(id.clone());
    }
}

pub fn now() -> String {
    // RFC3339 in UTC, seconds precision, so stored timestamps sort correctly as strings.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    iso_from_unix(secs)
}

pub fn new_id() -> String {
    // A v4-shaped identifier without pulling in the uuid crate. Randomness comes from the
    // OS via std's hasher seed, which is enough for row ids that never leave this machine.
    use std::hash::{BuildHasher, Hasher, RandomState};
    let mut bytes = [0u8; 16];
    for chunk in bytes.chunks_mut(8) {
        let mut h = RandomState::new().build_hasher();
        h.write_u64(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0));
        chunk.copy_from_slice(&h.finish().to_le_bytes()[..chunk.len()]);
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant
    let h: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}

pub fn now_year(secs: u64) -> i32 {
    civil_from_days((secs / 86_400) as i64).0 as i32
}

fn iso_from_unix(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

// Howard Hinnant's days-from-civil, inverted.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_is_uuid_v4_shaped_and_unique() {
        let a = new_id();
        assert_eq!(a.len(), 36);
        assert_eq!(a.as_bytes()[14], b'4', "version nibble");
        assert!(matches!(a.as_bytes()[19], b'8' | b'9' | b'a' | b'b'), "variant nibble");
        let ids: std::collections::HashSet<String> = (0..1000).map(|_| new_id()).collect();
        assert_eq!(ids.len(), 1000, "ids must not collide");
    }

    #[test]
    fn timestamps_are_rfc3339_utc() {
        assert_eq!(iso_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_from_unix(1_757_625_600), "2025-09-11T21:20:00Z");
        assert_eq!(iso_from_unix(1_789_161_600), "2026-09-11T21:20:00Z");
        // Everything downstream sorts these as strings, so string order has to match
        // chronological order, including across a year boundary.
        assert!(iso_from_unix(1_000) < iso_from_unix(2_000));
        assert!(iso_from_unix(1_757_625_600) < iso_from_unix(1_789_161_600));
    }
}
