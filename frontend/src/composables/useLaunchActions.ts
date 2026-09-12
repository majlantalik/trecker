import { onMounted, onUnmounted } from 'vue'
import { appApi } from '@/api/app'
import { useQuickAddPalette } from '@/composables/useQuickAddPalette'

/**
 * Opens quick add when the desktop asks for it.
 *
 * Two routes, because the request can arrive before the page exists. A launch that starts
 * the app is stored in Rust and taken here on mount. A launch while the app is already
 * running, or the tray's Quick add, arrives as an event.
 */
export function useLaunchActions() {
  const palette = useQuickAddPalette()
  let unlisten: (() => void) | null = null
  let unmounted = false

  onMounted(async () => {
    try {
      const stop = await appApi.onQuickAdd(palette.show)
      // Unmounted while the listener was being registered: remove it at once.
      if (unmounted) stop()
      else unlisten = stop

      if ((await appApi.takeLaunchAction()) === 'quick-add') palette.show()
    } catch {
      // Outside Tauri, such as in a plain browser, there is no desktop to hear from.
    }
  })

  onUnmounted(() => {
    unmounted = true
    unlisten?.()
  })
}
