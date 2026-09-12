<template>
  <span v-if="value" class="country-label" :title="value">
    <span v-if="flag" class="country-flag" aria-hidden="true">{{ flag }}</span>
    <span class="country-name">{{ compact ? code : name }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { countryFlag, countryName } from '@/utils/country'

const props = withDefaults(
  defineProps<{
    value: string | null | undefined
    /** Show the bare code next to the flag. For tight rows where a full name would wrap. */
    compact?: boolean
  }>(),
  { compact: false }
)

const flag = computed(() => countryFlag(props.value))
const name = computed(() => countryName(props.value))
// The stored value is the truth, and it is what the title attribute shows on hover.
const code = computed(() => (props.value ?? '').trim().toUpperCase())
</script>

<style scoped>
.country-label {
  display: inline-flex;
  align-items: baseline;
  gap: 0.4em;
  white-space: nowrap;
}

.country-flag {
  /* Ask for the emoji font explicitly. Without one installed the codepoints fall back to
     two boxed letters, which still reads as the country code. */
  font-family: 'Noto Color Emoji', 'Apple Color Emoji', 'Segoe UI Emoji', sans-serif;
  font-size: 0.95em;
  line-height: 1;
}

.country-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
