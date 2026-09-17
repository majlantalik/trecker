//! Integration tests for the repository layer, against a real SQLite file.
//!
//! These run the actual migration and the actual SQL. They exist because the Phase 3 port
//! moved a lot of behaviour out of JPA, where the framework guaranteed it, into SQL where
//! nothing does: dedup on external ids, genre find-or-create, cascade on delete, the
//! filter predicates, pagination arithmetic and the date-range maths in the stats.

use super::{artists, releases, stats};
use crate::db;
use crate::domain::*;
use sqlx::SqlitePool;
use std::collections::HashMap;

async fn fresh() -> (tempfile::TempDir, SqlitePool) {
    let dir = tempfile::tempdir().unwrap();
    let handle = db::connect(dir.path()).await.unwrap();
    (dir, handle.pool)
}

fn req(artist: &str, title: &str) -> ReleaseRequest {
    ReleaseRequest {
        artist: artist.into(),
        title: title.into(),
        ..Default::default()
    }
}

/// Adds a release and immediately marks it listened on a given date with a given rating.
async fn add_listened(
    pool: &SqlitePool,
    artist: &str,
    title: &str,
    rating: f64,
    date: &str,
    country: Option<&str>,
    genres: &[&str],
) -> String {
    let mut r = req(artist, title);
    r.country = country.map(String::from);
    r.genres = Some(genres.iter().map(ToString::to_string).collect());
    let created = releases::create(pool, r).await.unwrap();

    releases::update(
        pool,
        &created.id,
        ReleaseUpdateRequest {
            status: Some(ReleaseStatus::Listened),
            rating: Some(rating),
            date_listened: Some(date.into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    created.id
}

#[tokio::test]
async fn starts_empty() {
    let (_d, pool) = fresh().await;
    let page = releases::list(&pool, &ReleaseFilterParams::default()).await.unwrap();
    assert_eq!(page.total_elements, 0);
    assert!(page.content.is_empty());
    assert!(page.first && page.last, "an empty page is both first and last");
}

#[tokio::test]
async fn create_then_read_back() {
    let (_d, pool) = fresh().await;
    let mut r = req("Duster", "Stratosphere");
    r.release_year = Some(1998);
    r.country = Some("US".into());
    r.genres = Some(vec!["slowcore".into(), "space rock".into()]);
    r.streaming_links = Some(HashMap::from([("spotify".into(), "https://x".into())]));

    let created = releases::create(&pool, r).await.unwrap();
    assert_eq!(created.artist, "Duster");
    assert_eq!(created.status, ReleaseStatus::Queued);
    assert_eq!(created.genres, vec!["slowcore", "space rock"], "sorted by name");
    assert_eq!(created.streaming_links.get("spotify").unwrap(), "https://x");

    let fetched = releases::get(&pool, &created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.genres, created.genres);
}

#[tokio::test]
async fn create_rejects_blank_artist_or_title() {
    let (_d, pool) = fresh().await;
    assert!(releases::create(&pool, req("  ", "Title")).await.is_err());
    assert!(releases::create(&pool, req("Artist", "")).await.is_err());
}

#[tokio::test]
async fn dedupes_catalog_on_external_id() {
    let (_d, pool) = fresh().await;

    let mut first = req("Slint", "Spiderland");
    first.musicbrainz_release_group_id = Some("mbid-1".into());
    let a = releases::create(&pool, first).await.unwrap();

    // Same MusicBrainz id, different spelling of the title: it is the same album, so the
    // catalog row must be reused rather than duplicated.
    let mut second = req("Slint", "Spiderland (Remaster)");
    second.musicbrainz_release_group_id = Some("mbid-1".into());
    let b = releases::create(&pool, second).await.unwrap();

    assert_eq!(a.id, b.id, "same catalog row");
    assert_eq!(b.title, "Spiderland", "the existing catalog row wins");

    let catalog: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM releases")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(catalog, 1);

    let tracking: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_releases")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tracking, 1, "tracking the same release twice is idempotent");
}

#[tokio::test]
async fn releases_without_an_external_id_are_never_merged() {
    // Two records with no MusicBrainz id are two releases, however similar they look.
    // Guessing that identical text means an identical album belongs in import, where the
    // user can see the result, not silently inside create().
    let (_d, pool) = fresh().await;
    let a = releases::create(&pool, req("Various", "Untitled")).await.unwrap();
    let b = releases::create(&pool, req("Various", "Untitled")).await.unwrap();
    assert_ne!(a.id, b.id);
}

#[tokio::test]
async fn blank_external_ids_do_not_collide() {
    // Empty strings would collide on the UNIQUE constraint; NULL does not collide.
    let (_d, pool) = fresh().await;
    let mut a = req("A", "One");
    a.musicbrainz_release_group_id = Some("   ".into());
    let mut b = req("B", "Two");
    b.musicbrainz_release_group_id = Some("".into());

    let ra = releases::create(&pool, a).await.unwrap();
    let rb = releases::create(&pool, b).await.unwrap();
    assert_ne!(ra.id, rb.id, "two unrelated releases, two catalog rows");
}

#[tokio::test]
async fn marking_listened_stamps_the_date_once() {
    let (_d, pool) = fresh().await;
    let created = releases::create(&pool, req("Tirzah", "Devotion")).await.unwrap();
    assert!(created.date_listened.is_none());

    let listened = releases::update(
        &pool,
        &created.id,
        ReleaseUpdateRequest {
            status: Some(ReleaseStatus::Listened),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let stamped = listened.date_listened.clone().expect("date filled in");

    // Setting the status again must not move a date that is already there.
    let again = releases::update(
        &pool,
        &created.id,
        ReleaseUpdateRequest {
            status: Some(ReleaseStatus::Listened),
            rating: Some(4.0),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(again.date_listened, Some(stamped));
    assert_eq!(again.rating, Some(4.0));
}

#[tokio::test]
async fn update_replaces_genres_wholesale() {
    let (_d, pool) = fresh().await;
    let mut r = req("Boards of Canada", "Geogaddi");
    r.genres = Some(vec!["electronic".into(), "ambient".into()]);
    let created = releases::create(&pool, r).await.unwrap();
    assert_eq!(created.genres, vec!["ambient", "electronic"]);

    let updated = releases::update(
        &pool,
        &created.id,
        ReleaseUpdateRequest {
            genres: Some(vec!["idm".into()]),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(updated.genres, vec!["idm"]);
}

#[tokio::test]
async fn genres_are_case_insensitively_deduped() {
    let (_d, pool) = fresh().await;
    let mut a = req("A", "One");
    a.genres = Some(vec!["Jazz".into()]);
    releases::create(&pool, a).await.unwrap();

    let mut b = req("B", "Two");
    b.genres = Some(vec!["jazz".into()]);
    releases::create(&pool, b).await.unwrap();

    let names = releases::list_genres(&pool).await.unwrap();
    assert_eq!(names, vec!["Jazz"], "one genre row, first spelling wins");
}

#[tokio::test]
async fn delete_removes_tracking_but_keeps_the_catalog() {
    // This is the whole point of the two-table split.
    let (_d, pool) = fresh().await;
    let created = releases::create(&pool, req("Codeine", "The White Birch")).await.unwrap();

    releases::delete(&pool, &created.id).await.unwrap();

    let catalog: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM releases")
        .fetch_one(&pool).await.unwrap();
    let tracking: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_releases")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(catalog, 1, "catalog row survives");
    assert_eq!(tracking, 0, "tracking row is gone");

    assert!(releases::get(&pool, &created.id).await.is_err());
    assert!(releases::delete(&pool, &created.id).await.is_err(), "second delete is not found");
}

#[tokio::test]
async fn filters_narrow_the_result_set() {
    let (_d, pool) = fresh().await;
    add_listened(&pool, "Alice Coltrane", "Journey in Satchidananda", 5.0, "2026-03-01T10:00:00Z", Some("US"), &["jazz"]).await;
    add_listened(&pool, "Ryo Fukui", "Scenery", 4.0, "2026-04-01T10:00:00Z", Some("JP"), &["jazz"]).await;
    releases::create(&pool, req("Slint", "Spiderland")).await.unwrap();

    let by = |p: ReleaseFilterParams| {
        let pool = pool.clone();
        async move { releases::list(&pool, &p).await.unwrap().total_elements }
    };

    assert_eq!(by(ReleaseFilterParams { status: Some(ReleaseStatus::Queued), ..Default::default() }).await, 1);
    assert_eq!(by(ReleaseFilterParams { status: Some(ReleaseStatus::Listened), ..Default::default() }).await, 2);
    assert_eq!(by(ReleaseFilterParams { country: Some("jp".into()), ..Default::default() }).await, 1, "country is case-insensitive");
    assert_eq!(by(ReleaseFilterParams { genre: Some("JAZZ".into()), ..Default::default() }).await, 2, "genre is case-insensitive");
    assert_eq!(by(ReleaseFilterParams { rating_min: Some(4.5), ..Default::default() }).await, 1);
    assert_eq!(by(ReleaseFilterParams { search: Some("scen".into()), ..Default::default() }).await, 1, "substring, not prefix");
    assert_eq!(by(ReleaseFilterParams { search: Some("ANANDA".into()), ..Default::default() }).await, 1, "substring mid-word, case-insensitive");
    assert_eq!(by(ReleaseFilterParams { year: Some(1991), ..Default::default() }).await, 0);
}

#[tokio::test]
async fn filters_by_musicbrainz_link() {
    let (_d, pool) = fresh().await;
    let mut linked = req("Slint", "Spiderland");
    linked.musicbrainz_release_group_id = Some("0b2ea9a3-3ab6-3b0b-a2f5-e7c3d5d9d2b5".into());
    releases::create(&pool, linked).await.unwrap();
    releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();

    let list = |unlinked| {
        let pool = pool.clone();
        async move {
            releases::list(&pool, &ReleaseFilterParams { unlinked, ..Default::default() })
                .await
                .unwrap()
                .content
                .into_iter()
                .map(|r| r.title)
                .collect::<Vec<_>>()
        }
    };

    assert_eq!(list(Some(true)).await, vec!["Stratosphere"]);
    assert_eq!(list(Some(false)).await, vec!["Spiderland"]);
    assert_eq!(list(None).await.len(), 2);
}

#[tokio::test]
async fn filters_by_cover() {
    let (_d, pool) = fresh().await;
    let mut with = req("Slint", "Spiderland");
    with.album_art_url = Some("https://coverartarchive.org/release-group/x/front-250".into());
    releases::create(&pool, with).await.unwrap();
    releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();
    let mut blank = req("Codeine", "Frigid Stars");
    blank.album_art_url = Some("  ".into());
    releases::create(&pool, blank).await.unwrap();

    let list = |without_cover| {
        let pool = pool.clone();
        async move {
            let mut titles = releases::list(&pool, &ReleaseFilterParams { without_cover, ..Default::default() })
                .await
                .unwrap()
                .content
                .into_iter()
                .map(|r| r.title)
                .collect::<Vec<_>>();
            titles.sort();
            titles
        }
    };

    assert_eq!(list(Some(true)).await, vec!["Frigid Stars", "Stratosphere"], "a blank URL is no cover");
    assert_eq!(list(Some(false)).await, vec!["Spiderland"]);
    assert_eq!(list(None).await.len(), 3);
}

#[tokio::test]
async fn genre_filter_does_not_duplicate_rows() {
    // The Java needed query.distinct(true) here because it joined; EXISTS cannot duplicate.
    let (_d, pool) = fresh().await;
    let mut r = req("Nala Sinephro", "Space 1.8");
    r.genres = Some(vec!["jazz".into(), "ambient".into(), "electronic".into()]);
    releases::create(&pool, r).await.unwrap();

    let page = releases::list(&pool, &ReleaseFilterParams {
        genre: Some("jazz".into()),
        ..Default::default()
    }).await.unwrap();
    assert_eq!(page.total_elements, 1);
    assert_eq!(page.content.len(), 1);
}

#[tokio::test]
async fn rejects_sort_fields_outside_the_whitelist() {
    let (_d, pool) = fresh().await;
    let err = releases::list(&pool, &ReleaseFilterParams {
        sort: Some("notes; DROP TABLE releases".into()),
        ..Default::default()
    }).await;
    assert!(err.is_err(), "unknown sort must be an error, never a silent fallback");

    let still_there: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name='releases'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(still_there, 1);
}

#[tokio::test]
async fn sorts_case_insensitively_by_artist() {
    let (_d, pool) = fresh().await;
    for a in ["zZ Top", "Aphex Twin", "beach House"] {
        releases::create(&pool, req(a, "T")).await.unwrap();
    }
    let page = releases::list(&pool, &ReleaseFilterParams {
        sort: Some("artist".into()),
        direction: Some("ASC".into()),
        ..Default::default()
    }).await.unwrap();
    let artists: Vec<&str> = page.content.iter().map(|r| r.artist.as_str()).collect();
    assert_eq!(artists, vec!["Aphex Twin", "beach House", "zZ Top"]);
}

#[tokio::test]
async fn paginates_without_dropping_or_repeating_rows() {
    let (_d, pool) = fresh().await;
    for i in 0..7 {
        releases::create(&pool, req(&format!("Artist {i}"), "T")).await.unwrap();
    }

    let mut seen = Vec::new();
    for page_no in 0..3 {
        let page = releases::list(&pool, &ReleaseFilterParams {
            page: Some(page_no),
            size: Some(3),
            ..Default::default()
        }).await.unwrap();
        assert_eq!(page.total_elements, 7);
        assert_eq!(page.total_pages, 3);
        assert_eq!(page.first, page_no == 0);
        assert_eq!(page.last, page_no == 2);
        seen.extend(page.content.into_iter().map(|r| r.id));
    }
    assert_eq!(seen.len(), 7, "3 + 3 + 1");
    let unique: std::collections::HashSet<_> = seen.iter().collect();
    assert_eq!(unique.len(), 7, "no row appears on two pages");
}

#[tokio::test]
async fn catalog_search_matches_prefixes_and_survives_fts_operators() {
    let (_d, pool) = fresh().await;
    releases::create(&pool, req("Godspeed You! Black Emperor", "Lift Your Skinny Fists")).await.unwrap();
    releases::create(&pool, req("Sigur Rós", "Ágætis byrjun")).await.unwrap();

    let hits = |q: &'static str| {
        let pool = pool.clone();
        async move { releases::search_catalog(&pool, q, 10).await.unwrap().len() }
    };

    assert_eq!(hits("god").await, 1, "prefix match");
    assert_eq!(hits("skinny").await, 1, "matches on title too");
    assert_eq!(hits("ros").await, 1, "diacritics fold");
    assert_eq!(hits("").await, 0);
    // Raw FTS5 syntax must be neutralised rather than blowing up the query.
    assert_eq!(hits("god OR NOT \"").await, 0, "operators are treated as literals");
    assert_eq!(hits("*").await, 0);
}

#[tokio::test]
async fn search_index_follows_catalog_edits() {
    let (_d, pool) = fresh().await;
    let created = releases::create(&pool, req("Mount Kimbie", "Crooks & Lovers")).await.unwrap();
    assert_eq!(releases::search_catalog(&pool, "crooks", 10).await.unwrap().len(), 1);

    releases::update(&pool, &created.id, ReleaseUpdateRequest {
        title: Some("The Sunset Violent".into()),
        ..Default::default()
    }).await.unwrap();

    assert_eq!(releases::search_catalog(&pool, "crooks", 10).await.unwrap().len(), 0, "stale term gone");
    assert_eq!(releases::search_catalog(&pool, "sunset", 10).await.unwrap().len(), 1, "new term indexed");
}

#[tokio::test]
async fn random_queued_only_returns_queued() {
    let (_d, pool) = fresh().await;
    assert!(releases::random_queued(&pool).await.is_err(), "nothing queued yet");

    add_listened(&pool, "A", "Listened", 4.0, "2026-01-01T00:00:00Z", None, &[]).await;
    assert!(releases::random_queued(&pool).await.is_err(), "listened does not count");

    releases::create(&pool, req("B", "Queued")).await.unwrap();
    for _ in 0..10 {
        assert_eq!(releases::random_queued(&pool).await.unwrap().title, "Queued");
    }
}

#[tokio::test]
async fn stats_aggregate_only_listened_releases() {
    let (_d, pool) = fresh().await;
    add_listened(&pool, "Alice Coltrane", "Journey", 5.0, "2026-03-14T10:00:00Z", Some("US"), &["jazz", "spiritual jazz"]).await;
    add_listened(&pool, "Ryo Fukui", "Scenery", 4.0, "2026-03-20T10:00:00Z", Some("JP"), &["jazz"]).await;
    add_listened(&pool, "Talk Talk", "Laughing Stock", 5.0, "2025-11-02T10:00:00Z", Some("GB"), &["post-rock"]).await;
    // Queued, and therefore invisible to every stat.
    releases::create(&pool, req("Slint", "Spiderland")).await.unwrap();

    let activity = stats::activity(&pool).await.unwrap();
    assert_eq!(activity.len(), 2, "two distinct months");
    assert_eq!((activity[0].year, activity[0].month, activity[0].count), (2025, 11, 1));
    assert_eq!((activity[1].year, activity[1].month, activity[1].count), (2026, 3, 2));

    let genres = stats::by_genre(&pool).await.unwrap();
    assert_eq!(genres[0].label, "jazz");
    assert_eq!(genres[0].count, 2, "counted once per release, not per genre");

    let countries = stats::by_country(&pool).await.unwrap();
    assert_eq!(countries.len(), 3);
    assert!(countries.iter().all(|c| c.count == 1));
}

#[tokio::test]
async fn year_end_respects_the_year_boundary() {
    let (_d, pool) = fresh().await;
    add_listened(&pool, "Late", "Dec 31st", 4.0, "2025-12-31T23:59:59Z", None, &[]).await;
    add_listened(&pool, "Early", "Jan 1st", 5.0, "2026-01-01T00:00:00Z", None, &[]).await;
    add_listened(&pool, "Mid", "Jun", 3.0, "2026-06-15T12:00:00Z", None, &[]).await;

    let y2026 = stats::year_end(&pool, 2026, YearEndBasis::Listened).await.unwrap();
    assert_eq!(y2026.len(), 2, "the December entry belongs to 2025");
    assert_eq!(y2026[0].rank, 1);
    assert_eq!(y2026[0].release.artist, "Early", "ranked by rating");
    assert_eq!(y2026[1].rank, 2);

    let y2025 = stats::year_end(&pool, 2025, YearEndBasis::Listened).await.unwrap();
    assert_eq!(y2025.len(), 1);
    assert_eq!(y2025[0].release.artist, "Late");

    assert!(stats::year_end(&pool, 2024, YearEndBasis::Listened).await.unwrap().is_empty());
}

#[tokio::test]
async fn year_end_by_release_year_ignores_when_it_was_heard() {
    let (_d, pool) = fresh().await;
    let old = add_listened(&pool, "Slint", "Spiderland", 5.0, "2026-03-01T10:00:00Z", None, &[]).await;
    let new = add_listened(&pool, "Gaupa", "Myriad", 4.0, "2023-02-01T10:00:00Z", None, &[]).await;
    add_listened(&pool, "Undated", "No Year", 5.0, "2022-06-01T10:00:00Z", None, &[]).await;
    for (id, year) in [(&old, 1991), (&new, 2022)] {
        sqlx::query("UPDATE releases SET release_year = ? WHERE id = ?")
            .bind(year)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let released = stats::year_end(&pool, 2022, YearEndBasis::Released).await.unwrap();
    assert_eq!(released.len(), 1, "an album with no release year is in no release-year list");
    assert_eq!(released[0].release.title, "Myriad", "listened in 2023, released in 2022");

    let listened = stats::year_end(&pool, 2022, YearEndBasis::Listened).await.unwrap();
    assert_eq!(listened.len(), 1);
    assert_eq!(listened[0].release.title, "No Year");

    assert_eq!(stats::year_end(&pool, 1991, YearEndBasis::Released).await.unwrap()[0].rank, 1);
}

#[tokio::test]
async fn year_end_ranks_at_most_twenty() {
    let (_d, pool) = fresh().await;
    for i in 0..25 {
        let date = format!("2026-01-{:02}T10:00:00Z", i + 1);
        add_listened(&pool, "Artist", &format!("Album {i}"), 4.0, &date, None, &[]).await;
    }

    let list = stats::year_end(&pool, 2026, YearEndBasis::Listened).await.unwrap();
    assert_eq!(list.len(), 20);
    assert_eq!(list[19].rank, 20);
    assert_eq!(list[0].release.title, "Album 24", "ties on rating go to the latest listen");
}

#[tokio::test]
async fn unrated_releases_are_excluded_from_rankings() {
    let (_d, pool) = fresh().await;
    let created = releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();
    releases::update(&pool, &created.id, ReleaseUpdateRequest {
        status: Some(ReleaseStatus::Listened),
        date_listened: Some("2026-05-01T10:00:00Z".into()),
        ..Default::default()
    }).await.unwrap();

    assert!(stats::year_end(&pool, 2026, YearEndBasis::Listened).await.unwrap().is_empty());
    // But it still counts as listening activity.
    assert_eq!(stats::activity(&pool).await.unwrap().len(), 1);
}

/// An album with no id is linked to the one the person chose, which repairs its metadata
/// and stores the id. Hits the live services, so it is excluded from normal runs.
#[tokio::test]
#[ignore = "makes real network requests"]
async fn linking_repairs_metadata_without_touching_user_data() {
    use crate::commands::{link_metadata, refresh_metadata};
    use crate::resolve::Resolver;

    let (_d, pool) = fresh().await;
    let resolver = Resolver::new("0.1.0-test");
    let created = releases::create(&pool, req("Avenged Sevenfold", "City of Evil"))
        .await
        .unwrap();

    // Put it in the state an imported library is in, with the user's own data alongside.
    releases::update(
        &pool,
        &created.id,
        ReleaseUpdateRequest {
            country: Some("CA".into()),
            status: Some(ReleaseStatus::Listened),
            rating: Some(4.5),
            notes: Some("loud".into()),
            date_listened: Some("2026-02-01T12:00:00Z".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let linked = link_metadata(&pool, &resolver, &created.id, "180560ee-2d9d-33cf-8de7-cdaaba610739")
        .await
        .unwrap();

    // Catalog fields repaired.
    assert_eq!(linked.country.as_deref(), Some("US"), "the band, not the pressing");
    assert!(linked.album_art_url.is_some(), "artwork from the release group");
    assert!(linked.album_art_url.unwrap().starts_with("https://"));
    assert!(!linked.genres.is_empty());
    assert_eq!(linked.release_year, Some(2005));
    assert_eq!(
        releases::release_group_id(&pool, &created.id).await.unwrap().as_deref(),
        Some("180560ee-2d9d-33cf-8de7-cdaaba610739")
    );

    // Everything that is the user's is untouched.
    assert_eq!(linked.status, ReleaseStatus::Listened);
    assert_eq!(linked.rating, Some(4.5));
    assert_eq!(linked.notes.as_deref(), Some("loud"));
    assert_eq!(linked.date_listened.as_deref(), Some("2026-02-01T12:00:00Z"));

    // And from now on a plain refresh works, by id.
    assert_eq!(refresh_metadata(&pool, &resolver, &created.id).await.unwrap().title, "City of Evil");
}

/// A release with a group id is refreshed by lookup, not by searching its current text.
/// A title that no longer matches anything must still come back as the album.
#[tokio::test]
#[ignore = "makes real network requests"]
async fn refresh_goes_by_id_when_the_album_has_one() {
    use crate::commands::refresh_metadata;
    use crate::resolve::Resolver;

    let (_d, pool) = fresh().await;
    let mut r = req("Avenged Sevenfold", "a title no search would match");
    r.musicbrainz_release_group_id = Some("180560ee-2d9d-33cf-8de7-cdaaba610739".into());
    let created = releases::create(&pool, r).await.unwrap();

    let refreshed = refresh_metadata(&pool, &Resolver::new("0.1.0-test"), &created.id)
        .await
        .unwrap();
    assert_eq!(refreshed.title, "City of Evil");
    assert_eq!(refreshed.release_year, Some(2005));
}

#[tokio::test]
async fn release_group_id_is_read_back_for_tracked_releases_only() {
    let (_d, pool) = fresh().await;
    let mut with = req("Slint", "Spiderland");
    with.musicbrainz_release_group_id = Some("group-1".into());
    let with = releases::create(&pool, with).await.unwrap();
    let without = releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();

    assert_eq!(releases::release_group_id(&pool, &with.id).await.unwrap().as_deref(), Some("group-1"));
    assert_eq!(releases::release_group_id(&pool, &without.id).await.unwrap(), None);

    releases::delete(&pool, &with.id).await.unwrap();
    assert!(releases::release_group_id(&pool, &with.id).await.is_err(), "untracked is not found");
}

#[tokio::test]
async fn refresh_of_a_missing_release_is_not_found() {
    use crate::commands::refresh_metadata;
    use crate::resolve::Resolver;

    let (_d, pool) = fresh().await;
    // No network is reached: the lookup fails before the resolver is consulted.
    assert!(refresh_metadata(&pool, &Resolver::new("0.1.0-test"), "nope")
        .await
        .is_err());
}

#[tokio::test]
async fn refresh_without_an_id_asks_for_a_choice_instead_of_searching() {
    use crate::commands::refresh_metadata;
    use crate::error::AppError;
    use crate::resolve::Resolver;

    let (_d, pool) = fresh().await;
    let created = releases::create(&pool, req("KEN Mode", "NULL")).await.unwrap();
    // No network is reached: the missing id is found before the resolver is consulted.
    let err = refresh_metadata(&pool, &Resolver::new("0.1.0-test"), &created.id).await.unwrap_err();
    assert!(matches!(err, AppError::NotLinked), "{err:?}");
    assert_eq!(releases::get(&pool, &created.id).await.unwrap().title, "NULL", "nothing written");
}

#[tokio::test]
async fn link_checks_the_id_and_the_release_before_any_request() {
    use crate::commands::link_metadata;
    use crate::error::AppError;
    use crate::resolve::Resolver;

    let (_d, pool) = fresh().await;
    let resolver = Resolver::new("0.1.0-test");
    let created = releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();

    let bad = link_metadata(&pool, &resolver, &created.id, "../release/x").await.unwrap_err();
    assert!(matches!(bad, AppError::Invalid(_)), "{bad:?}");

    let missing = link_metadata(&pool, &resolver, "nope", "180560ee-2d9d-33cf-8de7-cdaaba610739")
        .await
        .unwrap_err();
    assert!(matches!(missing, AppError::NotFound(_)), "{missing:?}");
}

#[tokio::test]
async fn linking_stores_the_id() {
    let (_d, pool) = fresh().await;
    let created = releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();
    releases::link_release_group(&pool, &created.id, "group-1").await.unwrap();
    assert_eq!(releases::release_group_id(&pool, &created.id).await.unwrap().as_deref(), Some("group-1"));

    // Linking again to the same album is not a conflict with itself.
    releases::link_release_group(&pool, &created.id, "group-1").await.unwrap();
}

#[tokio::test]
async fn linking_to_an_album_you_already_track_names_it_and_changes_nothing() {
    use crate::error::AppError;

    let (_d, pool) = fresh().await;
    let mut searched = req("Slint", "Spiderland");
    searched.musicbrainz_release_group_id = Some("group-1".into());
    let searched = releases::create(&pool, searched).await.unwrap();
    let imported = releases::create(&pool, req("slint", "spiderland (remaster)")).await.unwrap();

    let err = releases::link_release_group(&pool, &imported.id, "group-1").await.unwrap_err();
    match err {
        AppError::AlreadyInLibrary { release_id, artist, title } => {
            assert_eq!(release_id, searched.id);
            assert_eq!((artist.as_str(), title.as_str()), ("Slint", "Spiderland"));
        }
        other => panic!("expected AlreadyInLibrary, got {other:?}"),
    }
    assert_eq!(releases::release_group_id(&pool, &imported.id).await.unwrap(), None);
    assert_eq!(releases::release_group_id(&pool, &searched.id).await.unwrap().as_deref(), Some("group-1"));
}

#[tokio::test]
async fn linking_takes_the_id_from_a_release_you_deleted() {
    let (_d, pool) = fresh().await;
    let mut old = req("Slint", "Spiderland");
    old.musicbrainz_release_group_id = Some("group-1".into());
    let old = releases::create(&pool, old).await.unwrap();
    releases::delete(&pool, &old.id).await.unwrap();
    let imported = releases::create(&pool, req("Slint", "Spiderland (import)")).await.unwrap();

    releases::link_release_group(&pool, &imported.id, "group-1").await.unwrap();
    assert_eq!(releases::release_group_id(&pool, &imported.id).await.unwrap().as_deref(), Some("group-1"));
    let holders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM releases WHERE musicbrainz_release_group_id = 'group-1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(holders, 1);
}

#[tokio::test]
async fn unlinked_lists_tracked_albums_without_an_id_in_the_order_added() {
    let (_d, pool) = fresh().await;
    let mut linked = req("Slint", "Spiderland");
    linked.musicbrainz_release_group_id = Some("group-1".into());
    releases::create(&pool, linked).await.unwrap();
    let first = releases::create(&pool, req("Hexis", "Aeternum")).await.unwrap();
    let mut second = req("Gaupa", "Myriad");
    second.release_year = Some(2022);
    second.streaming_links = Some(HashMap::from([("spotify".into(), "https://open.spotify.com/album/1".into())]));
    let second = releases::create(&pool, second).await.unwrap();
    let deleted = releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();
    releases::delete(&pool, &deleted.id).await.unwrap();

    let list = releases::unlinked(&pool).await.unwrap();
    let ids: Vec<_> = list.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, [first.id.as_str(), second.id.as_str()], "no linked, no untracked");
    assert_eq!(list[1].release_year, Some(2022));
    assert!(list[0].streaming_links.is_empty());
    assert_eq!(list[1].streaming_links["spotify"], "https://open.spotify.com/album/1");

    releases::link_release_group(&pool, &first.id, "group-2").await.unwrap();
    assert_eq!(releases::unlinked(&pool).await.unwrap().len(), 1);
}

#[test]
fn genres_merge_keeping_yours_first_and_ignoring_case() {
    use crate::commands::merge_genres;
    let s = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();

    assert_eq!(
        merge_genres(&s(&["atmo black metal", "Sludge"]), &s(&["black metal", "sludge", "post-metal"])),
        s(&["atmo black metal", "Sludge", "black metal", "post-metal"])
    );
    assert_eq!(merge_genres(&s(&["emo"]), &[]), s(&["emo"]), "nothing found removes nothing");
    assert_eq!(merge_genres(&[], &s(&["emo", "Emo", " "])), s(&["emo"]));
}

#[tokio::test]
async fn country_list_covers_the_library_and_nothing_else() {
    let (_d, pool) = fresh().await;
    assert!(releases::list_countries(&pool).await.unwrap().is_empty());

    add_listened(&pool, "Alice Coltrane", "Journey", 5.0, "2026-03-01T10:00:00Z", Some("US"), &[]).await;
    add_listened(&pool, "Ryo Fukui", "Scenery", 4.0, "2026-04-01T10:00:00Z", Some("JP"), &[]).await;

    // Queued releases count too: the library filter covers both statuses.
    let mut queued = req("Slint", "Spiderland");
    queued.country = Some("US".into());
    releases::create(&pool, queued).await.unwrap();

    // No country at all, and a blank one, must not become options.
    releases::create(&pool, req("Nobody", "Nowhere")).await.unwrap();
    let mut blank = req("Blank", "Country");
    blank.country = Some("   ".into());
    releases::create(&pool, blank).await.unwrap();

    let countries = releases::list_countries(&pool).await.unwrap();
    assert_eq!(countries, vec!["JP", "US"], "distinct, sorted, no empties");
}

#[tokio::test]
async fn data_survives_reopening_the_database() {
    let dir = tempfile::tempdir().unwrap();
    let first = db::connect(dir.path()).await.unwrap();
    let created = releases::create(&first.pool, req("Slint", "Spiderland")).await.unwrap();
    first.pool.close().await;

    let second = db::connect(dir.path()).await.unwrap();
    let page = releases::list(&second.pool, &ReleaseFilterParams::default()).await.unwrap();
    assert_eq!(page.total_elements, 1);
    assert_eq!(page.content[0].id, created.id);
    assert_eq!(page.content[0].title, "Spiderland");
}

// ---------------------------------------------------------------- export and import

use crate::commands::{collect_export, import_library};
use crate::library::{self, ImportMode};

/// Exports the whole library and reads it straight back into a second, empty one.
///
/// The point of the format is that the file survives the trip to another machine, and a
/// second temporary database is the closest thing to one a test can have.
async fn round_trip(pool: &SqlitePool, format: library::Format) -> (tempfile::TempDir, SqlitePool) {
    let rows = collect_export(pool).await.unwrap();
    let text = library::render(&rows, format, "0.1.0", "2026-09-12T09:20:00Z").unwrap();
    let (dir, other) = fresh().await;
    let report = import_library(&other, &format!("lib.{}", format.extension()), &text, ImportMode::Skip)
        .await
        .unwrap();
    assert!(report.rejected.is_empty(), "{:?}", report.rejected);
    assert_eq!(report.added, rows.len());
    (dir, other)
}

#[tokio::test]
async fn a_library_survives_a_json_round_trip() {
    let (_d, pool) = fresh().await;
    let id = add_listened(&pool, "Slint", "Spiderland", 4.5, "1991-03-27T00:00:00Z", Some("US"),
                          &["alternative rock", "rock"]).await;
    releases::update(&pool, &id, ReleaseUpdateRequest {
        notes: Some("Needs a full sitting.".into()),
        did_not_finish: Some(true),
        discovery_link: Some("https://example.test/tip".into()),
        streaming_links: Some(HashMap::from([("spotify".into(), "https://open.spotify.com/album/x".into())])),
        ..Default::default()
    }).await.unwrap();
    let before = releases::get(&pool, &id).await.unwrap();

    let (_d2, other) = round_trip(&pool, library::Format::Json).await;
    let after = releases::list(&other, &ReleaseFilterParams::default()).await.unwrap();
    assert_eq!(after.total_elements, 1);
    let r = &after.content[0];

    assert_eq!(r.artist, before.artist);
    assert_eq!(r.title, before.title);
    assert_eq!(r.release_year, before.release_year);
    assert_eq!(r.country, before.country);
    assert_eq!(r.status, ReleaseStatus::Listened);
    assert_eq!(r.rating, before.rating);
    assert_eq!(r.did_not_finish, before.did_not_finish);
    assert_eq!(r.date_listened, before.date_listened);
    assert_eq!(r.notes, before.notes);
    assert_eq!(r.discovery_link, before.discovery_link);
    assert_eq!(r.genres, before.genres);
    assert_eq!(r.streaming_links, before.streaming_links);
    // Not the row id, which is local and deliberately absent from the file, but the date
    // you added it, which is the one timestamp that means something on another machine.
    assert_eq!(r.created_at, before.created_at);
    assert_ne!(r.id, before.id, "a new machine assigns its own row ids");
}

#[tokio::test]
async fn a_library_survives_a_csv_round_trip() {
    let (_d, pool) = fresh().await;
    add_listened(&pool, "Godspeed You! Black Emperor", "Lift Your Skinny Fists, Like Antennas to Heaven",
                 5.0, "2026-01-04T00:00:00Z", Some("CA"), &["post-rock"]).await;
    releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();

    let (_d2, other) = round_trip(&pool, library::Format::Csv).await;
    let page = releases::list(&other, &ReleaseFilterParams::default()).await.unwrap();
    assert_eq!(page.total_elements, 2);

    let gybe = page.content.iter().find(|r| r.artist.starts_with("Godspeed")).unwrap();
    assert_eq!(gybe.title, "Lift Your Skinny Fists, Like Antennas to Heaven",
               "the comma in the title survives the delimiter");
    assert_eq!(gybe.genres, vec!["post-rock"]);
    assert_eq!(gybe.rating, Some(5.0));
    let duster = page.content.iter().find(|r| r.artist == "Duster").unwrap();
    assert_eq!(duster.status, ReleaseStatus::Queued);
    assert_eq!(duster.rating, None);
}

#[tokio::test]
async fn importing_the_same_file_twice_changes_nothing() {
    // The format promises idempotence. Without it, restoring a backup you are not sure
    // completed doubles your library.
    let (_d, pool) = fresh().await;
    add_listened(&pool, "Slint", "Spiderland", 4.5, "1991-03-27T00:00:00Z", Some("US"), &["rock"]).await;
    let text = library::to_json(&collect_export(&pool).await.unwrap(), "0.1.0", "x").unwrap();

    let (_d2, other) = fresh().await;
    let first = import_library(&other, "lib.json", &text, ImportMode::Skip).await.unwrap();
    let second = import_library(&other, "lib.json", &text, ImportMode::Skip).await.unwrap();

    assert_eq!((first.added, first.skipped), (1, 0));
    assert_eq!((second.added, second.skipped), (0, 1));
    assert_eq!(releases::list(&other, &ReleaseFilterParams::default()).await.unwrap().total_elements, 1);
}

#[tokio::test]
async fn skip_leaves_your_own_rating_and_notes_alone() {
    let (_d, pool) = fresh().await;
    let id = add_listened(&pool, "Slint", "Spiderland", 2.0, "2026-01-01T00:00:00Z", Some("US"), &[]).await;
    releases::update(&pool, &id, ReleaseUpdateRequest { notes: Some("mine".into()), ..Default::default() })
        .await.unwrap();

    let incoming = format!(
        r#"{{"format":"{}","formatVersion":1,"releases":[
             {{"artist":"slint","title":"SPIDERLAND","rating":5,"notes":"theirs","status":"LISTENED"}}]}}"#,
        library::FORMAT
    );
    let report = import_library(&pool, "lib.json", &incoming, ImportMode::Skip).await.unwrap();
    assert_eq!(report.skipped, 1);

    let r = releases::get(&pool, &id).await.unwrap();
    assert_eq!(r.rating, Some(2.0));
    assert_eq!(r.notes.as_deref(), Some("mine"));
}

#[tokio::test]
async fn overwrite_replaces_every_field_including_the_ones_it_clears() {
    let (_d, pool) = fresh().await;
    let id = add_listened(&pool, "Slint", "Spiderland", 2.0, "2026-01-01T00:00:00Z", Some("US"),
                          &["rock", "noise"]).await;
    releases::update(&pool, &id, ReleaseUpdateRequest { notes: Some("mine".into()), ..Default::default() })
        .await.unwrap();

    // No rating, no notes, no date and no genres: an overwrite has to clear them, because
    // half-replacing would be the merge mode the format refuses to offer.
    let incoming = format!(
        r#"{{"format":"{}","formatVersion":1,"releases":[
             {{"artist":"Slint","title":"Spiderland","releaseYear":1991,"country":"US"}}]}}"#,
        library::FORMAT
    );
    let report = import_library(&pool, "lib.json", &incoming, ImportMode::Overwrite).await.unwrap();
    assert_eq!((report.added, report.overwritten, report.skipped), (0, 1, 0));

    let r = releases::get(&pool, &id).await.unwrap();
    assert_eq!(r.rating, None);
    assert_eq!(r.notes, None);
    assert_eq!(r.date_listened, None);
    assert_eq!(r.status, ReleaseStatus::Queued);
    assert!(r.genres.is_empty());
    assert_eq!(r.release_year, Some(1991));
}

#[tokio::test]
async fn a_release_group_id_matches_across_a_renamed_title() {
    // Rule one of the identity order. The text can differ; the id is the album.
    let (_d, pool) = fresh().await;
    let mut r = req("Slint", "Spiderland (2014 remaster)");
    r.musicbrainz_release_group_id = Some("266e8eb6-244f-450d-b419-7e3cdf815d4c".into());
    let created = releases::create(&pool, r).await.unwrap();

    let incoming = format!(
        r#"{{"format":"{}","formatVersion":1,"releases":[
             {{"artist":"Slint","title":"Spiderland",
               "musicbrainzId":"266e8eb6-244f-450d-b419-7e3cdf815d4c"}}]}}"#,
        library::FORMAT
    );
    let report = import_library(&pool, "lib.json", &incoming, ImportMode::Skip).await.unwrap();
    assert_eq!(report.skipped, 1);
    assert_eq!(releases::list(&pool, &ReleaseFilterParams::default()).await.unwrap().total_elements, 1);
    assert!(releases::get(&pool, &created.id).await.is_ok());
}

#[tokio::test]
async fn a_release_you_deleted_comes_back_onto_its_own_catalog_row() {
    // Delete keeps the catalog entry. Re-importing has to reuse it rather than inserting a
    // second copy of the same album under a new id.
    let (_d, pool) = fresh().await;
    let id = add_listened(&pool, "Duster", "Stratosphere", 4.0, "2026-01-01T00:00:00Z", None, &[]).await;
    let text = library::to_json(&collect_export(&pool).await.unwrap(), "0.1.0", "x").unwrap();
    releases::delete(&pool, &id).await.unwrap();

    let report = import_library(&pool, "lib.json", &text, ImportMode::Skip).await.unwrap();
    assert_eq!(report.added, 1);

    let page = releases::list(&pool, &ReleaseFilterParams::default()).await.unwrap();
    assert_eq!(page.total_elements, 1);
    assert_eq!(page.content[0].id, id, "the catalog row is reused, so the id is the old one");

    let catalog: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM releases")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(catalog, 1, "no duplicate catalog row");
}

#[tokio::test]
async fn two_rows_sharing_a_release_group_id_become_one_release() {
    // The column is UNIQUE, so a file claiming one id twice cannot produce two rows. The
    // id is the identity: the second row is the same album, described differently.
    let (_d, pool) = fresh().await;
    let incoming = format!(
        r#"{{"format":"{}","formatVersion":1,"releases":[
             {{"artist":"Slint","title":"Spiderland",
               "musicbrainzId":"266e8eb6-244f-450d-b419-7e3cdf815d4c"}},
             {{"artist":"Slint","title":"Spiderland (remaster)",
               "musicbrainzId":"266e8eb6-244f-450d-b419-7e3cdf815d4c"}}]}}"#,
        library::FORMAT
    );

    let report = import_library(&pool, "lib.json", &incoming, ImportMode::Skip).await.unwrap();
    assert_eq!((report.added, report.skipped), (1, 1));
    assert!(report.rejected.is_empty(), "{:?}", report.rejected);

    let page = releases::list(&pool, &ReleaseFilterParams::default()).await.unwrap();
    assert_eq!(page.total_elements, 1);
    assert_eq!(page.content[0].title, "Spiderland", "the first row wins under skip");
}

#[tokio::test]
async fn an_imported_release_is_findable_in_the_search_index() {
    // The FTS table is external-content and kept in step by triggers. An insert that goes
    // around them would leave a release the catalog knows about and search does not.
    let (_d, pool) = fresh().await;
    let incoming = "artist,title\nStereolab,Dots and Loops\n";
    import_library(&pool, "lib.csv", incoming, ImportMode::Skip).await.unwrap();

    let hits = releases::search_catalog(&pool, "stereo", 10).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title.as_deref(), Some("Dots and Loops"));
}

#[tokio::test]
async fn a_file_that_is_not_a_library_is_refused_before_anything_is_written() {
    let (_d, pool) = fresh().await;
    releases::create(&pool, req("Duster", "Stratosphere")).await.unwrap();

    let err = import_library(&pool, "notes.json", r#"{"todo":["buy milk"]}"#, ImportMode::Overwrite)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not a Trecker export"), "{err}");
    assert_eq!(releases::list(&pool, &ReleaseFilterParams::default()).await.unwrap().total_elements, 1);
}

#[tokio::test]
async fn genres_from_a_file_reuse_the_ones_already_there() {
    let (_d, pool) = fresh().await;
    add_listened(&pool, "Slint", "Spiderland", 4.5, "2026-01-01T00:00:00Z", None, &["Jazz"]).await;

    import_library(&pool, "lib.csv", "artist,title,genres\nDuster,Stratosphere,jazz\n", ImportMode::Skip)
        .await
        .unwrap();

    assert_eq!(releases::list_genres(&pool).await.unwrap(), vec!["Jazz"],
               "case-insensitive find-or-create, as on every other path");
}

// ---------------------------------------------------------------- artists

fn artist(mbid: &str, name: &str) -> ArtistMetadata {
    ArtistMetadata {
        musicbrainz_artist_id: mbid.into(),
        name: name.into(),
        country: Some("US".into()),
        begin_year: Some(1999),
        image_url: Some("https://coverartarchive.org/release-group/x/front-250".into()),
        genres: vec!["heavy metal".into(), "metalcore".into()],
        links: vec![
            ArtistLink { kind: "official homepage".into(), url: "https://a.example".into() },
            ArtistLink { kind: "bandcamp".into(), url: "https://a.bandcamp.com".into() },
        ],
        ..Default::default()
    }
}

#[tokio::test]
async fn an_added_artist_comes_back_with_genres_and_links_in_order() {
    let (_d, pool) = fresh().await;
    let added = artists::add(&pool, artist("mb-1", "Avenged Sevenfold"), Some("  from a friend ".into()))
        .await
        .unwrap();

    assert_eq!(added.name, "Avenged Sevenfold");
    assert_eq!(added.status, ArtistStatus::ToCheck);
    assert_eq!(added.note.as_deref(), Some("from a friend"));
    assert_eq!(added.genres, ["heavy metal", "metalcore"]);
    let urls: Vec<&str> = added.links.iter().map(|l| l.url.as_str()).collect();
    assert_eq!(urls, ["https://a.example", "https://a.bandcamp.com"]);

    let listed = artists::list(&pool).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, added.id);
}

#[tokio::test]
async fn adding_an_artist_again_refreshes_the_catalog_and_keeps_your_note_and_status() {
    let (_d, pool) = fresh().await;
    let first = artists::add(&pool, artist("mb-1", "Old Name"), Some("keep me".into())).await.unwrap();
    artists::update(
        &pool,
        &first.id,
        ArtistUpdateRequest { status: Some(ArtistStatus::Checked), verdict: Some(ArtistVerdict::Liked), ..Default::default() },
    )
    .await
    .unwrap();

    let mut fresher = artist("mb-1", "New Name");
    fresher.genres = vec!["rock".into()];
    fresher.image_url = None;
    let again = artists::add(&pool, fresher, Some("ignored".into())).await.unwrap();

    assert_eq!(again.id, first.id, "one catalog row per MusicBrainz artist");
    assert_eq!(again.name, "New Name");
    assert_eq!(again.genres, ["rock"]);
    assert!(again.image_url.is_some(), "a lookup without a picture keeps the old one");
    assert_eq!(again.note.as_deref(), Some("keep me"));
    assert_eq!(again.status, ArtistStatus::Checked);
    assert_eq!(artists::list(&pool).await.unwrap().len(), 1);
}

#[tokio::test]
async fn find_tracked_only_finds_artists_on_your_list() {
    let (_d, pool) = fresh().await;
    let a = artists::add(&pool, artist("mb-1", "A"), None).await.unwrap();
    assert!(artists::find_tracked(&pool, "mb-1").await.unwrap().is_some());
    assert!(artists::find_tracked(&pool, "mb-2").await.unwrap().is_none());

    artists::delete(&pool, &a.id).await.unwrap();
    assert!(artists::find_tracked(&pool, "mb-1").await.unwrap().is_none(), "removed from the list");
    assert!(matches!(artists::get(&pool, &a.id).await, Err(crate::error::AppError::NotFound(_))));
    assert!(matches!(artists::delete(&pool, &a.id).await, Err(crate::error::AppError::NotFound(_))));

    let back = artists::add(&pool, artist("mb-1", "A"), None).await.unwrap();
    assert_eq!(back.id, a.id, "the catalog row survived the delete");
    assert_eq!(back.status, ArtistStatus::ToCheck);
}

#[tokio::test]
async fn checking_an_artist_records_when_and_moving_them_back_clears_it() {
    let (_d, pool) = fresh().await;
    let a = artists::add(&pool, artist("mb-1", "A"), Some("note".into())).await.unwrap();

    let checked = artists::update(
        &pool,
        &a.id,
        ArtistUpdateRequest { status: Some(ArtistStatus::Checked), verdict: Some(ArtistVerdict::NotForMe), ..Default::default() },
    )
    .await
    .unwrap();
    assert_eq!(checked.verdict, Some(ArtistVerdict::NotForMe));
    let when = checked.checked_at.clone().expect("checked_at is set");

    // A new verdict for someone already checked keeps the date.
    let rejudged = artists::update(
        &pool,
        &a.id,
        ArtistUpdateRequest { status: Some(ArtistStatus::Checked), verdict: Some(ArtistVerdict::Liked), ..Default::default() },
    )
    .await
    .unwrap();
    assert_eq!(rejudged.verdict, Some(ArtistVerdict::Liked));
    assert_eq!(rejudged.checked_at.as_deref(), Some(when.as_str()));

    // A note edit alone leaves the status as it is.
    let noted = artists::update(&pool, &a.id, ArtistUpdateRequest { note: Some("new".into()), ..Default::default() })
        .await
        .unwrap();
    assert_eq!(noted.status, ArtistStatus::Checked);
    assert_eq!(noted.note.as_deref(), Some("new"));

    let back = artists::update(&pool, &a.id, ArtistUpdateRequest { status: Some(ArtistStatus::ToCheck), ..Default::default() })
        .await
        .unwrap();
    assert_eq!(back.status, ArtistStatus::ToCheck);
    assert_eq!(back.verdict, None);
    assert_eq!(back.checked_at, None);

    let cleared = artists::update(&pool, &a.id, ArtistUpdateRequest { note: Some("  ".into()), ..Default::default() })
        .await
        .unwrap();
    assert_eq!(cleared.note, None, "an empty note removes it");
}

#[tokio::test]
async fn the_list_puts_artists_to_check_before_checked_ones() {
    let (_d, pool) = fresh().await;
    let a = artists::add(&pool, artist("mb-a", "A"), None).await.unwrap();
    let b = artists::add(&pool, artist("mb-b", "B"), None).await.unwrap();
    artists::update(&pool, &a.id, ArtistUpdateRequest { status: Some(ArtistStatus::Checked), ..Default::default() })
        .await
        .unwrap();
    let order: Vec<String> = artists::list(&pool).await.unwrap().into_iter().map(|x| x.id).collect();
    assert_eq!(order, [b.id, a.id]);
}

#[tokio::test]
async fn artist_genres_stay_out_of_the_library_genre_list() {
    let (_d, pool) = fresh().await;
    artists::add(&pool, artist("mb-1", "A"), None).await.unwrap();
    assert!(releases::list_genres(&pool).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_discography_is_marked_with_what_you_already_track() {
    let (_d, pool) = fresh().await;
    let mut queued = req("Avenged Sevenfold", "Nightmare");
    queued.musicbrainz_release_group_id = Some("group-nightmare".into());
    let queued = releases::create(&pool, queued).await.unwrap();

    let mut listened = req("Avenged Sevenfold", "City of Evil");
    listened.musicbrainz_release_group_id = Some("group-city".into());
    let listened = releases::create(&pool, listened).await.unwrap();
    releases::update(
        &pool,
        &listened.id,
        ReleaseUpdateRequest { status: Some(ReleaseStatus::Listened), rating: Some(4.5), ..Default::default() },
    )
    .await
    .unwrap();

    let album = |id: &str| AlbumCandidate {
        musicbrainz_release_group_id: id.into(),
        artist: None,
        title: Some(id.into()),
        release_year: None,
        primary_type: Some("Album".into()),
        secondary_types: vec![],
        disambiguation: None,
        album_art_url: String::new(),
    };
    let entries = crate::commands::attach_library(
        &pool,
        vec![album("group-city"), album("group-other"), album("group-nightmare")],
    )
    .await
    .unwrap();

    assert_eq!(
        entries[0].library,
        Some(LibraryMatch { release_id: listened.id, status: ReleaseStatus::Listened, rating: Some(4.5) })
    );
    assert_eq!(entries[1].library, None);
    assert_eq!(
        entries[2].library,
        Some(LibraryMatch { release_id: queued.id, status: ReleaseStatus::Queued, rating: None })
    );
}
