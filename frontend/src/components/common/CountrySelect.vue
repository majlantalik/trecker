<template>
  <Select
    ref="select"
    :model-value="modelValue"
    :options="options"
    option-label="label"
    option-value="value"
    :filter-fields="['label', 'value']"
    filter
    auto-filter-focus
    filter-placeholder="Find a country..."
    :placeholder="placeholder"
    aria-label="Country"
    v-bind="$attrs"
    @update:model-value="(v: string | null) => emit('update:modelValue', v)"
  >
    <template #value="{ value, placeholder: empty }">
      <CountryLabel v-if="value" :value="value" />
      <span v-else>{{ empty }}</span>
    </template>
    <template #option="{ option }">
      <CountryLabel :value="option.value" />
    </template>
  </Select>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import Select from 'primevue/select'
import CountryLabel from '@/components/common/CountryLabel.vue'
import { countryOptions } from '@/utils/country'

/**
 * Picks a release's country from the full list, searchable by name or code.
 *
 * Emits the two-letter code, which is what the database stores. Everything else a PrimeVue
 * Select accepts, such as `fluid`, `show-clear` or `@hide`, passes straight through.
 */
const props = withDefaults(
  defineProps<{
    modelValue: string | null | undefined
    placeholder?: string
  }>(),
  { placeholder: 'Choose a country' }
)

const emit = defineEmits<{ 'update:modelValue': [value: string | null] }>()

defineOptions({ inheritAttrs: false })

// Recomputed from the current value, so a stored value that is not on the list stays
// selectable rather than vanishing when the picker opens.
const options = computed(() => countryOptions(props.modelValue))

const select = ref<{ show: (focus?: boolean) => void } | null>(null)

/** Opens the list. For editors that swap a label for this picker on click. */
function open() {
  select.value?.show(true)
}

defineExpose({ open })
</script>
