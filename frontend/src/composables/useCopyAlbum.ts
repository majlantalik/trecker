import { useToast } from 'primevue/usetoast'
import type { Release } from '@/types'

/**
 * Copying an album to the clipboard, from its page and from a library row.
 *
 * One composable for both, so the text and what the app says about it are the same
 * wherever the button is.
 */

/**
 * The album as one line: `Artist - Album`.
 *
 * A plain spaced hyphen, not the en dash the confirm dialogs use, because this line is
 * meant to be pasted somewhere else: a search box, a message, a note. It is also the shape
 * `search_query` reads as an artist and a title, so a copied line pasted back into quick
 * add searches the two fields separately.
 */
export function albumText(release: Pick<Release, 'artist' | 'title'>): string {
  return `${release.artist} - ${release.title}`
}

export function useCopyAlbum() {
  const toast = useToast()

  async function copyAlbum(release: Pick<Release, 'artist' | 'title'>) {
    const text = albumText(release)
    try {
      await navigator.clipboard.writeText(text)
      toast.add({ severity: 'success', summary: 'Copied', detail: text, life: 2000 })
    } catch {
      // The clipboard can be refused to a webview, and then nothing would happen and
      // nothing would say why. There is no text field to select here, as the Settings
      // copy button has, so the toast carries the line itself and stays up long enough
      // to read it.
      toast.add({ severity: 'warn', summary: 'Could not reach the clipboard', detail: text, life: 8000 })
    }
  }

  return { copyAlbum }
}
