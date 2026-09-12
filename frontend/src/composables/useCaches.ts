import { ref } from 'vue'
import { cacheApi } from '@/api/cache'
import { formatBytes } from '@/utils/bytes'
import type { CoverCacheInfo } from '@/types'

/**
 * The Info view's cache controls: what the cover cache holds, and clearing it or the
 * webview's own data. Both caches are derived, so clearing either loses nothing.
 */

function covers(count: number): string {
  return count === 1 ? '1 cover' : `${count} covers`
}

export function describeCovers(info: CoverCacheInfo): string {
  return info.count === 0 ? 'none downloaded yet' : `${covers(info.count)}, ${formatBytes(info.sizeBytes)}`
}

export function describeCleared(removed: CoverCacheInfo): string {
  return removed.count === 0
    ? 'There were no covers to remove'
    : `Removed ${covers(removed.count)}, ${formatBytes(removed.sizeBytes)}`
}

export function useCaches() {
  const info = ref<CoverCacheInfo | null>(null)
  const busy = ref(false)
  const message = ref('')
  const error = ref('')

  async function run(work: () => Promise<void>) {
    message.value = ''
    error.value = ''
    busy.value = true
    try {
      await work()
    } catch (e: any) {
      error.value = e?.message ?? String(e)
    } finally {
      busy.value = false
    }
  }

  async function load() {
    await run(async () => {
      info.value = await cacheApi.covers()
    })
  }

  async function clearCovers() {
    await run(async () => {
      message.value = describeCleared(await cacheApi.clearCovers())
      info.value = await cacheApi.covers()
    })
  }

  async function clearWebview() {
    await run(async () => {
      await cacheApi.clearWebview()
      message.value = 'Web view data cleared'
    })
  }

  return { info, busy, message, error, load, clearCovers, clearWebview }
}
