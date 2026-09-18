import { computed } from 'vue'
import { useToast } from 'primevue/usetoast'
import { openerApi } from '@/api/opener'
import { useSettings } from '@/composables/useSettings'
import { linkToOpen, type LinkChoice } from '@/utils/links'
import type { LinkTarget } from '@/types'

/**
 * Opens an album's streaming link the way the settings ask: in the browser, or in the
 * service's desktop app.
 *
 * Both go through the opener rather than an `<a>`. The button sits inside a clickable
 * library row and uses PrimeVue's Button, which renders a `<button>`, so an `href` on it
 * does nothing.
 */
export function useOpenLink() {
  const { settings, loadOnce } = useSettings()
  const toast = useToast()
  loadOnce()

  // Until the settings arrive, or if they cannot be read, links open in the browser.
  const target = computed<LinkTarget>(() => settings.value?.openLinksIn ?? 'web')

  function choose(links: Record<string, string> | null | undefined): LinkChoice | null {
    return linkToOpen(links, target.value)
  }

  /** Opens the choice, falling back to its web address when the app cannot be reached. */
  async function open(choice: LinkChoice) {
    try {
      await openerApi.openUri(choice.uri)
      return
    } catch (e: any) {
      if (choice.fallback) {
        try {
          await openerApi.openUri(choice.fallback)
          return
        } catch {
          // Reported below, with the first error, which names what was asked for.
        }
      }
      toast.add({
        severity: 'error',
        summary: 'Could not open the link',
        detail: e?.message ?? String(e),
        life: 5000
      })
    }
  }

  return { choose, open }
}
