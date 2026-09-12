//! Database setup: file location, connection pragmas, migrations.
//!
//! Phase 2 stands the database up and proves it. Phase 3 moves the reads and writes off
//! `store.rs` and onto it.

use crate::error::{AppError, AppResult};
use serde::Serialize;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The single local user. Every `user_releases` row carries it so that the table keeps the
/// shape sync needs; it is rewritten to a real account id if and when sync ever lands.
/// Written by the schema default today; read by the repositories in Phase 3.
#[allow(dead_code)]
pub const LOCAL_USER_ID: &str = "00000000-0000-0000-0000-000000000001";

pub struct Db {
    pub pool: SqlitePool,
    pub path: PathBuf,
}

/// What the Settings view shows, and what proves at a glance that the database is real.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbInfo {
    pub path: String,
    pub size_bytes: i64,
    pub schema_version: i64,
    pub fts5: bool,
    pub journal_mode: String,
    pub foreign_keys: bool,
    pub release_count: i64,
    pub tracked_count: i64,
}

pub async fn connect(dir: &Path) -> AppResult<Db> {
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::Internal(format!("cannot create {}: {e}", dir.display())))?;

    let path = dir.join("trecker.db");

    let opts = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        // WAL lets a reader run while a writer commits. On a desktop app the writer is
        // usually the UI thread reacting to a click, so this is about not blocking paint.
        .journal_mode(SqliteJournalMode::Wal)
        // NORMAL is the right pairing with WAL: a crash can lose the last commit but
        // cannot corrupt the file. FULL would fsync on every write for no real gain here.
        .synchronous(SqliteSynchronous::Normal)
        // Off by default in SQLite, unlike Postgres. Every ON DELETE CASCADE in the
        // schema is inert without this.
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .map_err(|e| AppError::Internal(format!("cannot open database: {e}")))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("migration failed: {e}")))?;

    Ok(Db { pool, path })
}

impl Db {
    pub async fn info(&self) -> AppResult<DbInfo> {
        let internal = |e: sqlx::Error| AppError::Internal(e.to_string());

        let schema_version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations WHERE success = 1",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;

        let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;

        // The index exists only if FTS5 was compiled in, so its presence is the check.
        let fts5: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'releases_fts'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;

        let release_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM releases")
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;

        let tracked_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_releases")
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;

        let size_bytes = std::fs::metadata(&self.path).map(|m| m.len() as i64).unwrap_or(0);

        Ok(DbInfo {
            path: self.path.display().to_string(),
            size_bytes,
            schema_version,
            fts5: fts5 > 0,
            journal_mode,
            foreign_keys: foreign_keys == 1,
            release_count,
            tracked_count,
        })
    }
}
