import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import type { ReleaseFilterParams } from '@/types'

vi.mock('@/api/genres', () => ({
  genresApi: { getAll: vi.fn().mockResolvedValue([]) },
  countriesApi: { getAll: vi.fn().mockResolvedValue([]) }
}))

const { default: LibraryFilters } = await import('./LibraryFilters.vue')

async function mountFilters() {
  const wrapper = mount(LibraryFilters, {
    props: { initialStatus: 'LISTENED' },
    global: { stubs: { Select: true, HalfStarRating: true, Checkbox: true } }
  })
  await flushPromises()
  return wrapper
}

const yearInput = (w: Awaited<ReturnType<typeof mountFilters>>) => w.find('input[aria-label="Year"]')
const lastYear = (w: Awaited<ReturnType<typeof mountFilters>>) =>
  (w.emitted('update:filters')?.at(-1)?.[0] as ReleaseFilterParams | undefined)?.year

beforeEach(() => vi.useFakeTimers())

describe('the year filter', () => {
  it('filters by the year as typed, not the one before it', async () => {
    // It used to read InputNumber's model, which only updates on blur, so "2025" did nothing
    // and deleting a digit then applied 2025.
    const w = await mountFilters()
    await yearInput(w).setValue('2025')
    vi.advanceTimersByTime(300)
    expect(lastYear(w)).toBe(2025)
  })

  it('waits for four digits, and clears when emptied', async () => {
    const w = await mountFilters()
    await yearInput(w).setValue('202')
    vi.advanceTimersByTime(300)
    expect(w.emitted('update:filters')).toBeUndefined()

    await yearInput(w).setValue('2010')
    vi.advanceTimersByTime(300)
    expect(lastYear(w)).toBe(2010)

    await yearInput(w).setValue('')
    vi.advanceTimersByTime(300)
    expect(lastYear(w)).toBeUndefined()
    expect(w.emitted('update:filters')).toHaveLength(2)
  })

  it('applies at once on Enter', async () => {
    const w = await mountFilters()
    await yearInput(w).setValue('2005')
    await yearInput(w).trigger('keydown', { key: 'Enter' })
    expect(lastYear(w)).toBe(2005)
  })

  it('drops a letter from the field even when the digits do not change', async () => {
    const w = await mountFilters()
    await yearInput(w).setValue('20')
    await yearInput(w).setValue('20x')
    expect((yearInput(w).element as HTMLInputElement).value).toBe('20')
  })

  it('shows no separators and takes four digits at most', async () => {
    const w = await mountFilters()
    await yearInput(w).setValue('2,0256')
    expect((yearInput(w).element as HTMLInputElement).value).toBe('2025')
    expect(yearInput(w).attributes('maxlength')).toBe('4')
  })
})
