<template>
  <div class="artists-view">
    <div class="view-header">
      <h1>Artists <span class="count" v-if="store.loaded">({{ store.toCheck.length }})</span></h1>
    </div>

    <form class="add-row" @submit.prevent="find">
      <div class="add-input-wrap">
        <i class="pi pi-user-plus add-icon" />
        <InputText
          v-model="text"
          class="add-input"
          placeholder="Find an artist to check out..."
          aria-label="Artist name"
          fluid
        />
      </div>
      <Button type="submit" label="Find" icon="pi pi-search" :loading="searching" :disabled="!text.trim()" />
    </form>

    <div v-if="store.loading && !store.loaded" class="loading-state">
      <ProgressSpinner />
    </div>

    <p v-else-if="store.error" class="load-error">Could not load your artists: {{ store.error }}</p>

    <div v-else-if="store.artists.length === 0" class="empty-state">
      <i class="pi pi-users empty-icon" />
      <p>No artists yet. Heard a name worth exploring? Find them above and they wait here.</p>
    </div>

    <template v-else>
      <p v-if="store.toCheck.length === 0" class="all-done">
        <i class="pi pi-check-circle" /> Nobody left to check.
      </p>
      <TransitionGroup name="list" tag="div" class="artist-list">
        <ArtistCard v-for="artist in store.toCheck" :key="artist.id" :artist="artist" @click="open" />
      </TransitionGroup>

      <section v-if="store.checked.length" class="checked-section">
        <button
          type="button"
          class="checked-toggle"
          :aria-expanded="showChecked"
          @click="showChecked = !showChecked"
        >
          <i :class="showChecked ? 'pi pi-chevron-down' : 'pi pi-chevron-right'" />
          Checked <span class="count">({{ store.checked.length }})</span>
        </button>
        <div v-if="showChecked" class="artist-list">
          <ArtistCard v-for="artist in store.checked" :key="artist.id" :artist="artist" @click="open" />
        </div>
      </section>
    </template>

    <ArtistPicker
      v-model:visible="pickerVisible"
      :candidates="candidates"
      :query="query"
      :adding-id="addingId"
      :error="pickerVisible ? error : ''"
      @choose="add"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import ProgressSpinner from 'primevue/progressspinner'
import { useToast } from 'primevue/usetoast'
import ArtistCard from '@/components/artist/ArtistCard.vue'
import ArtistPicker from '@/components/artist/ArtistPicker.vue'
import { useArtistSearch } from '@/composables/useArtistSearch'
import { useArtistsStore } from '@/stores/artists'
import type { Artist, ArtistCandidate } from '@/types'

const router = useRouter()
const toast = useToast()
const store = useArtistsStore()
const { query, searching, candidates, pickerVisible, addingId, error, search, choose } = useArtistSearch()

const text = ref('')
// Collapsed by default: the page is about who is left to check.
const showChecked = ref(false)

onMounted(() => store.fetchArtists())

async function find() {
  const outcome = await search(text.value)
  if (outcome === 'none') {
    toast.add({ severity: 'info', summary: 'No artist found on MusicBrainz', detail: query.value, life: 3500 })
  } else if (outcome === 'failed') {
    toast.add({ severity: 'warn', summary: 'Could not search MusicBrainz', detail: error.value, life: 4000 })
  }
}

async function add(candidate: ArtistCandidate) {
  const known = store.artists.some((a) => a.musicbrainzArtistId === candidate.musicbrainzArtistId)
  const artist = await choose(candidate)
  if (!artist) return
  text.value = ''
  toast.add({
    severity: known ? 'info' : 'success',
    summary: known ? 'Already on your list' : 'Added to your artists',
    detail: artist.name,
    life: 3000
  })
}

function open(artist: Artist) {
  router.push({ name: 'artist', params: { id: artist.id } })
}
</script>

<style scoped>
.artists-view {
  flex: 1;
  max-width: 900px;
  margin: 0 auto;
  width: 100%;
  display: flex;
  flex-direction: column;
}

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.25rem;
}

.view-header h1 {
  font-size: 1.75rem;
  font-weight: 700;
}

.count {
  color: var(--tk-text-muted);
  font-weight: 400;
  font-family: var(--tk-font-body);
  letter-spacing: 0;
}

.add-row {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 1.75rem;
}

.add-input-wrap {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
}

.add-icon {
  position: absolute;
  left: 0.8rem;
  color: var(--tk-text-muted);
  pointer-events: none;
}

.add-input {
  padding-left: 2.3rem;
}

.loading-state {
  display: flex;
  justify-content: center;
  padding: 3rem;
}

.load-error {
  color: #f43f5e;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--tk-text-muted);
}

.empty-icon {
  font-size: 3rem;
  display: block;
  margin-bottom: 1rem;
}

.all-done {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0 0 1rem;
  color: rgba(226, 228, 240, 0.55);
}

.all-done i {
  color: var(--tk-accent);
}

.artist-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.checked-section {
  margin-top: 2rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.checked-toggle {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.25rem;
  border: none;
  background: none;
  cursor: pointer;
  font: inherit;
  font-family: var(--tk-font-display);
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--tk-text);
}

.checked-toggle i {
  font-size: 0.75rem;
  color: var(--tk-accent);
}

.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>
