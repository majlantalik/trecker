import { onMounted, onUnmounted } from 'vue'

/**
 * Fires when a modifier key is pressed twice in quick succession, the way JetBrains IDEs
 * open Search Everywhere on double Shift.
 *
 * Three things make this fiddly, and all three are guarded here:
 *
 *  - **Holding the key repeats it.** Auto-repeat sends a stream of keydown events, which
 *    would read as a double tap after a few milliseconds. `event.repeat` filters them.
 *  - **Shift is normally a modifier.** Typing `AB` is Shift, A, Shift, B. Any other key in
 *    between has to cancel the pending tap, or capital letters would open the palette.
 *  - **So are chords.** Ctrl+Shift+P starts with a Shift press. A tap only counts when no
 *    other modifier is held.
 *
 * Deliberately still fires while a text field has focus, matching the IDE behaviour: the
 * whole point is to reach it without leaving the keyboard.
 */
export function useDoubleTap(
  key: 'Shift' | 'Control' | 'Alt' | 'Meta',
  handler: () => void,
  windowMs = 400
) {
  let lastTap = 0

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== key) {
      // Anything else cancels a tap that was waiting for its partner.
      lastTap = 0
      return
    }
    if (event.repeat) return
    if (event.ctrlKey && key !== 'Control') return
    if (event.altKey && key !== 'Alt') return
    if (event.metaKey && key !== 'Meta') return

    const now = performance.now()
    if (lastTap && now - lastTap <= windowMs) {
      lastTap = 0
      handler()
    } else {
      lastTap = now
    }
  }

  // Losing focus mid-tap would otherwise leave a tap armed indefinitely, so that coming
  // back to the window and pressing the key once would fire.
  function reset() {
    lastTap = 0
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeydown)
    window.addEventListener('blur', reset)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeydown)
    window.removeEventListener('blur', reset)
  })
}
