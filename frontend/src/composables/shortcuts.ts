/**
 * The catalogue of keyboard shortcuts.
 *
 * Single source of truth on purpose: the help dialog renders from this list, so it cannot
 * quietly describe a shortcut the app no longer has. Adding one means adding it here and
 * wiring it, and forgetting the second half is visible immediately.
 */

export interface Shortcut {
  /** Rendered as separate keys. `Mod` becomes Ctrl or Command depending on the platform. */
  keys: string[]
  description: string
  group: string
}

export const SHORTCUTS: Shortcut[] = [
  {
    keys: ['Shift', 'Shift'],
    description: 'Open quick add',
    group: 'Adding'
  },
  {
    keys: ['Mod', 'K'],
    description: 'Open quick add',
    group: 'Adding'
  },
  {
    keys: ['Enter'],
    description: 'Look up what you typed',
    group: 'Adding'
  },
  {
    keys: ['?'],
    description: 'Show this list',
    group: 'General'
  },
  {
    keys: ['Esc'],
    description: 'Close a dialog',
    group: 'General'
  }
]

/** Groups, in the order they should be shown. */
export const SHORTCUT_GROUPS = ['Adding', 'General']

const isMac =
  typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.userAgent)

/** `Mod` is Command on macOS and Ctrl everywhere else. */
export function displayKey(key: string): string {
  if (key !== 'Mod') return key
  return isMac ? '⌘' : 'Ctrl'
}
