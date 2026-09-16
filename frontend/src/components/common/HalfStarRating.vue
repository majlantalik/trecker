<template>
  <div class="hsr-root" :class="{ 'hsr-readonly': readonly }" @mouseleave="hoverValue = null">
    <button
      v-if="cancel && !readonly"
      class="hsr-cancel"
      type="button"
      :class="{ 'hsr-cancel-active': modelValue !== null }"
      @click="emit('update:modelValue', null)"
      aria-label="Clear rating"
    >
      <i class="pi pi-ban" />
    </button>

    <!-- A slider, so the rating can be set from the keyboard as well as by clicking a half
         star. The click is handled here rather than on each half, which carries its value. -->
    <div
      class="hsr-stars"
      :role="readonly ? 'img' : 'slider'"
      :tabindex="readonly ? undefined : 0"
      :aria-labelledby="ariaLabelledby"
      :aria-label="accessibleName"
      :aria-valuemin="readonly ? undefined : 0"
      :aria-valuemax="readonly ? undefined : 5"
      :aria-valuenow="readonly ? undefined : (modelValue ?? 0)"
      :aria-valuetext="readonly ? undefined : valueText"
      @click="onClick"
      @keydown="onKeydown"
    >
      <div v-for="n in 5" :key="n" class="hsr-star-wrap">
        <span class="hsr-half hsr-left" :data-value="n - 0.5"
          @mousemove="!readonly && (hoverValue = n - 0.5)" />
        <span class="hsr-half hsr-right" :data-value="n"
          @mousemove="!readonly && (hoverValue = n)" />
        <i class="hsr-bg pi pi-star" />
        <i class="hsr-fg pi pi-star-fill" :style="fgStyle(n)" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const props = withDefaults(defineProps<{
  modelValue: number | null
  readonly?: boolean
  cancel?: boolean
  /** The id of a visible label, since a `<label for>` cannot point at a slider. */
  ariaLabelledby?: string
}>(), { readonly: false, cancel: false, ariaLabelledby: undefined })

const emit = defineEmits<{ 'update:modelValue': [value: number | null] }>()

const hoverValue = ref<number | null>(null)

const valueText = computed(() =>
  props.modelValue === null ? 'Not rated' : `${props.modelValue} of 5 stars`
)

/** A name for when no visible label is given. Read-only stars are named by their value. */
const accessibleName = computed(() => {
  if (props.ariaLabelledby) return undefined
  return props.readonly ? valueText.value : 'Rating'
})

function effective() { return hoverValue.value ?? props.modelValue }

function fgStyle(n: number) {
  const v = effective()
  if (v === null || v < n - 0.5) return { opacity: '0' }
  if (v >= n) return { opacity: '1' }
  return { clipPath: 'inset(0 50% 0 0)', opacity: '1' }
}

function select(value: number) {
  emit('update:modelValue', props.modelValue === value ? null : value)
}

function onClick(event: MouseEvent) {
  if (props.readonly) return
  const half = (event.target as HTMLElement).closest<HTMLElement>('[data-value]')
  if (half) select(Number(half.dataset.value))
}

/**
 * Arrows step by half a star, Home and End jump to the ends. Stepping below half a star,
 * or Delete and Backspace, clears the rating where a clear button is offered.
 */
function onKeydown(event: KeyboardEvent) {
  if (props.readonly) return
  const current = props.modelValue ?? 0
  let next: number | null
  switch (event.key) {
    case 'ArrowRight':
    case 'ArrowUp':
      next = Math.min(5, current + 0.5)
      break
    case 'ArrowLeft':
    case 'ArrowDown':
      next = current - 0.5
      break
    case 'Home':
      next = 0.5
      break
    case 'End':
      next = 5
      break
    case 'Delete':
    case 'Backspace':
      next = null
      break
    default:
      return
  }
  event.preventDefault()
  if (next !== null && next < 0.5) next = props.cancel ? null : 0.5
  if (next === null && !props.cancel) return
  if (next !== props.modelValue) emit('update:modelValue', next)
}
</script>

<style scoped>
.hsr-root { display: inline-flex; align-items: center; gap: 2px; line-height: 1; }

.hsr-cancel {
  background: none; border: none; padding: 0; margin-right: 4px; cursor: pointer;
  color: var(--p-rating-icon-color, var(--tk-text-muted)); font-size: 1rem;
  opacity: 0.7; transition: opacity 0.15s, color 0.15s; display: inline-flex;
}
.hsr-cancel:hover, .hsr-cancel-active { opacity: 1; color: var(--p-rating-icon-active-color, var(--tk-accent)); }

.hsr-stars { display: inline-flex; align-items: center; gap: 2px; border-radius: 4px; }
.hsr-stars:focus-visible { outline: 2px solid var(--tk-accent); outline-offset: 2px; }

.hsr-star-wrap { position: relative; width: 1.25rem; height: 1.25rem; display: inline-flex; align-items: center; justify-content: center; }

.hsr-half { position: absolute; top: 0; bottom: 0; width: 50%; z-index: 2; cursor: pointer; }
.hsr-left { left: 0; }
.hsr-right { right: 0; }

.hsr-bg { position: absolute; font-size: 1.25rem; color: var(--p-rating-icon-color, var(--tk-text-muted)); opacity: 0.7; pointer-events: none; z-index: 0; }
.hsr-fg { position: absolute; font-size: 1.25rem; color: var(--p-rating-icon-active-color, var(--tk-accent)); pointer-events: none; z-index: 1; transition: clip-path 0.1s, opacity 0.1s; }

.hsr-readonly .hsr-half { cursor: default; pointer-events: none; }
</style>
