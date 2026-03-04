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

    <div v-for="n in 5" :key="n" class="hsr-star-wrap">
      <span class="hsr-half hsr-left"
        @mousemove="!readonly && (hoverValue = n - 0.5)"
        @click="!readonly && select(n - 0.5)" />
      <span class="hsr-half hsr-right"
        @mousemove="!readonly && (hoverValue = n)"
        @click="!readonly && select(n)" />
      <i class="hsr-bg pi pi-star" />
      <i class="hsr-fg pi pi-star-fill" :style="fgStyle(n)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const props = withDefaults(defineProps<{
  modelValue: number | null
  readonly?: boolean
  cancel?: boolean
}>(), { readonly: false, cancel: false })

const emit = defineEmits<{ 'update:modelValue': [value: number | null] }>()

const hoverValue = ref<number | null>(null)

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
</script>

<style scoped>
.hsr-root { display: inline-flex; align-items: center; gap: 2px; line-height: 1; }

.hsr-cancel {
  background: none; border: none; padding: 0; margin-right: 4px; cursor: pointer;
  color: var(--p-rating-icon-color, var(--tk-text-muted)); font-size: 1rem;
  opacity: 0.4; transition: opacity 0.15s, color 0.15s; display: inline-flex;
}
.hsr-cancel:hover, .hsr-cancel-active { opacity: 1; color: var(--p-rating-icon-active-color, var(--tk-accent)); }

.hsr-star-wrap { position: relative; width: 1.25rem; height: 1.25rem; display: inline-flex; align-items: center; justify-content: center; }

.hsr-half { position: absolute; top: 0; bottom: 0; width: 50%; z-index: 2; cursor: pointer; }
.hsr-left { left: 0; }
.hsr-right { right: 0; }

.hsr-bg { position: absolute; font-size: 1.25rem; color: var(--p-rating-icon-color, var(--tk-text-muted)); opacity: 0.35; pointer-events: none; z-index: 0; }
.hsr-fg { position: absolute; font-size: 1.25rem; color: var(--p-rating-icon-active-color, var(--tk-accent)); pointer-events: none; z-index: 1; transition: clip-path 0.1s, opacity 0.1s; }

.hsr-readonly .hsr-half { cursor: default; pointer-events: none; }
</style>
