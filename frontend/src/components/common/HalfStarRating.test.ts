import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import HalfStarRating from './HalfStarRating.vue'

function mountRating(props: { modelValue: number | null; cancel?: boolean; readonly?: boolean }) {
  return mount(HalfStarRating, { props })
}

const emitted = (w: ReturnType<typeof mountRating>) =>
  w.emitted('update:modelValue')?.map((e) => e[0])

describe('the rating from the keyboard', () => {
  it('steps by half a star', async () => {
    const w = mountRating({ modelValue: 3 })
    const stars = w.find('[role="slider"]')
    await stars.trigger('keydown', { key: 'ArrowRight' })
    await stars.trigger('keydown', { key: 'ArrowLeft' })
    expect(emitted(w)).toEqual([3.5, 2.5])
  })

  it('starts from half a star when unrated, and stops at five', async () => {
    const unrated = mountRating({ modelValue: null })
    await unrated.find('[role="slider"]').trigger('keydown', { key: 'ArrowUp' })
    expect(emitted(unrated)).toEqual([0.5])

    const full = mountRating({ modelValue: 5 })
    await full.find('[role="slider"]').trigger('keydown', { key: 'ArrowRight' })
    expect(emitted(full)).toBeUndefined()
  })

  it('clears only where a clear button is offered', async () => {
    const clearable = mountRating({ modelValue: 0.5, cancel: true })
    await clearable.find('[role="slider"]').trigger('keydown', { key: 'ArrowLeft' })
    await clearable.find('[role="slider"]').trigger('keydown', { key: 'Delete' })
    expect(emitted(clearable)).toEqual([null, null])

    const fixed = mountRating({ modelValue: 0.5 })
    await fixed.find('[role="slider"]').trigger('keydown', { key: 'ArrowLeft' })
    await fixed.find('[role="slider"]').trigger('keydown', { key: 'Backspace' })
    expect(emitted(fixed)).toBeUndefined()
  })

  it('jumps to the ends with Home and End', async () => {
    const w = mountRating({ modelValue: 3 })
    await w.find('[role="slider"]').trigger('keydown', { key: 'Home' })
    await w.find('[role="slider"]').trigger('keydown', { key: 'End' })
    expect(emitted(w)).toEqual([0.5, 5])
  })

  it('ignores other keys, so global shortcuts still reach the window', async () => {
    const w = mountRating({ modelValue: 3 })
    await w.find('[role="slider"]').trigger('keydown', { key: 'g' })
    expect(emitted(w)).toBeUndefined()
  })
})

describe('the rating by click', () => {
  it('sets the half star clicked, and clears it when clicked again', async () => {
    const w = mountRating({ modelValue: 4 })
    await w.findAll('.hsr-half')[5].trigger('click')
    await w.findAll('.hsr-half')[7].trigger('click')
    expect(emitted(w)).toEqual([3, null])
  })

  it('is read-only when asked, and says its value instead', async () => {
    const w = mountRating({ modelValue: 4, readonly: true })
    const stars = w.find('[role="img"]')
    expect(stars.attributes('aria-label')).toBe('4 of 5 stars')
    expect(stars.attributes('tabindex')).toBeUndefined()
    await stars.trigger('keydown', { key: 'ArrowRight' })
    await w.findAll('.hsr-half')[5].trigger('click')
    expect(emitted(w)).toBeUndefined()
  })
})
