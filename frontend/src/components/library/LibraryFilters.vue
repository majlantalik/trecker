<template>
  <div class="library-filters">
    <div class="filters-header">
      <i class="pi pi-sliders-h filters-header-icon" />
      <span class="filters-header-title">Filters</span>
    </div>

    <div class="filters-body">
      <div class="filter-group">
        <div class="filter-item">
          <label class="filter-label">
            <i class="pi pi-search" />
            Search
          </label>
          <InputText v-model="localFilters.search" placeholder="Artist or title..." fluid @input="debouncedEmit" />
        </div>
      </div>

      <div class="filter-divider" />

      <div class="filter-group">
        <div class="filter-item">
          <label class="filter-label">
            <i class="pi pi-tag" />
            Status
          </label>
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

        <div class="filter-item">
          <label class="filter-label">
            <i class="pi pi-microphone" />
            Genre
          </label>
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

        <div class="filter-item">
          <label class="filter-label">
            <i class="pi pi-globe" />
            Country
          </label>
          <InputText v-model="localFilters.country" placeholder="e.g. US, GB" fluid @input="debouncedEmit" />
        </div>

        <div class="filter-item">
          <label class="filter-label">
            <i class="pi pi-calendar" />
            Year
          </label>
          <InputNumber v-model="localFilters.year" placeholder="2024" :min="1900" :max="2099" fluid @input="debouncedEmit" />
        </div>
      </div>

      <div class="filter-divider" />

      <div class="filter-group">
        <div class="filter-item">
          <label class="filter-label">
            <i class="pi pi-star" />
            Min Rating
          </label>
          <HalfStarRating
            :modelValue="localFilters.ratingMin ?? null"
            :cancel="true"
            @update:modelValue="v => { localFilters.ratingMin = v ?? undefined; emit() }"
            class="filter-rating"
          />
        </div>

        <div class="filter-item">
          <label class="filter-label inline-label">
            <Checkbox v-model="localFilters.didNotFinish" binary @change="emit" />
            DNF only
          </label>
        </div>
      </div>
    </div>

    <div class="filters-footer">
      <button class="clear-btn" @click="clearFilters">
        <i class="pi pi-times" />
        Clear Filters
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import Select from 'primevue/select'
import AutoComplete from 'primevue/autocomplete'
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import Checkbox from 'primevue/checkbox'
import { useGenresStore } from '@/stores/genres'
import type { ReleaseFilterParams } from '@/types'

const props = defineProps<{
  initialGenre?: string
  initialStatus?: string
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
  status: props.initialStatus as ReleaseFilterParams['status'],
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
  emits('update:filters', {
    search: localFilters.value.search || undefined,
    status: localFilters.value.status || undefined,
    genre: localFilters.value.genre ? localFilters.value.genre as string : undefined,
    country: localFilters.value.country || undefined,
    year: localFilters.value.year || undefined,
    ratingMin: localFilters.value.ratingMin || undefined,
    didNotFinish: localFilters.value.didNotFinish || undefined,
  })
}

function clearFilters() {
  localFilters.value = {
    search: '',
    status: props.initialStatus as ReleaseFilterParams['status'],
    genre: undefined,
    country: '',
    year: undefined,
    ratingMin: undefined,
    didNotFinish: undefined
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
  if (props.initialStatus) {
    localFilters.value.status = props.initialStatus as ReleaseFilterParams['status']
  }
})
</script>

<style scoped>
.library-filters {
  display: flex;
  flex-direction: column;
  border-radius: 14px;
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  overflow: hidden;
  /* Accent left border */
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

/* ── Header ── */
.filters-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 1rem 1.25rem 0.875rem;
  border-bottom: 1px solid var(--tk-border);
  background: rgba(0, 229, 176, 0.05);
}

.filters-header-icon {
  color: var(--tk-accent);
  font-size: 0.9rem;
}

.filters-header-title {
  font-family: var(--tk-font-display);
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-accent);
}

/* ── Body ── */
.filters-body {
  padding: 1rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.filter-divider {
  height: 1px;
  background: var(--tk-border);
  margin: 1rem 0;
}

.filter-item {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.filter-label {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--tk-text-muted);
}

.filter-label i {
  font-size: 0.7rem;
  color: var(--tk-accent);
  opacity: 0.7;
}

.inline-label {
  flex-direction: row;
  gap: 0.6rem;
  cursor: pointer;
  font-size: 0.82rem;
  text-transform: none;
  letter-spacing: 0;
  font-weight: 500;
  color: var(--tk-text-muted);
}

.filter-rating {
  font-size: 1.1rem;
}

/* ── Footer ── */
.filters-footer {
  padding: 0.75rem 1.25rem 1rem;
  border-top: 1px solid var(--tk-border);
}

.clear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  width: 100%;
  padding: 0.55rem 1rem;
  background: transparent;
  border: 1px solid var(--tk-border);
  border-radius: 8px;
  color: var(--tk-text-muted);
  font-family: var(--tk-font-body);
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.clear-btn:hover {
  border-color: rgba(0, 229, 176, 0.35);
  color: var(--tk-accent);
  background: var(--tk-accent-dim);
}

.clear-btn i {
  font-size: 0.75rem;
}
</style>
