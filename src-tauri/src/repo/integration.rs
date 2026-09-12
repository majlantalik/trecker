//! Integration tests for the repository layer, against a real SQLite file.
//!
//! These run the actual migration and the actual SQL. They exist because the Phase 3 port
//! moved a lot of behaviour out of JPA, where the framework guaranteed it, into SQL where
//! nothing does: dedup on external ids, genre find-or-create, cascade on delete, the
//! filter predicates, pagination arithmetic and the date-range maths in the stats.

use super::{releases, stats};
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
    r.genres = Some(genres.iter().map(|s| s.to_string()).collect());
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

    let top = stats::top_rated(&pool, 25).await.unwrap();
    assert_eq!(top.len(), 3, "the queued release is excluded");
    assert_eq!(top[0].rating, Some(5.0));
    assert_eq!(top[2].rating, Some(4.0));
}

#[tokio::test]
async fn year_end_respects_the_year_boundary() {
    let (_d, pool) = fresh().await;
    add_listened(&pool, "Late", "Dec 31st", 4.0, "2025-12-31T23:59:59Z", None, &[]).await;
    add_listened(&pool, "Early", "Jan 1st", 5.0, "2026-01-01T00:00:00Z", None, &[]).await;
    add_listened(&pool, "Mid", "Jun", 3.0, "2026-06-15T12:00:00Z", None, &[]).await;

    let y2026 = stats::year_end(&pool, 2026).await.unwrap();
    assert_eq!(y2026.len(), 2, "the December entry belongs to 2025");
    assert_eq!(y2026[0].rank, 1);
    assert_eq!(y2026[0].release.artist, "Early", "ranked by rating");
    assert_eq!(y2026[1].rank, 2);

    let y2025 = stats::year_end(&pool, 2025).await.unwrap();
    assert_eq!(y2025.len(), 1);
    assert_eq!(y2025[0].release.artist, "Late");

    assert!(stats::year_end(&pool, 2024).await.unwrap().is_empty());
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

    assert!(stats::top_rated(&pool, 25).await.unwrap().is_empty());
    assert!(stats::year_end(&pool, 2026).await.unwrap().is_empty());
    // But it still counts as listening activity.
    assert_eq!(stats::activity(&pool).await.unwrap().len(), 1);
}

/// Refresh repairs a record whose metadata was captured before the resolver understood
/// release groups: no artwork, and the country of whichever pressing the search matched.
/// Hits the live services, so it is excluded from normal runs.
#[tokio::test]
#[ignore = "makes real network requests"]
async fn refresh_repairs_stale_metadata_without_touching_user_data() {
    use crate::commands::refresh_metadata;
    use crate::resolve::Resolver;

    let (_d, pool) = fresh().await;
    let created = releases::create(&pool, req("Avenged Sevenfold", "City of Evil"))
        .await
        .unwrap();

    // Put it in the state the real library was in, and add the user's own data alongside.
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

    let refreshed = refresh_metadata(&pool, &Resolver::new("0.1.0-test"), &created.id)
        .await
        .unwrap();

    // Catalog fields repaired.
    assert_eq!(refreshed.country.as_deref(), Some("US"), "the band, not the pressing");
    assert!(refreshed.album_art_url.is_some(), "artwork from the release group");
    assert!(refreshed.album_art_url.unwrap().starts_with("https://"));
    assert!(!refreshed.genres.is_empty());
    assert_eq!(refreshed.release_year, Some(2005));

    // Everything that is the user's is untouched.
    assert_eq!(refreshed.status, ReleaseStatus::Listened);
    assert_eq!(refreshed.rating, Some(4.5));
    assert_eq!(refreshed.notes.as_deref(), Some("loud"));
    assert_eq!(refreshed.date_listened.as_deref(), Some("2026-02-01T12:00:00Z"));
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
