import { ref } from 'vue'

// The keyboard shortcuts dialog is opened from two places: the "?" key and the button in
// the sidebar. Module-level state gives both the same dialog without a store for one flag.
const open = ref(false)

export function useShortcutsHelp() {
  return {
    open,
    show: () => {
      open.value = true
    },
    toggle: () => {
      open.value = !open.value
    }
  }
}
