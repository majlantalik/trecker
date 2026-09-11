//! Port of `UserReleaseSpecification`.
//!
//! The Java built a JPA Criteria predicate tree; this builds the equivalent WHERE clause.
//! Two things change on the way across, both deliberate and both noted below.

use crate::domain::{ReleaseFilterParams, ReleaseStatus};
use crate::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Sqlite};

/// Sort fields the client may ask for, mapped to the SQL they mean.
///
/// This is the one piece of the port that is load-bearing for safety. `sort` arrives as a
/// client-supplied string. JPA resolved it against entity metadata and rejected anything
/// unknown; string-interpolating it into SQL would not. Anything not on this list is an
/// error, never a fallback, because silently sorting by something else hides the bug.
///
/// `COLLATE NOCASE` on the text columns is a change from Postgres, which sorted by its
/// database collation. SQLite's default BINARY collation orders every capital before
/// every lowercase letter, which in a library list reads as broken.
fn sort_column(field: &str) -> Option<&'static str> {
    Some(match field {
        "createdAt" => "ur.created_at",
        "artist" => "r.artist COLLATE NOCASE",
        "title" => "r.title COLLATE NOCASE",
        "releaseYear" => "r.release_year",
        "rating" => "ur.rating",
        "dateListened" => "ur.date_listened",
        _ => return None,
    })
}

pub fn status_str(s: ReleaseStatus) -> &'static str {
    match s {
        ReleaseStatus::Queued => "QUEUED",
        ReleaseStatus::Listened => "LISTENED",
    }
}

/// Appends every filter in `params` to a query that already has a `WHERE` and at least
/// one condition, so each clause here starts with `AND`.
pub fn push_filters(qb: &mut QueryBuilder<'_, Sqlite>, p: &ReleaseFilterParams) {
    if let Some(s) = p.status {
        qb.push(" AND ur.status = ").push_bind(status_str(s));
    }
    if let Some(min) = p.rating_min {
        qb.push(" AND ur.rating >= ").push_bind(min);
    }
    if let Some(max) = p.rating_max {
        qb.push(" AND ur.rating <= ").push_bind(max);
    }
    if let Some(dnf) = p.did_not_finish {
        qb.push(" AND ur.did_not_finish = ").push_bind(i32::from(dnf));
    }
    if let Some(c) = non_blank(&p.country) {
        qb.push(" AND LOWER(r.country) = ").push_bind(c.to_lowercase());
    }
    if let Some(y) = p.year {
        qb.push(" AND r.release_year = ").push_bind(y);
    }

    // The Java joined through to genres and then needed query.distinct(true) to undo the
    // row multiplication that join caused. EXISTS asks the same question without ever
    // producing the duplicates, so the DISTINCT is not needed here.
    if let Some(g) = non_blank(&p.genre) {
        qb.push(
            " AND EXISTS (SELECT 1 FROM release_genres rg \
             JOIN genres g ON g.id = rg.genre_id \
             WHERE rg.release_id = r.id AND LOWER(g.name) = ",
        )
        .push_bind(g.to_lowercase())
        .push(")");
    }

    // Substring LIKE, matching the Java exactly, rather than the FTS5 index. FTS matches
    // whole tokens and prefixes, so it would stop finding "phere" inside "Stratosphere".
    // For a filter box that narrows as you type, substring is the behaviour people expect.
    // The FTS index is used where prefix semantics are right: catalog autocomplete.
    if let Some(s) = non_blank(&p.search) {
        let pattern = format!("%{}%", s.to_lowercase());
        qb.push(" AND (LOWER(r.artist) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR LOWER(r.title) LIKE ")
            .push_bind(pattern)
            .push(")");
    }
}

pub fn push_order_by(qb: &mut QueryBuilder<'_, Sqlite>, p: &ReleaseFilterParams) -> AppResult<()> {
    let field = p.sort.as_deref().unwrap_or("createdAt");
    let column = sort_column(field).ok_or_else(|| AppError::Invalid(format!("cannot sort by '{field}'")))?;

    let descending = !p
        .direction
        .as_deref()
        .is_some_and(|d| d.eq_ignore_ascii_case("ASC"));

    qb.push(" ORDER BY ").push(column);
    qb.push(if descending { " DESC" } else { " ASC" });
    // Without a tiebreaker, rows equal on the sort column can swap places between pages
    // and the same release shows up twice while another never appears.
    qb.push(", ur.id ASC");
    Ok(())
}

fn non_blank(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Turns arbitrary user text into an FTS5 MATCH expression.
///
/// FTS5 has its own query language: bare input containing `*`, `"`, `:`, `-`, `NEAD`,
/// `AND`, `OR` or `NOT` is either a syntax error or silently means something else. Every
/// token is therefore quoted, which makes it a literal, and given a trailing `*` so that
/// typing "god" still finds "Godspeed".
pub fn fts_query(input: &str) -> Option<String> {
    let tokens: Vec<String> = input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
        .collect();
    (!tokens.is_empty()).then(|| tokens.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_sort_fields() {
        assert!(sort_column("createdAt").is_some());
        assert!(sort_column("rating").is_some());
        // The point of the whitelist: none of these may reach the SQL.
        assert!(sort_column("id; DROP TABLE releases").is_none());
        assert!(sort_column("(SELECT 1)").is_none());
        assert!(sort_column("notes").is_none());
        assert!(sort_column("").is_none());
    }

    #[test]
    fn fts_query_quotes_and_prefixes_each_token() {
        assert_eq!(fts_query("godspeed"), Some(r#""godspeed"*"#.into()));
        assert_eq!(
            fts_query("Godspeed You!"),
            Some(r#""Godspeed"* "You"*"#.into())
        );
    }

    #[test]
    fn fts_query_neutralises_operators() {
        // These are all FTS5 syntax in raw form; none may survive as syntax.
        assert_eq!(fts_query("NOT"), Some(r#""NOT"*"#.into()));
        assert_eq!(fts_query("a OR b"), Some(r#""a"* "OR"* "b"*"#.into()));
        assert_eq!(fts_query(r#"say "hi""#), Some(r#""say"* "hi"*"#.into()));
        assert_eq!(fts_query("foo:bar"), Some(r#""foo"* "bar"*"#.into()));
    }

    #[test]
    fn fts_query_is_none_when_nothing_searchable_remains() {
        assert_eq!(fts_query(""), None);
        assert_eq!(fts_query("   "), None);
        assert_eq!(fts_query("\"*:-"), None);
    }
}
