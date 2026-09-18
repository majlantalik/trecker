import { config } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach } from 'vitest'

// Stubs for PrimeVue components that properly forward v-model
config.global.stubs = {
  Dialog: {
    template: '<div data-stub="dialog"><slot /><slot name="header" /></div>',
    props: ['visible', 'header', 'modal', 'draggable', 'style'],
    emits: ['update:visible']
  },
  Button: {
    template: '<button :data-label="label" @click="$emit(\'click\', $event)"><slot /></button>',
    props: ['label', 'loading', 'severity', 'outlined', 'icon', 'type'],
    emits: ['click']
  },
  InputText: {
    template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
    props: ['modelValue', 'placeholder', 'fluid', 'required'],
    emits: ['update:modelValue']
  },
  CountrySelect: {
    template: '<select data-stub="country-select" :data-value="modelValue" @change="$emit(\'update:modelValue\', $event.target.value)"></select>',
    props: ['modelValue', 'fluid', 'placeholder'],
    emits: ['update:modelValue']
  },
  InputNumber: {
    template: '<input type="number" :value="modelValue" @input="$emit(\'update:modelValue\', Number($event.target.value))" />',
    props: ['modelValue', 'placeholder', 'min', 'max', 'useGrouping', 'fluid'],
    emits: ['update:modelValue']
  },
  HalfStarRating: {
    template: '<div data-stub="half-star-rating" :data-value="modelValue" @click="$emit(\'update:modelValue\', 5)"></div>',
    props: ['modelValue', 'readonly', 'cancel'],
    emits: ['update:modelValue']
  },
  Checkbox: {
    template: '<input type="checkbox" :checked="modelValue" @change="$emit(\'update:modelValue\', $event.target.checked)" />',
    props: ['modelValue', 'binary', 'inputId'],
    emits: ['update:modelValue']
  },
  Textarea: {
    template: '<textarea :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)"></textarea>',
    props: ['modelValue', 'rows', 'placeholder', 'fluid'],
    emits: ['update:modelValue']
  },
  Divider: { template: '<hr />' },
  AutoComplete: {
    template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
    props: ['modelValue', 'suggestions', 'multiple'],
    emits: ['update:modelValue', 'complete']
  },
  GenreTagInput: {
    template: '<div data-stub="genre-input" :data-value="(modelValue || []).join(\'|\')"></div>',
    props: ['modelValue'],
    emits: ['update:modelValue']
  }
}

// Fresh Pinia instance before each test
beforeEach(() => {
  setActivePinia(createPinia())
})
