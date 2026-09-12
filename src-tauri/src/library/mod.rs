//! The library file format: rendering an export and reading one back.
//!
//! The specification lives in `docs/export-format.md` and was written before this module.
//! Where the two disagree the document is right and this is a bug.
//!
//! Nothing here touches the database. Rendering takes rows and returns text; parsing takes
//! text and returns rows plus the ones it refused. `repo::releases` does the writing, so
//! every decision about what the format means is testable without a SQLite file.

mod csv;
#[cfg(test)]
mod tests;

use crate::domain::ReleaseStatus;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The discriminator. Without it, reading an arbitrary JSON file would begin by guessing
/// whether it was ever meant for us.
pub const FORMAT: &str = "trecker-library";

/// Bumped only when an older reader would read a newer file wrongly. Adding an optional
/// field does not qualify; renaming or repurposing one does.
pub const FORMAT_VERSION: u32 = 1;

/// The CSV header row, and the JSON field order. One list, so the two formats cannot
/// drift apart into needing a mapping table between them.
pub const COLUMNS: [&str; 15] = [
    "artist",
    "title",
    "releaseYear",
    "country",
    "albumArtUrl",
    "musicbrainzId",
    "genres",
    "streamingLinks",
    "status",
    "rating",
    "didNotFinish",
    "dateListened",
    "notes",
    "discoveryLink",
    "addedAt",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Json,
    Csv,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Csv => "csv",
        }
    }

    fn from_extension(path: &str) -> Option<Self> {
        let ext = path.rsplit_once('.')?.1.to_ascii_lowercase();
        match ext.as_str() {
            "json" => Some(Format::Json),
            "csv" => Some(Format::Csv),
            _ => None,
        }
    }
}

/// What to do when an incoming release is already in the library.
///
/// There is deliberately no merge mode: merging needs a rule for every field, and the
/// honest rule for a rating or a note is that one of the two is simply wrong and only the
/// person can say which.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportMode {
    #[default]
    Skip,
    Overwrite,
}

/// One release as it appears in a file.
///
/// Field order is the order they serialize in, which is the order of `COLUMNS`. Local row
/// ids are absent on purpose: they mean nothing on another machine, and carrying them
/// would invite a reader to trust them.
///
/// Everything but artist and title has a default, so a hand-written file with three
/// columns is readable. `status` is a plain string rather than the enum so that one
/// mistyped value rejects one row instead of failing the whole file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedRelease {
    pub artist: String,
    pub title: String,
    #[serde(default)]
    pub release_year: Option<i32>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub album_art_url: Option<String>,
    #[serde(default)]
    pub musicbrainz_id: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub streaming_links: BTreeMap<String, String>,
    #[serde(default = "queued")]
    pub status: String,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub did_not_finish: bool,
    #[serde(default)]
    pub date_listened: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub discovery_link: Option<String>,
    /// The tracking row's `created_at`: when *you* added it. Renamed on the way out
    /// because "created" in a file a human may open reads as being about the album.
    #[serde(default)]
    pub added_at: Option<String>,
}

fn queued() -> String {
    "QUEUED".into()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryExport {
    pub format: String,
    pub format_version: u32,
    pub exported_at: String,
    pub app_version: String,
    /// Redundant with the array length on purpose: a mismatch means truncation, which is
    /// worth catching before anything is written.
    pub release_count: usize,
    pub releases: Vec<ExportedRelease>,
}

/// A row the importer refused, with enough of it to recognise which one.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectedRow {
    /// 1-based position among the releases in the file, not the line number: a CSV value
    /// may span several lines.
    pub row: usize,
    pub artist: String,
    pub title: String,
    pub reason: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub added: usize,
    pub overwritten: usize,
    pub skipped: usize,
    pub rejected: Vec<RejectedRow>,
}

/// The valid rows of a file and the ones it refused, in file order.
#[derive(Debug, Default)]
pub struct ParsedFile {
    pub releases: Vec<(usize, ExportedRelease)>,
    pub rejected: Vec<RejectedRow>,
}

// ---------------------------------------------------------------- rendering

pub fn render(
    releases: &[ExportedRelease],
    format: Format,
    app_version: &str,
    exported_at: &str,
) -> AppResult<String> {
    match format {
        Format::Json => to_json(releases, app_version, exported_at),
        Format::Csv => Ok(to_csv(releases)),
    }
}

pub fn to_json(
    releases: &[ExportedRelease],
    app_version: &str,
    exported_at: &str,
) -> AppResult<String> {
    let doc = LibraryExport {
        format: FORMAT.into(),
        format_version: FORMAT_VERSION,
        exported_at: exported_at.into(),
        app_version: app_version.into(),
        release_count: releases.len(),
        releases: releases.to_vec(),
    };
    // Pretty-printed: this is a file a person may open, diff or hand-edit, and the size
    // difference is noise next to the album art URLs it already carries.
    let mut text = serde_json::to_string_pretty(&doc)
        .map_err(|e| AppError::Internal(format!("could not write the export: {e}")))?;
    text.push('\n');
    Ok(text)
}

pub fn to_csv(releases: &[ExportedRelease]) -> String {
    let mut out = String::new();
    csv::write_record(&mut out, COLUMNS);
    for r in releases {
        let fields = csv_fields(r);
        csv::write_record(&mut out, fields.iter().map(String::as_str));
    }
    out
}

fn csv_fields(r: &ExportedRelease) -> Vec<String> {
    let links = if r.streaming_links.is_empty() {
        // An empty cell rather than "{}". Every other empty value in the file is blank,
        // and a spreadsheet column of braces reads as data when it is not.
        String::new()
    } else {
        serde_json::to_string(&r.streaming_links).unwrap_or_default()
    };

    vec![
        r.artist.clone(),
        r.title.clone(),
        r.release_year.map(|y| y.to_string()).unwrap_or_default(),
        r.country.clone().unwrap_or_default(),
        r.album_art_url.clone().unwrap_or_default(),
        r.musicbrainz_id.clone().unwrap_or_default(),
        r.genres.join("; "),
        links,
        r.status.clone(),
        r.rating.map(|v| v.to_string()).unwrap_or_default(),
        r.did_not_finish.to_string(),
        r.date_listened.clone().unwrap_or_default(),
        r.notes.clone().unwrap_or_default(),
        r.discovery_link.clone().unwrap_or_default(),
        r.added_at.clone().unwrap_or_default(),
    ]
}

// ---------------------------------------------------------------- parsing

/// Picks the format from the file name, falling back to the first character of the
/// content. A file called `library.txt` is still readable, which matters because a save
/// dialog will happily let you type any name at all.
pub fn detect(path: &str, text: &str) -> Format {
    Format::from_extension(path).unwrap_or_else(|| {
        if text.trim_start().starts_with('{') {
            Format::Json
        } else {
            Format::Csv
        }
    })
}

/// Makes sure an export path carries the extension for what is about to be written into
/// it. An extension the user typed wins, because they were more specific than the filter
/// dropdown they may never have looked at.
pub fn export_path(path: &str, requested: Format) -> (String, Format) {
    match Format::from_extension(path) {
        Some(f) => (path.to_string(), f),
        None => (format!("{path}.{}", requested.extension()), requested),
    }
}

pub fn parse(text: &str, format: Format) -> AppResult<ParsedFile> {
    match format {
        Format::Json => parse_json(text),
        Format::Csv => parse_csv(text),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    #[serde(default)]
    format: String,
    #[serde(default)]
    format_version: u32,
    #[serde(default)]
    release_count: Option<usize>,
    /// Read loosely so that one malformed release is one rejected row rather than an
    /// unreadable file.
    #[serde(default)]
    releases: Vec<serde_json::Value>,
}

fn parse_json(text: &str) -> AppResult<ParsedFile> {
    let env: Envelope = serde_json::from_str(text)
        .map_err(|e| AppError::Invalid(format!("this is not a readable JSON file: {e}")))?;

    if env.format != FORMAT {
        return Err(AppError::Invalid(format!(
            "this is not a Trecker export: its \"format\" is {}, not \"{FORMAT}\"",
            if env.format.is_empty() {
                "missing".to_string()
            } else {
                format!("\"{}\"", env.format)
            }
        )));
    }
    if env.format_version > FORMAT_VERSION {
        return Err(AppError::Invalid(format!(
            "this file is format version {}; this version of Trecker reads up to {FORMAT_VERSION}",
            env.format_version
        )));
    }
    if let Some(claimed) = env.release_count {
        if claimed != env.releases.len() {
            return Err(AppError::Invalid(format!(
                "the file says it holds {claimed} releases but contains {}; it looks truncated, so nothing was imported",
                env.releases.len()
            )));
        }
    }

    let mut parsed = ParsedFile::default();
    for (i, value) in env.releases.into_iter().enumerate() {
        let row = i + 1;
        // Names come out of the raw value so that a row too broken to deserialize is still
        // identifiable in the report.
        let (artist, title) = names_from_json(&value);
        match serde_json::from_value::<ExportedRelease>(value) {
            Ok(r) => push(&mut parsed, row, r),
            Err(e) => parsed.rejected.push(RejectedRow {
                row,
                artist,
                title,
                reason: e.to_string(),
            }),
        }
    }
    Ok(parsed)
}

fn names_from_json(value: &serde_json::Value) -> (String, String) {
    let get = |k: &str| {
        value
            .get(k)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };
    (get("artist"), get("title"))
}

fn parse_csv(text: &str) -> AppResult<ParsedFile> {
    let records = csv::parse(text).map_err(|e| AppError::Invalid(format!("unreadable CSV: {e}")))?;
    let Some((header, rows)) = records.split_first() else {
        return Err(AppError::Invalid("the file is empty".into()));
    };

    // CSV carries no envelope, so it has no version and no discriminator. The header row
    // is the only thing that says this file is a library, and artist and title are the
    // only two columns nothing can stand in for.
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    for (i, name) in header.iter().enumerate() {
        index.insert(name.trim().to_ascii_lowercase(), i);
    }
    for required in ["artist", "title"] {
        if !index.contains_key(required) {
            return Err(AppError::Invalid(format!(
                "this CSV has no \"{required}\" column, so it is not a library export"
            )));
        }
    }

    let mut parsed = ParsedFile::default();
    for (i, record) in rows.iter().enumerate() {
        let row = i + 1;
        let cell = |name: &str| -> String {
            index
                .get(&name.to_ascii_lowercase())
                .and_then(|&c| record.get(c))
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };

        match csv_release(&cell) {
            Ok(r) => push(&mut parsed, row, r),
            Err(reason) => parsed.rejected.push(RejectedRow {
                row,
                artist: cell("artist"),
                title: cell("title"),
                reason,
            }),
        }
    }
    Ok(parsed)
}

fn csv_release(cell: &dyn Fn(&str) -> String) -> Result<ExportedRelease, String> {
    let release_year = match cell("releaseYear").as_str() {
        "" => None,
        v => Some(
            v.parse::<i32>()
                .map_err(|_| format!("releaseYear is not a number: {v}"))?,
        ),
    };
    let rating = match cell("rating").as_str() {
        "" => None,
        v => Some(
            v.parse::<f64>()
                .map_err(|_| format!("rating is not a number: {v}"))?,
        ),
    };
    let streaming_links = match cell("streamingLinks").as_str() {
        "" => BTreeMap::new(),
        v => serde_json::from_str(v).map_err(|e| format!("streamingLinks is not valid JSON: {e}"))?,
    };

    Ok(ExportedRelease {
        artist: cell("artist"),
        title: cell("title"),
        release_year,
        country: blank_to_none(cell("country")),
        album_art_url: blank_to_none(cell("albumArtUrl")),
        musicbrainz_id: blank_to_none(cell("musicbrainzId")),
        genres: cell("genres")
            .split(';')
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty())
            .collect(),
        streaming_links,
        status: cell("status"),
        rating,
        did_not_finish: parse_bool(&cell("didNotFinish"))?,
        date_listened: blank_to_none(cell("dateListened")),
        notes: blank_to_none(cell("notes")),
        discovery_link: blank_to_none(cell("discoveryLink")),
        added_at: blank_to_none(cell("addedAt")),
    })
}

fn parse_bool(v: &str) -> Result<bool, String> {
    match v.to_ascii_lowercase().as_str() {
        "" | "false" | "0" | "no" => Ok(false),
        "true" | "1" | "yes" => Ok(true),
        other => Err(format!("didNotFinish is not true or false: {other}")),
    }
}

fn push(parsed: &mut ParsedFile, row: usize, release: ExportedRelease) {
    match validate(release) {
        Ok(r) => parsed.releases.push((row, r)),
        Err((artist, title, reason)) => parsed.rejected.push(RejectedRow {
            row,
            artist,
            title,
            reason,
        }),
    }
}

// ---------------------------------------------------------------- validation

/// Normalises a row and refuses the ones that cannot be stored.
///
/// A bad value rejects its row rather than being quietly corrected. Silently rounding
/// someone's rating is the same class of invisible change that the format refuses to make
/// when it declines to offer a merge mode.
fn validate(r: ExportedRelease) -> Result<ExportedRelease, (String, String, String)> {
    let artist = r.artist.trim().to_string();
    let title = r.title.trim().to_string();
    check(r).map_err(|reason| (artist, title, reason))
}

fn check(mut r: ExportedRelease) -> Result<ExportedRelease, String> {
    r.artist = r.artist.trim().to_string();
    r.title = r.title.trim().to_string();
    if r.artist.is_empty() || r.title.is_empty() {
        return Err("artist and title are required".into());
    }

    r.status = match r.status.trim().to_ascii_uppercase().as_str() {
        // An empty status means a hand-made file that never had the column. Queued is the
        // state every release passes through, so it is the safe reading.
        "" | "QUEUED" => "QUEUED".to_string(),
        "LISTENED" => "LISTENED".to_string(),
        other => return Err(format!("unknown status: {other}")),
    };

    if let Some(v) = r.rating {
        if !(0.5..=5.0).contains(&v) || (v * 2.0 - (v * 2.0).round()).abs() > 1e-9 {
            return Err(format!("rating must be 0.5 to 5.0 in half steps, not {v}"));
        }
    }

    for (field, value) in [("dateListened", &mut r.date_listened), ("addedAt", &mut r.added_at)] {
        if let Some(raw) = value.as_deref() {
            *value = normalize_timestamp(raw).map_err(|e| format!("{field}: {e}"))?;
        }
    }

    r.country = r.country.take().and_then(blank_to_none);
    r.album_art_url = r.album_art_url.take().and_then(blank_to_none);
    r.musicbrainz_id = r.musicbrainz_id.take().and_then(blank_to_none);
    r.notes = r.notes.take().and_then(blank_to_none);
    r.discovery_link = r.discovery_link.take().and_then(blank_to_none);

    // Sorted and deduplicated case-insensitively, matching what the catalog would hold:
    // `set_genres` finds existing genres with COLLATE NOCASE, so "Jazz" and "jazz" in one
    // file are one genre by the time they land.
    r.genres = r
        .genres
        .iter()
        .map(|g| g.trim().to_string())
        .filter(|g| !g.is_empty())
        .collect();
    r.genres.sort_by_key(|g| g.to_lowercase());
    r.genres.dedup_by_key(|g| g.to_lowercase());

    r.streaming_links = r
        .streaming_links
        .into_iter()
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .filter(|(k, v)| !k.is_empty() && !v.is_empty())
        .collect();

    Ok(r)
}

/// Accepts what the app writes and the shorthand a person types.
///
/// Everything downstream sorts and groups these as strings, so a bare date has to become a
/// full timestamp here rather than sitting in the database in a second shape.
fn normalize_timestamp(raw: &str) -> Result<Option<String>, String> {
    let v = raw.trim();
    if v.is_empty() {
        return Ok(None);
    }
    if v.len() == 10 && is_plain_date(v) {
        return Ok(Some(format!("{v}T00:00:00Z")));
    }
    if v.contains('T') && v.len() >= 19 {
        return Ok(Some(v.to_string()));
    }
    Err(format!("{v} is not a date or an RFC3339 timestamp"))
}

fn is_plain_date(v: &str) -> bool {
    let b = v.as_bytes();
    b.iter().enumerate().all(|(i, c)| match i {
        4 | 7 => *c == b'-',
        _ => c.is_ascii_digit(),
    })
}

fn blank_to_none(v: String) -> Option<String> {
    let t = v.trim();
    (!t.is_empty()).then(|| t.to_string())
}

pub fn status_of(r: &ExportedRelease) -> ReleaseStatus {
    match r.status.as_str() {
        "LISTENED" => ReleaseStatus::Listened,
        _ => ReleaseStatus::Queued,
    }
}
