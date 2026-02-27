<template>
  <div class="library-filters">
    <div class="filter-section">
      <label>Search</label>
      <InputText v-model="localFilters.search" placeholder="Artist or title..." fluid @input="debouncedEmit" />
    </div>

    <div class="filter-section">
      <label>Status</label>
      <Select
        v-model="localFilters.status"
        :options="statuses"
        option-label="label"
        option-value="value"
        placeholder="All"
        show-clear
        fluid
        @change="emit"
      />
    </div>

    <div class="filter-section">
      <label>Genre</label>
      <AutoComplete
        v-model="localFilters.genre"
        :suggestions="filteredGenres"
        @complete="searchGenre"
        placeholder="Filter by genre..."
        fluid
        @item-select="emit"
        @clear="emit"
      />
    </div>

    <div class="filter-section">
      <label>Country</label>
      <InputText v-model="localFilters.country" placeholder="e.g. US, GB" fluid @input="debouncedEmit" />
    </div>

    <div class="filter-section">
      <label>Year</label>
      <InputNumber v-model="localFilters.year" placeholder="2024" :min="1900" :max="2099" fluid @input="debouncedEmit" />
    </div>

    <div class="filter-section">
      <label>Min Rating</label>
      <Rating v-model="localFilters.ratingMin" :stars="5" :cancel="true" @change="emit" />
    </div>

    <div class="filter-section">
      <label class="inline-label">
        <Checkbox v-model="localFilters.didNotFinish" binary @change="emit" />
        DNF only
      </label>
    </div>

    <Button label="Clear Filters" severity="secondary" outlined size="small" @click="clearFilters" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import Select from 'primevue/select'
import AutoComplete from 'primevue/autocomplete'
import Rating from 'primevue/rating'
import Checkbox from 'primevue/checkbox'
import Button from 'primevue/button'
import { useGenresStore } from '@/stores/genres'
import type { ReleaseFilterParams } from '@/types'

const props = defineProps<{
  initialGenre?: string
}>()

const emits = defineEmits<{
  'update:filters': [filters: ReleaseFilterParams]
}>()

const genresStore = useGenresStore()
const filteredGenres = ref<string[]>([])

const statuses = [
  { label: 'Queued', value: 'QUEUED' },
  { label: 'Listened', value: 'LISTENED' }
]

const localFilters = ref<ReleaseFilterParams>({
  search: '',
  status: undefined,
  genre: undefined,
  country: '',
  year: undefined,
  ratingMin: undefined,
  didNotFinish: undefined
})

let debounceTimer: ReturnType<typeof setTimeout>

function debouncedEmit() {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(emit, 300)
}

function emit() {
  const filters: ReleaseFilterParams = {}
  if (localFilters.value.search) filters.search = localFilters.value.search
  if (localFilters.value.status) filters.status = localFilters.value.status
  if (localFilters.value.genre) filters.genre = localFilters.value.genre as string
  if (localFilters.value.country) filters.country = localFilters.value.country
  if (localFilters.value.year) filters.year = localFilters.value.year
  if (localFilters.value.ratingMin) filters.ratingMin = localFilters.value.ratingMin
  if (localFilters.value.didNotFinish) filters.didNotFinish = localFilters.value.didNotFinish
  emits('update:filters', filters)
}

function clearFilters() {
  localFilters.value = {
    search: '', status: undefined, genre: undefined,
    country: '', year: undefined, ratingMin: undefined, didNotFinish: undefined
  }
  emit()
}

function searchGenre(event: { query: string }) {
  const q = event.query.toLowerCase()
  filteredGenres.value = genresStore.genres.filter(g => g.toLowerCase().includes(q))
}

onMounted(() => {
  genresStore.fetchGenres()
  if (props.initialGenre) {
    localFilters.value.genre = props.initialGenre
  }
})
</script>

<style scoped>
.library-filters {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  background: var(--p-surface-900);
  border-radius: 8px;
  border: 1px solid var(--p-surface-700);
}

.filter-section {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.filter-section label {
  font-size: 0.8rem;
  color: var(--p-text-muted-color);
  font-weight: 500;
}

.inline-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}
</style>
