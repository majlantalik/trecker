//! Phase 1 in-memory store.
//!
//! This exists so the app is genuinely usable end to end before SQLite lands: adding,
//! editing, filtering and deleting all really work, they just do not survive a restart.
//! Phase 3 replaces this module with `repo/` backed by sqlx. The command signatures in
//! `commands/` must not change when it does.
//!
//! The filter and sort logic here is deliberately written to mirror what
//! `UserReleaseSpecification` does today, so that porting it to SQL later is a
//! translation rather than a redesign.

use crate::domain::*;
use crate::error::{AppError, AppResult};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Store {
    releases: Mutex<Vec<Release>>,
}

/// Columns the client is allowed to sort by.
///
/// In Phase 3 this becomes load-bearing for safety: `sort` is a client-supplied string,
/// and JPA used to make that safe. Raw SQL will not, so anything outside this list must
/// be rejected rather than interpolated. Keeping the whitelist here now means the
/// contract is already correct when the SQL arrives.
const SORTABLE: &[&str] = &[
    "createdAt",
    "artist",
    "title",
    "releaseYear",
    "rating",
    "dateListened",
];

impl Store {
    pub fn new() -> Self {
        Self {
            releases: Mutex::new(seed()),
        }
    }

    fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, Vec<Release>>> {
        self.releases
            .lock()
            .map_err(|_| AppError::Internal("store lock poisoned".into()))
    }

    pub fn list(&self, params: &ReleaseFilterParams) -> AppResult<PageResponse<Release>> {
        let all = self.lock()?;

        let mut matched: Vec<Release> = all
            .iter()
            .filter(|r| matches(r, params))
            .cloned()
            .collect();

        let sort = params.sort.as_deref().unwrap_or("createdAt");
        if !SORTABLE.contains(&sort) {
            return Err(AppError::Invalid(format!("cannot sort by '{sort}'")));
        }
        let descending = !params
            .direction
            .as_deref()
            .is_some_and(|d| d.eq_ignore_ascii_case("ASC"));
        sort_by(&mut matched, sort, descending);

        let size = params.size.unwrap_or(20).max(1);
        let page = params.page.unwrap_or(0);
        let total_elements = matched.len() as u64;
        let total_pages = ((total_elements as f64) / (size as f64)).ceil() as u32;

        let start = (page * size) as usize;
        let content = matched
            .into_iter()
            .skip(start)
            .take(size as usize)
            .collect();

        Ok(PageResponse {
            content,
            total_elements,
            total_pages,
            number: page,
            size,
            first: page == 0,
            last: page + 1 >= total_pages.max(1),
        })
    }

    pub fn get(&self, id: &str) -> AppResult<Release> {
        self.lock()?
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or(AppError::NotFound("release"))
    }

    pub fn random_queued(&self) -> AppResult<Release> {
        let all = self.lock()?;
        let queued: Vec<&Release> = all
            .iter()
            .filter(|r| r.status == ReleaseStatus::Queued)
            .collect();
        if queued.is_empty() {
            return Err(AppError::NotFound("queued release"));
        }
        // Good enough for a fixture store; Phase 3 uses ORDER BY RANDOM() LIMIT 1.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as usize)
            .unwrap_or(0);
        Ok(queued[nanos % queued.len()].clone())
    }

    pub fn create(&self, req: ReleaseRequest) -> AppResult<Release> {
        if req.artist.trim().is_empty() || req.title.trim().is_empty() {
            return Err(AppError::Invalid("artist and title are required".into()));
        }
        let release = Release {
            id: new_id(),
            artist: req.artist,
            title: req.title,
            release_year: req.release_year,
            album_art_url: req.album_art_url,
            status: ReleaseStatus::Queued,
            discovery_link: req.discovery_link,
            streaming_links: req.streaming_links.unwrap_or_default(),
            country: req.country,
            rating: None,
            did_not_finish: false,
            date_listened: None,
            notes: None,
            created_at: now(),
            genres: req.genres.unwrap_or_default(),
        };
        self.lock()?.insert(0, release.clone());
        Ok(release)
    }

    pub fn update(&self, id: &str, req: ReleaseUpdateRequest) -> AppResult<Release> {
        let mut all = self.lock()?;
        let r = all
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(AppError::NotFound("release"))?;

        if let Some(v) = req.artist {
            r.artist = v;
        }
        if let Some(v) = req.title {
            r.title = v;
        }
        if let Some(v) = req.status {
            r.status = v;
            // Mirrors the backend: marking listened stamps the date if none was given.
            if v == ReleaseStatus::Listened && r.date_listened.is_none() {
                r.date_listened = Some(now());
            }
        }
        if req.release_year.is_some() {
            r.release_year = req.release_year;
        }
        if req.album_art_url.is_some() {
            r.album_art_url = req.album_art_url;
        }
        if req.discovery_link.is_some() {
            r.discovery_link = req.discovery_link;
        }
        if let Some(v) = req.streaming_links {
            r.streaming_links = v;
        }
        if req.country.is_some() {
            r.country = req.country;
        }
        if req.rating.is_some() {
            r.rating = req.rating;
        }
        if let Some(v) = req.did_not_finish {
            r.did_not_finish = v;
        }
        if req.date_listened.is_some() {
            r.date_listened = req.date_listened;
        }
        if req.notes.is_some() {
            r.notes = req.notes;
        }
        if let Some(v) = req.genres {
            r.genres = v;
        }
        Ok(r.clone())
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let mut all = self.lock()?;
        let before = all.len();
        all.retain(|r| r.id != id);
        if all.len() == before {
            return Err(AppError::NotFound("release"));
        }
        Ok(())
    }

    pub fn genres(&self) -> AppResult<Vec<String>> {
        let all = self.lock()?;
        let mut g: Vec<String> = all.iter().flat_map(|r| r.genres.clone()).collect();
        g.sort();
        g.dedup();
        Ok(g)
    }

    pub fn listened(&self) -> AppResult<Vec<Release>> {
        Ok(self
            .lock()?
            .iter()
            .filter(|r| r.status == ReleaseStatus::Listened)
            .cloned()
            .collect())
    }
}

fn matches(r: &Release, p: &ReleaseFilterParams) -> bool {
    if let Some(s) = p.status {
        if r.status != s {
            return false;
        }
    }
    if let Some(min) = p.rating_min {
        if r.rating.unwrap_or(0.0) < min {
            return false;
        }
    }
    if let Some(max) = p.rating_max {
        if r.rating.unwrap_or(0.0) > max {
            return false;
        }
    }
    if let Some(dnf) = p.did_not_finish {
        if r.did_not_finish != dnf {
            return false;
        }
    }
    if let Some(c) = p.country.as_deref().filter(|c| !c.trim().is_empty()) {
        if !r.country.as_deref().is_some_and(|rc| rc.eq_ignore_ascii_case(c)) {
            return false;
        }
    }
    if let Some(y) = p.year {
        if r.release_year != Some(y) {
            return false;
        }
    }
    if let Some(g) = p.genre.as_deref().filter(|g| !g.trim().is_empty()) {
        if !r.genres.iter().any(|rg| rg.eq_ignore_ascii_case(g)) {
            return false;
        }
    }
    if let Some(q) = p.search.as_deref().filter(|q| !q.trim().is_empty()) {
        let q = q.to_lowercase();
        if !r.artist.to_lowercase().contains(&q) && !r.title.to_lowercase().contains(&q) {
            return false;
        }
    }
    true
}

fn sort_by(v: &mut [Release], field: &str, descending: bool) {
    v.sort_by(|a, b| {
        let o = match field {
            "artist" => a.artist.to_lowercase().cmp(&b.artist.to_lowercase()),
            "title" => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            "releaseYear" => a.release_year.cmp(&b.release_year),
            "rating" => a
                .rating
                .partial_cmp(&b.rating)
                .unwrap_or(std::cmp::Ordering::Equal),
            "dateListened" => a.date_listened.cmp(&b.date_listened),
            _ => a.created_at.cmp(&b.created_at),
        };
        if descending {
            o.reverse()
        } else {
            o
        }
    });
}

fn now() -> String {
    // Phase 3 swaps this for chrono. A fixture store does not warrant the dependency.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", iso_from_unix(secs))
}

fn new_id() -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{n:032x}")
}

/// Minimal civil-date conversion so fixtures carry plausible ISO timestamps.
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

fn fixture(
    id: &str,
    artist: &str,
    title: &str,
    year: i32,
    country: &str,
    genres: &[&str],
    status: ReleaseStatus,
    rating: Option<f64>,
    listened: Option<&str>,
    created: &str,
) -> Release {
    Release {
        id: id.into(),
        artist: artist.into(),
        title: title.into(),
        release_year: Some(year),
        album_art_url: None,
        status,
        discovery_link: None,
        streaming_links: HashMap::new(),
        country: Some(country.into()),
        rating,
        did_not_finish: false,
        date_listened: listened.map(String::from),
        notes: None,
        created_at: created.into(),
        genres: genres.iter().map(|s| s.to_string()).collect(),
    }
}

fn seed() -> Vec<Release> {
    use ReleaseStatus::*;
    vec![
        fixture("f1", "Godspeed You! Black Emperor", "Lift Your Skinny Fists Like Antennas to Heaven", 2000, "CA", &["post-rock", "experimental"], Listened, Some(4.5), Some("2026-09-01T19:30:00Z"), "2026-08-20T10:00:00Z"),
        fixture("f2", "Ryo Fukui", "Scenery", 1976, "JP", &["jazz"], Queued, None, None, "2026-09-05T08:15:00Z"),
        fixture("f3", "Mount Kimbie", "The Sunset Violent", 2024, "GB", &["electronic", "post-punk"], Listened, Some(3.5), Some("2026-09-09T21:00:00Z"), "2026-09-02T12:00:00Z"),
        fixture("f4", "Duster", "Stratosphere", 1998, "US", &["slowcore", "space rock"], Queued, None, None, "2026-09-06T17:45:00Z"),
        fixture("f5", "Alice Coltrane", "Journey in Satchidananda", 1971, "US", &["jazz", "spiritual jazz"], Listened, Some(5.0), Some("2026-08-28T20:10:00Z"), "2026-08-15T09:00:00Z"),
        fixture("f6", "Boards of Canada", "Music Has the Right to Children", 1998, "GB", &["electronic", "ambient"], Listened, Some(4.5), Some("2026-07-14T22:05:00Z"), "2026-07-01T11:30:00Z"),
        fixture("f7", "Slint", "Spiderland", 1991, "US", &["post-rock", "math rock"], Queued, None, None, "2026-09-08T14:20:00Z"),
        fixture("f8", "Tirzah", "Devotion", 2018, "GB", &["electronic", "r&b"], Listened, Some(4.0), Some("2026-06-03T18:40:00Z"), "2026-05-28T16:00:00Z"),
        fixture("f9", "Talk Talk", "Laughing Stock", 1991, "GB", &["post-rock", "art rock"], Listened, Some(5.0), Some("2026-09-10T19:55:00Z"), "2026-09-03T13:10:00Z"),
        fixture("f10", "Nala Sinephro", "Space 1.8", 2021, "BE", &["jazz", "ambient"], Queued, None, None, "2026-09-07T10:05:00Z"),
    ]
}
