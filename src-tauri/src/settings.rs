//! Preferences for how the app behaves on this machine.
//!
//! A JSON file in the OS config directory, deliberately not a table in the library
//! database. These describe this device, not the library: an export carries none of them,
//! and sync, if it is ever built, should not either. A second computer can reasonably want
//! the window to quit on close while this one keeps it in the tray.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// What closing the main window does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    /// Quit the app, as any ordinary window does.
    #[default]
    Quit,
    /// Hide the window and keep running with a tray icon, so a desktop shortcut can open
    /// quick add instantly instead of starting the app first.
    Tray,
}

/// Every preference, each with a default, so a file written by an older version, or
/// missing a field, still reads.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub close_action: CloseAction,
}

pub struct SettingsStore {
    path: PathBuf,
    current: Mutex<Settings>,
}

impl SettingsStore {
    /// Reads the file, or starts from the defaults when there is none.
    ///
    /// An unreadable file also means defaults, and is left untouched until the next save.
    /// Refusing to start over a preferences file would be out of all proportion, and
    /// overwriting it on load would destroy whatever a person was trying to fix by hand.
    pub fn load(path: PathBuf) -> Self {
        let current = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Self {
            path,
            current: Mutex::new(current),
        }
    }

    pub fn get(&self) -> Settings {
        self.current.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Writes the new settings, then adopts them. Written to a temporary file and renamed,
    /// so a crash mid-write leaves the previous file rather than half of a new one.
    pub fn replace(&self, next: Settings) -> AppResult<Settings> {
        let err = |e: std::io::Error| {
            AppError::Internal(format!("could not save settings to {}: {e}", self.path.display()))
        };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(err)?;
        }
        let text = serde_json::to_string_pretty(&next)
            .map_err(|e| AppError::Internal(format!("could not write settings: {e}")))?;
        let partial = self.path.with_extension("json.part");
        std::fs::write(&partial, text).map_err(err)?;
        std::fs::rename(&partial, &self.path).map_err(err)?;

        if let Ok(mut current) = self.current.lock() {
            *current = next.clone();
        }
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(dir: &tempfile::TempDir) -> SettingsStore {
        SettingsStore::load(dir.path().join("config").join("settings.json"))
    }

    #[test]
    fn a_missing_file_means_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(store(&dir).get().close_action, CloseAction::Quit);
    }

    #[test]
    fn saved_settings_are_read_back_by_the_next_launch() {
        let dir = tempfile::tempdir().unwrap();
        let first = store(&dir);
        first.replace(Settings { close_action: CloseAction::Tray }).unwrap();
        assert_eq!(first.get().close_action, CloseAction::Tray);
        assert_eq!(store(&dir).get().close_action, CloseAction::Tray, "a fresh load");
    }

    #[test]
    fn the_file_is_readable_json() {
        // It is a file a person may open to fix by hand.
        let dir = tempfile::tempdir().unwrap();
        store(&dir).replace(Settings { close_action: CloseAction::Tray }).unwrap();
        let text = std::fs::read_to_string(dir.path().join("config/settings.json")).unwrap();
        assert!(text.contains(r#""closeAction": "tray""#), "{text}");
    }

    #[test]
    fn a_damaged_file_falls_back_to_defaults_and_is_not_overwritten_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config/settings.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{ not json").unwrap();

        assert_eq!(store(&dir).get(), Settings::default());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
    }

    #[test]
    fn missing_and_unknown_fields_are_tolerated() {
        // Files from older and newer versions of the app.
        assert_eq!(serde_json::from_str::<Settings>("{}").unwrap(), Settings::default());
        let newer: Settings = serde_json::from_str(r#"{"closeAction":"tray","theme":"light"}"#).unwrap();
        assert_eq!(newer.close_action, CloseAction::Tray);
    }

    #[test]
    fn the_words_the_frontend_sends_are_the_ones_rust_reads() {
        assert_eq!(serde_json::from_str::<CloseAction>(r#""quit""#).unwrap(), CloseAction::Quit);
        assert_eq!(serde_json::from_str::<CloseAction>(r#""tray""#).unwrap(), CloseAction::Tray);
    }
}
