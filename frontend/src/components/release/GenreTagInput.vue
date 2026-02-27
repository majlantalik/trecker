<template>
  <AutoComplete
    v-model="selectedGenres"
    :suggestions="filteredGenres"
    @complete="search"
    multiple
    fluid
    placeholder="Add genres..."
    :pt="{
      input: { style: 'width: 100%' }
    }"
  />
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import AutoComplete from 'primevue/autocomplete'
import { useGenresStore } from '@/stores/genres'

const props = defineProps<{
  modelValue: string[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const genresStore = useGenresStore()
const filteredGenres = ref<string[]>([])

const selectedGenres = ref<string[]>([...props.modelValue])

watch(() => props.modelValue, (val) => {
  selectedGenres.value = [...val]
})

watch(selectedGenres, (val) => {
  emit('update:modelValue', val)
})

onMounted(() => {
  genresStore.fetchGenres()
})

function search(event: { query: string }) {
  const q = event.query.toLowerCase()
  filteredGenres.value = genresStore.genres.filter(g =>
    g.toLowerCase().includes(q)
  )
  if (q && !filteredGenres.value.includes(event.query)) {
    filteredGenres.value = [event.query, ...filteredGenres.value]
  }
}
</script>
