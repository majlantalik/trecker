//! Port of `StatsService`.
//!
//! The Java did three of these as native Postgres queries and two through the Criteria
//! API. All five are plain SQL here. The only real translation is date extraction:
//! Postgres `EXTRACT(YEAR FROM ts)` returns a number, SQLite `strftime('%Y', ts)` returns
//! a string, so each needs an explicit CAST.

use super::{hydrate, map_err, RELEASE_COLUMNS};
use crate::db::LOCAL_USER_ID;
use crate::domain::{ActivityDataPoint, BreakdownItem, YearEndBasis, YearEndEntry};
use crate::error::AppResult;
use sqlx::{Row, SqlitePool};

pub async fn activity(pool: &SqlitePool) -> AppResult<Vec<ActivityDataPoint>> {
    let rows = sqlx::query(
        "SELECT CAST(strftime('%Y', date_listened) AS INTEGER) AS year, \
                CAST(strftime('%m', date_listened) AS INTEGER) AS month, \
                COUNT(*) AS count \
         FROM user_releases \
         WHERE status = 'LISTENED' AND date_listened IS NOT NULL AND user_id = ? \
         GROUP BY year, month ORDER BY year, month",
    )
    .bind(LOCAL_USER_ID)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;

    rows.iter()
        .map(|r| {
            Ok(ActivityDataPoint {
                year: r.try_get("year").map_err(map_err)?,
                month: r.try_get::<i64, _>("month").map_err(map_err)? as u32,
                count: r.try_get::<i64, _>("count").map_err(map_err)? as u64,
            })
        })
        .collect()
}

pub async fn by_genre(pool: &SqlitePool) -> AppResult<Vec<BreakdownItem>> {
    breakdown(
        pool,
        "SELECT g.name AS label, COUNT(*) AS count \
         FROM user_releases ur \
         JOIN releases r        ON r.id = ur.release_id \
         JOIN release_genres rg ON rg.release_id = r.id \
         JOIN genres g          ON g.id = rg.genre_id \
         WHERE ur.status = 'LISTENED' AND ur.user_id = ? \
         GROUP BY g.name ORDER BY count DESC, label ASC",
    )
    .await
}

pub async fn by_country(pool: &SqlitePool) -> AppResult<Vec<BreakdownItem>> {
    breakdown(
        pool,
        "SELECT r.country AS label, COUNT(*) AS count \
         FROM user_releases ur \
         JOIN releases r ON r.id = ur.release_id \
         WHERE ur.status = 'LISTENED' AND r.country IS NOT NULL AND ur.user_id = ? \
         GROUP BY r.country ORDER BY count DESC, label ASC",
    )
    .await
}

async fn breakdown(pool: &SqlitePool, sql: &str) -> AppResult<Vec<BreakdownItem>> {
    let rows = sqlx::query(sql)
        .bind(LOCAL_USER_ID)
        .fetch_all(pool)
        .await
        .map_err(map_err)?;

    rows.iter()
        .map(|r| {
            Ok(BreakdownItem {
                label: r.try_get("label").map_err(map_err)?,
                count: r.try_get::<i64, _>("count").map_err(map_err)? as u64,
            })
        })
        .collect()
}

/// How many albums a year-end list ranks.
pub const YEAR_END_LIMIT: i64 = 20;

pub async fn year_end(
    pool: &SqlitePool,
    year: i32,
    basis: YearEndBasis,
) -> AppResult<Vec<YearEndEntry>> {
    let base = format!(
        "SELECT {RELEASE_COLUMNS} FROM user_releases ur JOIN releases r ON r.id = ur.release_id \
         WHERE ur.user_id = ? AND ur.status = 'LISTENED' AND ur.rating IS NOT NULL "
    );
    let order = "ORDER BY ur.rating DESC, ur.date_listened DESC, ur.id LIMIT ?";

    let rows = match basis {
        YearEndBasis::Listened => {
            // Half-open interval on the stored RFC3339 strings. They are UTC with a fixed
            // width, so a lexical comparison is a chronological one, and this stays
            // index-friendly in a way that wrapping the column in strftime() would not.
            let start = format!("{year:04}-01-01T00:00:00Z");
            let end = format!("{:04}-01-01T00:00:00Z", year + 1);
            let sql = format!("{base}AND ur.date_listened >= ? AND ur.date_listened < ? {order}");
            sqlx::query(&sql)
                .bind(LOCAL_USER_ID)
                .bind(start)
                .bind(end)
                .bind(YEAR_END_LIMIT)
                .fetch_all(pool)
                .await
        }
        YearEndBasis::Released => {
            let sql = format!("{base}AND r.release_year = ? {order}");
            sqlx::query(&sql)
                .bind(LOCAL_USER_ID)
                .bind(year)
                .bind(YEAR_END_LIMIT)
                .fetch_all(pool)
                .await
        }
    }
    .map_err(map_err)?;

    Ok(hydrate(pool, rows)
        .await?
        .into_iter()
        .enumerate()
        .map(|(i, release)| YearEndEntry {
            rank: i as u32 + 1,
            release,
        })
        .collect())
}
