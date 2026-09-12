import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import type { CoverCacheInfo } from '@/types'

/**
 * The address to render a cover from.
 *
 * Never put a stored `albumArtUrl` straight into an image tag. This routes it through the
 * `cover:` protocol, which serves the copy on disk and downloads it the first time. The
 * content security policy blocks remote images, so a cover that skips this does not load.
 */
export function coverSrc(url: string | null | undefined): string | undefined {
  return url ? convertFileSrc(url, 'cover') : undefined
}

export const cacheApi = {
  async covers(): Promise<CoverCacheInfo> {
    return invoke<CoverCacheInfo>('cache_covers_info')
  },

  /** Resolves to what was removed. */
  async clearCovers(): Promise<CoverCacheInfo> {
    return invoke<CoverCacheInfo>('cache_covers_clear')
  },

  async clearWebview(): Promise<void> {
    await invoke<void>('cache_webview_clear')
  }
}
