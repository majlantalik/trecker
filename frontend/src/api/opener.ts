import { openUrl } from '@tauri-apps/plugin-opener'

/**
 * Opens a URI through the desktop's own registered handler, such as the Spotify app for a
 * `spotify:` link. A plain `<a target="_blank">` only reaches http, https, mailto and tel;
 * anything else needs this command instead, scoped per-scheme in the app's capability file.
 * Kept here for the same reason as the rest of `@/api/`: only this directory knows Tauri
 * exists.
 */
export const openerApi = {
  async openUri(uri: string): Promise<void> {
    await openUrl(uri)
  }
}
