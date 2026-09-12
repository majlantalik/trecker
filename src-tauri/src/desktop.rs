//! Living on the desktop: quick add from a desktop shortcut, the tray icon, and what
//! closing the window does.
//!
//! **Why a command and not a global shortcut.** On Wayland an app cannot grab a key for the
//! whole desktop. The sanctioned route is the desktop portal's GlobalShortcuts interface,
//! and COSMIC's portal does not implement it; Tauri's global-shortcut plugin goes through
//! X11 and only sees keys while an X11 window has focus. Every desktop does let a person
//! bind a key to a command, though, so Trecker offers the command: `trecker --quick-add`.
//! A second launch hands its arguments to the running app through the single-instance
//! plugin and exits, and the running app opens the palette.

use crate::error::{AppError, AppResult};
use crate::settings::{CloseAction, SettingsStore};
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, CloseRequestApi, Emitter, Manager, Window};

/// The argument a desktop shortcut passes to open quick add.
pub const QUICK_ADD_FLAG: &str = "--quick-add";

/// Sent to the frontend when a later launch asks for quick add. `frontend/src/api/app.ts`
/// listens for it.
pub const QUICK_ADD_EVENT: &str = "quick-add";

const TRAY_ID: &str = "trecker";

pub fn wants_quick_add<S: AsRef<str>>(args: impl IntoIterator<Item = S>) -> bool {
    args.into_iter().any(|a| a.as_ref() == QUICK_ADD_FLAG)
}

/// What this launch was asked to do, held until the frontend has loaded and asks.
///
/// An event would be lost: the app emits during startup, before the page exists to hear
/// it. So the first launch stores the action and the frontend takes it once on mount.
/// Later launches, arriving while the page is already listening, use the event instead.
#[derive(Default)]
pub struct LaunchAction(Mutex<Option<&'static str>>);

impl LaunchAction {
    pub fn from_args<S: AsRef<str>>(args: impl IntoIterator<Item = S>) -> Self {
        Self(Mutex::new(wants_quick_add(args).then_some(QUICK_ADD_EVENT)))
    }

    /// The pending action, once. A reload of the page must not reopen the palette.
    pub fn take(&self) -> Option<String> {
        self.0.lock().ok()?.take().map(String::from)
    }
}

/// A second launch while the app is running: bring the window back, and open quick add if
/// that is what the launch was for.
pub fn on_second_instance(app: &AppHandle, args: Vec<String>) {
    show_main_window(app);
    if wants_quick_add(&args) {
        let _ = app.emit(QUICK_ADD_EVENT, ());
    }
}

/// Unminimises, shows and focuses the main window.
///
/// On Wayland the compositor decides whether focus is granted, and may only mark the
/// window as wanting attention. That is the compositor's call to make, not something to
/// work around.
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Puts the tray icon in the state the close action needs: present for `Tray`, gone for
/// `Quit`.
pub fn apply_close_action(app: &AppHandle, action: CloseAction) -> AppResult<()> {
    match action {
        CloseAction::Tray => ensure_tray(app)
            .map_err(|e| AppError::Internal(format!("could not create the tray icon: {e}"))),
        CloseAction::Quit => {
            app.remove_tray_by_id(TRAY_ID);
            Ok(())
        }
    }
}

fn ensure_tray(app: &AppHandle) -> tauri::Result<()> {
    if app.tray_by_id(TRAY_ID).is_some() {
        return Ok(());
    }

    let open = MenuItemBuilder::with_id("open", "Open Trecker").build(app)?;
    let quick_add = MenuItemBuilder::with_id("quick-add", "Quick add…").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Trecker").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&open, &quick_add])
        .separator()
        .item(&quit)
        .build()?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Trecker")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "quick-add" => {
                show_main_window(app);
                let _ = app.emit(QUICK_ADD_EVENT, ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        // Linux trays show the menu on click and never report the click itself, so this
        // only does anything on Windows and macOS.
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Closing the window: hide it when the settings say to keep running.
///
/// Only when a tray icon actually exists. If creating it failed, for instance on a desktop
/// with no tray, hiding the window would leave an app running with no visible way back to
/// it but launching it again. Quitting is the honest fallback.
pub fn on_close_requested(window: &Window, api: &CloseRequestApi) {
    let app = window.app_handle();
    let keep_running = app.state::<SettingsStore>().get().close_action == CloseAction::Tray;
    if keep_running && app.tray_by_id(TRAY_ID).is_some() {
        api.prevent_close();
        let _ = window.hide();
    }
}

/// The command to bind to a desktop shortcut, written the way this copy of the app was
/// started.
///
/// An AppImage runs from a temporary mount, so its own path is what a shortcut needs, and
/// the AppImage runtime puts it in `APPIMAGE`. An installed package's binary is on `PATH`
/// and can be named bare. Anything else, such as a development build, gets its full path.
pub fn quick_add_command(exe: &Path, appimage: Option<&OsStr>, path_var: Option<&OsStr>) -> String {
    if let Some(image) = appimage.filter(|a| !a.is_empty()) {
        return format!("{} {QUICK_ADD_FLAG}", shell_quote(&image.to_string_lossy()));
    }

    let on_path = exe.file_name().and_then(|name| {
        let exe = std::fs::canonicalize(exe).ok()?;
        std::env::split_paths(path_var?)
            .map(|dir| dir.join(name))
            .any(|candidate| std::fs::canonicalize(candidate).is_ok_and(|c| c == exe))
            .then(|| name.to_string_lossy().into_owned())
    });

    match on_path {
        Some(name) => format!("{} {QUICK_ADD_FLAG}", shell_quote(&name)),
        None => format!("{} {QUICK_ADD_FLAG}", shell_quote(&exe.to_string_lossy())),
    }
}

/// Quotes a path for a shell only when it needs it, so the common case stays readable.
fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._-+:@%".contains(c))
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn finds_the_flag_among_other_arguments() {
        assert!(wants_quick_add(["trecker", "--quick-add"]));
        assert!(wants_quick_add(["/opt/Trecker.AppImage", "--verbose", "--quick-add"]));
        assert!(!wants_quick_add(["trecker"]));
        assert!(!wants_quick_add(["trecker", "--quick-adder"]), "exact match only");
    }

    #[test]
    fn a_launch_action_is_taken_once() {
        // A reload of the page asks again, and must not open the palette a second time.
        let action = LaunchAction::from_args(["trecker", "--quick-add"]);
        assert_eq!(action.take().as_deref(), Some("quick-add"));
        assert_eq!(action.take(), None);
        assert_eq!(LaunchAction::from_args(["trecker"]).take(), None);
    }

    #[test]
    fn an_appimage_is_named_by_its_own_path() {
        let cmd = quick_add_command(
            Path::new("/tmp/.mount_TreckerXyz/usr/bin/trecker"),
            Some(OsStr::new("/home/me/Apps/Trecker_0.1.0_amd64.AppImage")),
            None,
        );
        assert_eq!(cmd, "/home/me/Apps/Trecker_0.1.0_amd64.AppImage --quick-add");
    }

    #[test]
    fn a_binary_on_path_is_named_bare() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("trecker");
        std::fs::write(&exe, b"").unwrap();
        let path = std::env::join_paths([Path::new("/nonexistent"), dir.path()]).unwrap();
        assert_eq!(quick_add_command(&exe, None, Some(&path)), "trecker --quick-add");
    }

    #[test]
    fn a_binary_off_path_gets_its_full_path() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("trecker");
        std::fs::write(&exe, b"").unwrap();
        let unrelated: OsString = "/nonexistent".into();
        let cmd = quick_add_command(&exe, None, Some(&unrelated));
        assert_eq!(cmd, format!("{} --quick-add", exe.display()));
    }

    #[test]
    fn a_different_trecker_on_path_is_not_mistaken_for_this_one() {
        // Naming it bare would run whichever copy PATH finds first, not this one.
        let mine = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let exe = mine.path().join("trecker");
        std::fs::write(&exe, b"mine").unwrap();
        std::fs::write(other.path().join("trecker"), b"other").unwrap();
        let path = std::env::join_paths([other.path()]).unwrap();
        assert!(quick_add_command(&exe, None, Some(&path)).starts_with('/'));
    }

    #[test]
    fn quotes_only_paths_that_need_it() {
        assert_eq!(shell_quote("/usr/bin/trecker"), "/usr/bin/trecker");
        assert_eq!(shell_quote("/home/me/My Apps/Trecker.AppImage"), "'/home/me/My Apps/Trecker.AppImage'");
        assert_eq!(shell_quote("/home/me/it's/trecker"), r"'/home/me/it'\''s/trecker'");
    }
}
