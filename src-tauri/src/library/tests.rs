//! Tests for the file format itself. No database: these are about what the bytes mean.

use super::*;

fn sample() -> ExportedRelease {
    ExportedRelease {
        artist: "Slint".into(),
        title: "Spiderland".into(),
        release_year: Some(1991),
        country: Some("US".into()),
        album_art_url: Some("https://coverartarchive.org/release/266e/front-500.jpg".into()),
        musicbrainz_id: Some("266e8eb6-244f-450d-b419-7e3cdf815d4c".into()),
        genres: vec!["alternative rock".into(), "rock".into()],
        streaming_links: BTreeMap::from([(
            "spotify".to_string(),
            "https://open.spotify.com/album/x".to_string(),
        )]),
        status: "LISTENED".into(),
        rating: Some(4.5),
        did_not_finish: false,
        date_listened: Some("2026-09-01T19:30:00Z".into()),
        notes: Some("Long-form crescendo. Needs a full sitting.".into()),
        discovery_link: None,
        added_at: Some("2026-08-20T10:00:00Z".into()),
    }
}

fn only(parsed: &ParsedFile) -> &ExportedRelease {
    assert!(parsed.rejected.is_empty(), "{:?}", parsed.rejected);
    assert_eq!(parsed.releases.len(), 1);
    &parsed.releases[0].1
}

// ---------------------------------------------------------------- shape

#[test]
fn the_csv_header_is_the_json_field_list_in_order() {
    // The format promises the two sides need no mapping table between them. That holds
    // only if every column is a JSON key and they appear in the same order.
    let json = serde_json::to_string(&sample()).unwrap();
    let fields: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(fields.len(), COLUMNS.len(), "a field exists that is not a column");

    let mut last = 0;
    for column in COLUMNS {
        let at = json
            .find(&format!("\"{column}\":"))
            .unwrap_or_else(|| panic!("{column} is not a field of ExportedRelease"));
        assert!(at >= last, "{column} is out of order");
        last = at;
    }
}

#[test]
fn the_envelope_says_what_the_file_is() {
    let text = to_json(&[sample()], "0.1.0", "2026-09-12T09:20:00Z").unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["format"], FORMAT);
    assert_eq!(v["formatVersion"], 1);
    assert_eq!(v["appVersion"], "0.1.0");
    assert_eq!(v["exportedAt"], "2026-09-12T09:20:00Z");
    assert_eq!(v["releaseCount"], 1);
}

#[test]
fn an_empty_library_is_still_a_valid_file() {
    for format in [Format::Json, Format::Csv] {
        let text = render(&[], format, "0.1.0", "2026-09-12T09:20:00Z").unwrap();
        let parsed = parse(&text, format).unwrap();
        assert!(parsed.releases.is_empty() && parsed.rejected.is_empty());
    }
}

// ---------------------------------------------------------------- round trips

#[test]
fn json_round_trips_every_field() {
    let text = to_json(&[sample()], "0.1.0", "2026-09-12T09:20:00Z").unwrap();
    let back = parse(&text, Format::Json).unwrap();
    let r = only(&back);
    let s = sample();
    assert_eq!(r.artist, s.artist);
    assert_eq!(r.release_year, s.release_year);
    assert_eq!(r.musicbrainz_id, s.musicbrainz_id);
    assert_eq!(r.genres, s.genres);
    assert_eq!(r.streaming_links, s.streaming_links);
    assert_eq!(r.status, s.status);
    assert_eq!(r.rating, s.rating);
    assert_eq!(r.date_listened, s.date_listened);
    assert_eq!(r.notes, s.notes);
    assert_eq!(r.added_at, s.added_at);
}

#[test]
fn csv_round_trips_every_field() {
    let text = to_csv(&[sample()]);
    let back = parse(&text, Format::Csv).unwrap();
    let r = only(&back);
    let s = sample();
    assert_eq!(r.artist, s.artist);
    assert_eq!(r.release_year, s.release_year);
    assert_eq!(r.country, s.country);
    assert_eq!(r.album_art_url, s.album_art_url);
    assert_eq!(r.musicbrainz_id, s.musicbrainz_id);
    assert_eq!(r.genres, s.genres, "joined with a semicolon and split back");
    assert_eq!(r.streaming_links, s.streaming_links, "JSON in one cell");
    assert_eq!(r.status, s.status);
    assert_eq!(r.rating, s.rating);
    assert_eq!(r.did_not_finish, s.did_not_finish);
    assert_eq!(r.date_listened, s.date_listened);
    assert_eq!(r.notes, s.notes);
    assert_eq!(r.added_at, s.added_at);
}

#[test]
fn csv_survives_a_comma_in_a_title_and_a_newline_in_a_note() {
    let mut r = sample();
    r.title = "Lift Your Skinny Fists, Like Antennas to Heaven".into();
    r.notes = Some("first line\nsecond line".into());
    let back = parse(&to_csv(&[r.clone()]), Format::Csv).unwrap();
    assert_eq!(only(&back).title, r.title);
    assert_eq!(only(&back).notes, r.notes);
}

#[test]
fn csv_writes_nothing_rather_than_braces_for_an_empty_link_set() {
    let mut r = sample();
    r.streaming_links.clear();
    r.genres.clear();
    let text = to_csv(&[r]);
    let row = text.lines().nth(1).unwrap();
    assert!(!row.contains("{}"), "{row}");
    let back = parse(&text, Format::Csv).unwrap();
    assert!(only(&back).streaming_links.is_empty());
    assert!(only(&back).genres.is_empty());
}

#[test]
fn an_empty_cell_reads_as_null() {
    let text = "artist,title,releaseYear,country,rating,notes\nDuster,Stratosphere,,,,\n";
    let back = parse(text, Format::Csv).unwrap();
    let r = only(&back);
    assert_eq!(r.release_year, None);
    assert_eq!(r.country, None);
    assert_eq!(r.rating, None);
    assert_eq!(r.notes, None);
}

// ---------------------------------------------------------------- reading loosely

#[test]
fn a_csv_of_two_columns_is_enough() {
    // The point of CSV is that a person can type one. Everything but artist and title has
    // a defensible default.
    let back = parse("artist,title\nDuster,Stratosphere\n", Format::Csv).unwrap();
    let r = only(&back);
    assert_eq!(r.status, "QUEUED");
    assert_eq!(r.did_not_finish, false);
    assert_eq!(r.added_at, None, "filled in at write time, not invented here");
}

#[test]
fn headers_are_matched_ignoring_case_and_spacing() {
    let back = parse(" Artist , TITLE \nDuster,Stratosphere\n", Format::Csv).unwrap();
    assert_eq!(only(&back).artist, "Duster");
}

#[test]
fn unknown_columns_are_ignored() {
    let back = parse("artist,title,mood\nDuster,Stratosphere,hazy\n", Format::Csv).unwrap();
    assert_eq!(only(&back).title, "Stratosphere");
}

#[test]
fn a_bare_date_becomes_a_timestamp() {
    // Everything downstream sorts and groups these as strings, so two shapes in the
    // column would quietly break the activity chart.
    let back = parse(
        "artist,title,dateListened\nDuster,Stratosphere,2026-09-01\n",
        Format::Csv,
    )
    .unwrap();
    assert_eq!(only(&back).date_listened.as_deref(), Some("2026-09-01T00:00:00Z"));
}

#[test]
fn did_not_finish_accepts_what_a_spreadsheet_writes() {
    for (cell, expected) in [("true", true), ("TRUE", true), ("1", true), ("yes", true),
                             ("false", false), ("0", false), ("", false)] {
        let text = format!("artist,title,didNotFinish\nDuster,Stratosphere,{cell}\n");
        let back = parse(&text, Format::Csv).unwrap();
        assert_eq!(only(&back).did_not_finish, expected, "cell {cell:?}");
    }
}

#[test]
fn genres_are_sorted_and_deduplicated_case_insensitively() {
    // `set_genres` finds existing genres with COLLATE NOCASE, so two spellings in one file
    // would otherwise become one genre and a wasted write.
    let back = parse(
        "artist,title,genres\nDuster,Stratosphere,Rock; slowcore ; rock;;\n",
        Format::Csv,
    )
    .unwrap();
    assert_eq!(only(&back).genres, vec!["Rock", "slowcore"]);
}

#[test]
fn whitespace_only_values_become_null() {
    let back = parse("artist,title,notes\n  Duster  ,Stratosphere,\"   \"\n", Format::Csv).unwrap();
    assert_eq!(only(&back).artist, "Duster");
    assert_eq!(only(&back).notes, None);
}

// ---------------------------------------------------------------- refusing

#[test]
fn a_json_file_that_is_not_ours_is_refused_by_name() {
    let err = parse(r#"{"releases":[]}"#, Format::Json).unwrap_err().to_string();
    assert!(err.contains("not a Trecker export"), "{err}");
}

#[test]
fn a_newer_format_version_is_refused_rather_than_guessed_at() {
    let text = format!(r#"{{"format":"{FORMAT}","formatVersion":99,"releases":[]}}"#);
    let err = parse(&text, Format::Json).unwrap_err().to_string();
    assert!(err.contains("99") && err.contains("reads up to 1"), "{err}");
}

#[test]
fn a_release_count_that_disagrees_with_the_array_stops_the_import() {
    // The only symptom of a truncated file, and it has to be caught before anything is
    // written rather than after half a library is in.
    let text = format!(r#"{{"format":"{FORMAT}","formatVersion":1,"releaseCount":9,"releases":[]}}"#);
    let err = parse(&text, Format::Json).unwrap_err().to_string();
    assert!(err.contains("truncated"), "{err}");
}

#[test]
fn a_csv_without_a_title_column_is_not_a_library() {
    let err = parse("artist,mood\nDuster,hazy\n", Format::Csv).unwrap_err().to_string();
    assert!(err.contains("title"), "{err}");
}

#[test]
fn one_bad_row_is_rejected_and_the_rest_are_kept() {
    let text = "artist,title,rating\nDuster,Stratosphere,4\n,Nameless,3\nSlint,Spiderland,5\n";
    let parsed = parse(text, Format::Csv).unwrap();
    assert_eq!(parsed.releases.len(), 2);
    assert_eq!(parsed.rejected.len(), 1);
    assert_eq!(parsed.rejected[0].row, 2);
    assert_eq!(parsed.rejected[0].title, "Nameless");
    assert!(parsed.rejected[0].reason.contains("artist and title"));
}

#[test]
fn a_rejected_json_row_is_still_identifiable() {
    let text = format!(
        r#"{{"format":"{FORMAT}","formatVersion":1,"releases":[
             {{"artist":"Duster","title":"Stratosphere","releaseYear":"not a year"}}]}}"#
    );
    let parsed = parse(&text, Format::Json).unwrap();
    assert_eq!(parsed.releases.len(), 0);
    assert_eq!(parsed.rejected[0].artist, "Duster");
    assert_eq!(parsed.rejected[0].title, "Stratosphere");
}

#[test]
fn an_impossible_rating_rejects_its_row_rather_than_being_rounded() {
    // Quietly changing a rating is the same invisible edit the format refuses to make
    // when it declines to offer a merge mode.
    for bad in ["4.3", "6", "0"] {
        let text = format!("artist,title,rating\nDuster,Stratosphere,{bad}\n");
        let parsed = parse(&text, Format::Csv).unwrap();
        assert_eq!(parsed.rejected.len(), 1, "rating {bad}");
        assert!(parsed.rejected[0].reason.contains("half steps"));
    }
}

#[test]
fn an_unknown_status_rejects_one_row_not_the_file() {
    let text = "artist,title,status\nDuster,Stratosphere,LISTENING\nSlint,Spiderland,LISTENED\n";
    let parsed = parse(text, Format::Csv).unwrap();
    assert_eq!(parsed.releases.len(), 1);
    assert!(parsed.rejected[0].reason.contains("LISTENING"));
}

#[test]
fn an_unreadable_date_rejects_its_row() {
    let text = "artist,title,dateListened\nDuster,Stratosphere,last tuesday\n";
    let parsed = parse(text, Format::Csv).unwrap();
    assert!(parsed.rejected[0].reason.contains("dateListened"), "{:?}", parsed.rejected);
}

// ---------------------------------------------------------------- paths

#[test]
fn the_extension_the_user_typed_decides_the_format() {
    assert_eq!(export_path("/tmp/lib.csv", Format::Json), ("/tmp/lib.csv".into(), Format::Csv));
    assert_eq!(export_path("/tmp/lib.json", Format::Csv), ("/tmp/lib.json".into(), Format::Json));
}

#[test]
fn a_name_without_an_extension_gets_the_one_that_was_asked_for() {
    assert_eq!(export_path("/tmp/lib", Format::Csv), ("/tmp/lib.csv".into(), Format::Csv));
    assert_eq!(export_path("/tmp/lib", Format::Json), ("/tmp/lib.json".into(), Format::Json));
}

#[test]
fn an_unfamiliar_extension_falls_back_to_the_content() {
    // A save dialog lets you type any name at all, so a file called .txt still has to be
    // readable rather than being refused for its name.
    assert_eq!(detect("library.txt", "  {\"format\":\"x\"}"), Format::Json);
    assert_eq!(detect("library.txt", "artist,title\n"), Format::Csv);
    assert_eq!(detect("library.CSV", "{"), Format::Csv, "the name wins when it is known");
}

#[test]
fn the_words_the_frontend_sends_are_the_ones_rust_reads() {
    // `ExportFormat` and `ImportMode` in frontend/src/types/index.ts are string unions.
    // Nothing else checks that their members survive serde's renaming.
    assert_eq!(serde_json::from_str::<Format>("\"json\"").unwrap(), Format::Json);
    assert_eq!(serde_json::from_str::<Format>("\"csv\"").unwrap(), Format::Csv);
    assert_eq!(serde_json::from_str::<ImportMode>("\"skip\"").unwrap(), ImportMode::Skip);
    assert_eq!(
        serde_json::from_str::<ImportMode>("\"overwrite\"").unwrap(),
        ImportMode::Overwrite
    );
    assert_eq!(serde_json::to_string(&Format::Csv).unwrap(), "\"csv\"");
}
