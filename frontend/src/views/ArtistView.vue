<template>
  <div class="artist-view" v-if="artist">
    <div class="artist-header">
      <Button icon="pi pi-arrow-left" label="Back" text @click="$router.back()" />
      <Button
        icon="pi pi-trash"
        severity="danger"
        text
        aria-label="Remove from your artists"
        v-tooltip.bottom="'Remove from your artists'"
        @click="confirmDelete"
      />
    </div>

    <!-- Hero -->
    <div class="hero-card">
      <div class="hero-inner">
        <ArtistAvatar :artist="artist" :size="168" />

        <div class="hero-info">
          <h1 class="artist-name">{{ artist.name }}</h1>
          <p v-if="artist.disambiguation" class="artist-disambiguation">{{ artist.disambiguation }}</p>

          <div class="meta-bar">
            <span v-if="artist.country" class="meta-pill"><CountryLabel :value="artist.country" /></span>
            <span v-if="artistKind(artist.artistType)" class="meta-pill">{{ artistKind(artist.artistType) }}</span>
            <span v-if="activeYears(artist)" class="meta-pill">{{ activeYears(artist) }}</span>
          </div>

          <div v-if="artist.genres.length" class="genres">
            <Tag v-for="genre in artist.genres" :key="genre" :value="genre" severity="secondary" />
          </div>

          <div v-if="links.length" class="links-list">
            <a v-for="link in links" :key="link.url" :href="link.url" target="_blank" class="link-chip">
              <i class="pi pi-external-link" /> {{ link.label }}
            </a>
          </div>

          <!-- Checking them out. The same three choices whether or not they are checked yet,
               so changing your mind is the same click as deciding. -->
          <div class="verdict-row">
            <span class="verdict-prompt">
              {{ artist.status === 'CHECKED' ? `Checked ${formatDate(artist.checkedAt)}` : 'Checked them out?' }}
            </span>
            <div class="verdict-buttons" role="group" aria-label="Your verdict">
              <button
                v-for="choice in CHOICES"
                :key="choice.key"
                type="button"
                class="verdict-btn"
                :class="[choice.key.toLowerCase(), { active: isChosen(choice.verdict) }]"
                :aria-pressed="isChosen(choice.verdict)"
                :disabled="saving"
                @click="check(choice.verdict)"
              >
                <i :class="choice.icon" /> {{ choice.label }}
              </button>
            </div>
            <Button
              v-if="artist.status === 'CHECKED'"
              label="Back to the list"
              icon="pi pi-undo"
              text
              size="small"
              :disabled="saving"
              @click="uncheck"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Note -->
    <div class="panel">
      <div class="panel-header">
        <i class="pi pi-pencil panel-header-icon" />
        <h3 class="panel-title">Note</h3>
      </div>
      <div class="panel-body">
        <Textarea
          v-model="noteDraft"
          rows="2"
          auto-resize
          placeholder="Who recommended them, where you heard them, where to start..."
          class="note-textarea"
          fluid
          @blur="saveNote"
        />
      </div>
    </div>

    <!-- Discography -->
    <div class="panel">
      <div class="panel-header">
        <i class="pi pi-images panel-header-icon" />
        <h3 class="panel-title">Discography</h3>
        <span class="panel-spacer" />
        <Button
          v-if="discography.hiddenCount.value > 0"
          :label="discography.showAll.value ? 'Albums and EPs only' : `Show live, compilations and more (${discography.hiddenCount.value})`"
          text
          size="small"
          class="show-all-btn"
          @click="discography.showAll.value = !discography.showAll.value"
        />
      </div>

      <div class="panel-body discography-body">
        <p
          v-if="artist.status === 'TO_CHECK' && discography.listenedCount.value > 0"
          class="checked-hint"
        >
          <i class="pi pi-lightbulb" />
          You have listened to {{ discography.listenedCount.value }}
          {{ discography.listenedCount.value === 1 ? 'album' : 'albums' }} of theirs. Done checking them out? Choose above.
        </p>

        <div v-if="discography.loading.value" class="discography-state">
          <ProgressSpinner style="width: 36px; height: 36px" />
          <span>Fetching their albums from MusicBrainz…</span>
        </div>

        <div v-else-if="discography.error.value" class="discography-state">
          <span>The discography needs a connection to MusicBrainz. {{ discography.error.value }}</span>
          <Button label="Try again" icon="pi pi-refresh" size="small" outlined @click="discography.load()" />
        </div>

        <p v-else-if="discography.loaded.value && discography.visible.value.length === 0" class="discography-state">
          MusicBrainz lists no albums or EPs for {{ artist.name }}.
        </p>

        <ul v-else class="album-list">
          <li v-for="album in discography.visible.value" :key="album.musicbrainzReleaseGroupId" class="album-row">
            <img
              v-if="!missingCovers.has(album.musicbrainzReleaseGroupId)"
              :src="coverSrc(album.albumArtUrl)"
              class="album-art"
              alt=""
              loading="lazy"
              @error="missingCovers.add(album.musicbrainzReleaseGroupId)"
            />
            <div v-else class="album-art album-art-empty"><i class="pi pi-music" /></div>

            <div class="album-text">
              <span class="album-title">{{ album.title }}</span>
              <span class="album-meta">{{ describeCandidate(album) }}</span>
            </div>

            <RouterLink
              v-if="album.library"
              :to="{ name: 'entry', params: { id: album.library.releaseId } }"
              class="library-badge"
              :class="album.library.status.toLowerCase()"
            >
              <template v-if="album.library.status === 'LISTENED'">
                <i class="pi pi-check" /> Listened
                <span v-if="album.library.rating" class="badge-rating">★ {{ album.library.rating }}</span>
              </template>
              <template v-else><i class="pi pi-list" /> In queue</template>
            </RouterLink>

            <div class="album-actions">
              <i v-if="discography.busyId.value === album.musicbrainzReleaseGroupId" class="pi pi-spin pi-spinner album-busy" />
              <template v-else>
                <Button
                  v-if="!album.library"
                  label="Queue"
                  icon="pi pi-plus"
                  size="small"
                  outlined
                  :disabled="!!discography.busyId.value"
                  @click="discography.queue(album)"
                />
                <Button
                  v-if="album.library?.status !== 'LISTENED'"
                  label="Log"
                  icon="pi pi-headphones"
                  size="small"
                  :disabled="!!discography.busyId.value"
                  @click="logAlbum(album)"
                />
              </template>
            </div>
          </li>
        </ul>
      </div>
    </div>

    <ReleaseForm
      v-model:visible="discography.formVisible.value"
      :prefill="discography.prefill.value"
      @submit="submitForm"
    />

    <QuickLogModal
      v-model:visible="discography.logVisible.value"
      :release="discography.logRelease.value"
      @logged="discography.logged"
    />

    <ConfirmDialog :pt="{ root: { class: 'tk-dialog' } }" />
  </div>

  <div v-else-if="notFound" class="not-found">
    <p>This artist is not on your list.</p>
    <Button label="Back to Artists" @click="$router.push('/artists')" />
  </div>

  <div v-else class="loading-state">
    <ProgressSpinner />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import Button from 'primevue/button'
import ConfirmDialog from 'primevue/confirmdialog'
import ProgressSpinner from 'primevue/progressspinner'
import Tag from 'primevue/tag'
import Textarea from 'primevue/textarea'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import { coverSrc } from '@/api/cache'
import { artistsApi } from '@/api/artists'
import ArtistAvatar from '@/components/artist/ArtistAvatar.vue'
import CountryLabel from '@/components/common/CountryLabel.vue'
import QuickLogModal from '@/components/release/QuickLogModal.vue'
import ReleaseForm from '@/components/release/ReleaseForm.vue'
import { useDiscography } from '@/composables/useDiscography'
import { useArtistsStore } from '@/stores/artists'
import { activeYears, artistKind, labelledLinks, VERDICTS } from '@/utils/artists'
import { describeCandidate } from '@/utils/candidates'
import type { Artist, ArtistVerdict, DiscographyEntry, ReleaseRequest } from '@/types'

const props = defineProps<{ id: string }>()

const router = useRouter()
const toast = useToast()
const confirm = useConfirm()
const store = useArtistsStore()

const artist = ref<Artist | null>(null)
const notFound = ref(false)
const saving = ref(false)
const noteDraft = ref('')
const missingCovers = reactive(new Set<string>())

const discography = useDiscography(
  () => props.id,
  () => artist.value?.name ?? ''
)

const links = computed(() => (artist.value ? labelledLinks(artist.value.links) : []))

const CHOICES: { key: string; verdict: ArtistVerdict | null; label: string; icon: string }[] = [
  { key: 'LIKED', verdict: 'LIKED', ...VERDICTS.LIKED },
  { key: 'NOT_FOR_ME', verdict: 'NOT_FOR_ME', ...VERDICTS.NOT_FOR_ME },
  { key: 'PLAIN', verdict: null, label: 'Just checked', icon: 'pi pi-check' }
]

async function load() {
  artist.value = null
  notFound.value = false
  missingCovers.clear()
  try {
    artist.value = await artistsApi.get(props.id)
    noteDraft.value = artist.value.note ?? ''
  } catch {
    notFound.value = true
    return
  }
  discography.load()
}

onMounted(load)
watch(() => props.id, load)

// The dialog closing is the one moment to know a log was abandoned.
watch(discography.logVisible, (open) => {
  if (!open) discography.logClosed()
})

function isChosen(verdict: ArtistVerdict | null) {
  return artist.value?.status === 'CHECKED' && artist.value.verdict === verdict
}

async function save(change: Parameters<typeof store.updateArtist>[1]) {
  if (!artist.value) return
  saving.value = true
  try {
    artist.value = await store.updateArtist(artist.value.id, change)
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Could not save', detail: e?.message, life: 4000 })
  } finally {
    saving.value = false
  }
}

function check(verdict: ArtistVerdict | null) {
  save({ status: 'CHECKED', verdict })
}

function uncheck() {
  save({ status: 'TO_CHECK' })
}

async function saveNote() {
  if (!artist.value || noteDraft.value.trim() === (artist.value.note ?? '')) return
  await save({ note: noteDraft.value })
}

async function submitForm(data: ReleaseRequest) {
  try {
    const release = await discography.submitForm(data)
    toast.add({ severity: 'success', summary: 'Added to queue', detail: `${release.artist} – ${release.title}`, life: 3000 })
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Failed to add', detail: e?.message, life: 4000 })
  }
}

async function logAlbum(album: DiscographyEntry) {
  try {
    await discography.log(album)
  } catch (e: any) {
    toast.add({ severity: 'error', summary: 'Could not open the album', detail: e?.message, life: 4000 })
  }
}

function confirmDelete() {
  if (!artist.value) return
  const { id, name } = artist.value
  confirm.require({
    message: `Remove ${name} from your artists? Your note goes with them.`,
    header: 'Remove artist',
    icon: 'pi pi-trash',
    rejectProps: { label: 'Cancel', severity: 'secondary', outlined: true },
    acceptProps: { label: 'Remove', severity: 'danger' },
    accept: async () => {
      try {
        await store.deleteArtist(id)
        toast.add({ severity: 'info', summary: 'Removed', detail: name, life: 2000 })
        router.push('/artists')
      } catch (e: any) {
        toast.add({ severity: 'error', summary: 'Failed', detail: e?.message, life: 3000 })
      }
    }
  })
}

function formatDate(iso: string | null) {
  if (!iso) return ''
  return new Date(iso).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })
}
</script>

<style scoped>
.artist-view {
  max-width: 820px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.artist-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.hero-card {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 16px;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.hero-inner {
  display: flex;
  gap: 1.75rem;
  padding: 1.75rem;
  align-items: flex-start;
}

.hero-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-width: 0;
}

.artist-name {
  font-size: 1.85rem;
  font-weight: 700;
  margin: 0;
  line-height: 1.2;
  color: var(--tk-text);
}

.artist-disambiguation {
  margin: -0.4rem 0 0;
  font-size: 0.95rem;
  color: rgba(226, 228, 240, 0.55);
}

.meta-bar,
.genres,
.links-list {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.meta-pill {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.6rem;
  border-radius: 6px;
  font-size: 0.82rem;
  color: var(--tk-text);
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--tk-border);
}

.link-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.7rem;
  border-radius: 6px;
  border: 1px solid var(--tk-border);
  background: rgba(255, 255, 255, 0.04);
  color: var(--tk-text);
  text-decoration: none;
  font-size: 0.85rem;
  transition: background 0.15s, border-color 0.15s;
}

.link-chip:hover {
  background: rgba(0, 229, 176, 0.08);
  border-color: rgba(0, 229, 176, 0.3);
}

.link-chip i {
  font-size: 0.7rem;
  color: var(--tk-accent);
}

.verdict-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.6rem;
  margin-top: 0.35rem;
  padding-top: 0.9rem;
  border-top: 1px solid var(--tk-border);
}

.verdict-prompt {
  font-size: 0.85rem;
  color: rgba(226, 228, 240, 0.6);
}

.verdict-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.verdict-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.35rem 0.75rem;
  border-radius: 999px;
  border: 1px solid var(--tk-border);
  background: transparent;
  color: rgba(226, 228, 240, 0.8);
  font: inherit;
  font-size: 0.82rem;
  cursor: pointer;
  transition: all 0.15s;
}

.verdict-btn:hover:not(:disabled) {
  border-color: rgba(0, 229, 176, 0.35);
  color: var(--tk-text);
}

.verdict-btn:disabled {
  cursor: default;
  opacity: 0.6;
}

.verdict-btn.active {
  color: var(--tk-text);
  border-color: rgba(226, 228, 240, 0.4);
  background: rgba(255, 255, 255, 0.06);
}

.verdict-btn.liked.active {
  color: var(--tk-accent);
  border-color: rgba(0, 229, 176, 0.45);
  background: var(--tk-accent-dim);
}

.verdict-btn.not_for_me.active {
  color: #f87171;
  border-color: rgba(248, 113, 113, 0.45);
  background: rgba(248, 113, 113, 0.1);
}

.panel {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  overflow: hidden;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.6rem 1.25rem;
  min-height: 2.9rem;
  border-bottom: 1px solid var(--tk-border);
  background: rgba(0, 229, 176, 0.05);
}

.panel-header-icon {
  font-size: 0.875rem;
  color: var(--tk-accent);
}

.panel-title {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-accent);
  margin: 0;
}

.panel-spacer {
  flex: 1;
}

.show-all-btn {
  font-size: 0.78rem;
}

.panel-body {
  padding: 1.25rem;
}

.note-textarea {
  width: 100%;
  resize: vertical;
}

.discography-body {
  padding: 0.5rem 0;
}

.checked-hint {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0.5rem 1.25rem 0.75rem;
  padding: 0.6rem 0.8rem;
  border-radius: 8px;
  font-size: 0.84rem;
  color: var(--tk-text);
  background: var(--tk-accent-dim);
}

.checked-hint i {
  color: var(--tk-accent);
}

.discography-state {
  display: flex;
  align-items: center;
  gap: 0.9rem;
  flex-wrap: wrap;
  margin: 0;
  padding: 1rem 1.25rem;
  font-size: 0.88rem;
  color: rgba(226, 228, 240, 0.6);
}

.album-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.album-row {
  display: flex;
  align-items: center;
  gap: 0.9rem;
  padding: 0.6rem 1.25rem;
  transition: background 0.15s;
}

.album-row + .album-row {
  border-top: 1px solid var(--tk-border);
}

.album-row:hover {
  background: rgba(255, 255, 255, 0.025);
}

.album-art {
  width: 52px;
  height: 52px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
}

.album-art-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  color: rgba(226, 228, 240, 0.3);
}

.album-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.album-title,
.album-meta {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.album-title {
  font-size: 0.94rem;
  font-weight: 600;
  color: var(--tk-text);
}

.album-meta {
  font-size: 0.78rem;
  color: rgba(226, 228, 240, 0.5);
}

.library-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  flex-shrink: 0;
  padding: 0.25rem 0.6rem;
  border-radius: 999px;
  font-size: 0.76rem;
  text-decoration: none;
  border: 1px solid var(--tk-border);
  color: rgba(226, 228, 240, 0.75);
}

.library-badge.listened {
  color: var(--tk-accent);
  border-color: rgba(0, 229, 176, 0.3);
  background: var(--tk-accent-dim);
}

.library-badge.queued {
  color: #fbbf24;
  border-color: rgba(251, 191, 36, 0.3);
  background: rgba(251, 191, 36, 0.08);
}

.library-badge i {
  font-size: 0.7rem;
}

.badge-rating {
  margin-left: 0.15rem;
  font-weight: 600;
}

.album-actions {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  flex-shrink: 0;
}

.album-busy {
  color: var(--tk-accent);
  margin: 0 0.75rem;
}

.not-found,
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: 4rem 2rem;
  color: var(--tk-text-muted);
}

@media (max-width: 640px) {
  .hero-inner {
    flex-direction: column;
    align-items: center;
  }
}
</style>
