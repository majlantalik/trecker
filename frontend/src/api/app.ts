import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/**
 * How the desktop reaches into the app: a shortcut running `trecker --quick-add`, or the
 * tray menu's Quick add. See `src-tauri/src/desktop.rs`.
 */
export const appApi = {
  /** What this launch was started to do, such as `quick-add`. Handed over once. */
  async takeLaunchAction(): Promise<string | null> {
    return invoke<string | null>('app_take_launch_action')
  },

  /** The command to bind to a desktop shortcut, written for this copy of the app. */
  async quickAddCommand(): Promise<string> {
    return invoke<string>('app_quick_add_command')
  },

  /** Calls back whenever a later launch or the tray asks for quick add. */
  async onQuickAdd(callback: () => void): Promise<UnlistenFn> {
    // The name must match `QUICK_ADD_EVENT` in src-tauri/src/desktop.rs.
    return listen('quick-add', () => callback())
  }
}
