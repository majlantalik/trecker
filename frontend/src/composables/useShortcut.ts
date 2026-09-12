import { onMounted, onUnmounted } from 'vue'

/**
 * A single keyboard shortcut, described the way you would say it: `"ctrl+k"`, `"?"`,
 * `"Escape"`. `ctrl` also matches Command on macOS, since every app there uses Command
 * for what Linux and Windows spell Ctrl.
 *
 * By default a shortcut does not fire while the user is typing. That guard is what makes
 * single-letter shortcuts safe to use at all; pass `whileTyping` for the few that should
 * work anywhere, such as Escape.
 */
export function useShortcut(
  combo: string,
  handler: (event: KeyboardEvent) => void,
  options: { whileTyping?: boolean } = {}
) {
  const parts = combo.toLowerCase().split('+')
  const wantsCtrl = parts.includes('ctrl')
  const wantsShift = parts.includes('shift')
  const wantsAlt = parts.includes('alt')
  const key = parts[parts.length - 1]

  function onKeydown(event: KeyboardEvent) {
    if (!options.whileTyping && isTyping(event.target)) return

    // Ctrl and Command are the same shortcut in different clothes.
    const modifier = event.ctrlKey || event.metaKey
    if (wantsCtrl !== modifier) return
    if (wantsAlt !== event.altKey) return
    // Shift is only checked when asked for: "?" is already Shift+/ on most layouts.
    if (wantsShift && !event.shiftKey) return

    if (event.key.toLowerCase() !== key) return

    event.preventDefault()
    handler(event)
  }

  onMounted(() => window.addEventListener('keydown', onKeydown))
  onUnmounted(() => window.removeEventListener('keydown', onKeydown))
}

/** Whether the event landed in something the user is typing into. */
export function isTyping(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null
  if (!el) return false
  const tag = el.tagName
  return (
    tag === 'INPUT' ||
    tag === 'TEXTAREA' ||
    tag === 'SELECT' ||
    el.isContentEditable === true
  )
}
