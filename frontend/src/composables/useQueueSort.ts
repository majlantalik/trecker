import { computed } from 'vue'
import { useSettings } from '@/composables/useSettings'
import type { QueueSort, ReleaseFilterParams } from '@/types'

export const QUEUE_SORTS: Record<QueueSort, { label: string; icon: string; direction: 'ASC' | 'DESC' }> = {
  newest: { label: 'Newest first', icon: 'pi pi-sort-amount-down', direction: 'DESC' },
  oldest: { label: 'Oldest first', icon: 'pi pi-sort-amount-up-alt', direction: 'ASC' }
}

/**
 * What the queue asks the store for.
 *
 * The store's filters are shared with the Library and merged on every change, so a search or
 * genre picked there would otherwise carry over and hide queued albums. Every filter the
 * Library can set is cleared here explicitly.
 */
export function queueParams(order: QueueSort): ReleaseFilterParams {
  return {
    status: 'QUEUED',
    sort: 'createdAt',
    direction: QUEUE_SORTS[order].direction,
    search: undefined,
    genre: undefined,
    country: undefined,
    year: undefined,
    ratingMin: undefined,
    ratingMax: undefined,
    didNotFinish: undefined,
    unlinked: undefined,
    withoutCover: undefined
  }
}

/**
 * The queue's order, kept in the settings file so it survives a restart. The Settings view
 * edits the same value.
 */
export function useQueueSort() {
  const { settings, saving, error, load, update } = useSettings()

  // Until the settings arrive, or if they cannot be read, the queue is newest first.
  const sort = computed<QueueSort>(() => settings.value?.queueSort ?? 'newest')

  /** Reverses the order and saves it. Resolves to an error message when saving failed. */
  async function toggle(): Promise<string | null> {
    await update({ queueSort: sort.value === 'newest' ? 'oldest' : 'newest' })
    return error.value || null
  }

  return { sort, saving, load, toggle }
}
