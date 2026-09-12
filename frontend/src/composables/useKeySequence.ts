import { onMounted, onUnmounted, ref } from 'vue'
import { isTyping } from './useShortcut'

/**
 * Two-key sequences, the way Gmail and GitHub do navigation: press `g`, then `l`.
 *
 * A prefix key is better than a modifier here because the second key is a mnemonic you
 * can pick by first letter, and because single letters on their own would collide with
 * everything.
 *
 * The pending prefix is exposed so the interface can show that it is waiting, which is
 * the difference between a discoverable sequence and a key that appears to do nothing.
 */
export function useKeySequence(
  prefix: string,
  handlers: Record<string, () => void>,
  timeoutMs = 1500
) {
  const pending = ref(false)
  let timer: ReturnType<typeof setTimeout> | null = null

  function cancel() {
    pending.value = false
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (isTyping(event.target)) return
    // A sequence is bare letters. Any modifier means the user meant something else.
    if (event.ctrlKey || event.metaKey || event.altKey) {
      cancel()
      return
    }

    const key = event.key.toLowerCase()

    if (pending.value) {
      const handler = handlers[key]
      cancel()
      if (handler) {
        event.preventDefault()
        handler()
      }
      // An unmapped second key just ends the sequence rather than doing anything.
      return
    }

    if (key === prefix) {
      event.preventDefault()
      pending.value = true
      // Without an expiry, a stray `g` would silently swallow the next keystroke minutes
      // later, which reads as the app randomly ignoring a key.
      timer = setTimeout(cancel, timeoutMs)
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeydown)
    window.addEventListener('blur', cancel)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeydown)
    window.removeEventListener('blur', cancel)
    cancel()
  })

  return { pending }
}
