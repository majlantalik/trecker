import { ref } from 'vue'

// The quick add palette opens from Shift Shift and Ctrl+K, from a desktop shortcut running
// `trecker --quick-add`, and from the tray menu. Module-level state gives all of them the
// same dialog.
const open = ref(false)

export function useQuickAddPalette() {
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
