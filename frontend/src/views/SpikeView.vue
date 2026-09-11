<template>
  <div class="spike">
    <h1>Phase 0 webview spike</h1>
    <p class="lede">
      Renders real components against a Tauri command. Everything below must look
      identical to the browser build.
    </p>

    <section class="panel">
      <div class="panel-header">
        <i class="pi pi-check-circle panel-header-icon" />
        <h2 class="panel-title">Automated checks</h2>
      </div>
      <div class="panel-body">
        <div v-for="c in checks" :key="c.name" class="check">
          <span class="check-dot" :class="c.pass ? 'ok' : 'bad'" />
          <span class="check-name">{{ c.name }}</span>
          <span class="check-value">{{ c.value }}</span>
        </div>
      </div>
    </section>

    <section class="panel">
      <div class="panel-header">
        <i class="pi pi-list panel-header-icon" />
        <h2 class="panel-title">ReleaseCard from invoke()</h2>
      </div>
      <div class="panel-body">
        <p v-if="error" class="error">{{ error }}</p>
        <p v-else-if="!releases.length">Loading…</p>
        <div v-else class="cards">
          <ReleaseCard v-for="r in releases" :key="r.id" :release="r" />
        </div>
      </div>
    </section>

    <section class="panel">
      <div class="panel-header">
        <i class="pi pi-chart-pie panel-header-icon" />
        <h2 class="panel-title">Chart.js canvas</h2>
      </div>
      <div class="panel-body">
        <BreakdownChart title="By genre" :data="breakdown" />
      </div>
    </section>

    <section class="panel">
      <div class="panel-header">
        <i class="pi pi-star panel-header-icon" />
        <h2 class="panel-title">Half-star rating</h2>
      </div>
      <div class="panel-body">
        <HalfStarRating v-model="rating" />
        <p class="hint">Value: {{ rating }}. Click to verify pointer maths in WebKit.</p>
      </div>
    </section>

    <section class="panel">
      <div class="panel-header">
        <i class="pi pi-arrows-v panel-header-icon" />
        <h2 class="panel-title">Scroll filler</h2>
      </div>
      <div class="panel-body">
        <p v-for="n in 30" :key="n" class="filler">
          Line {{ n }}. Scroll the page to check the custom scrollbar and the sticky
          header's backdrop blur.
        </p>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ReleaseCard from '@/components/release/ReleaseCard.vue'
import BreakdownChart from '@/components/stats/BreakdownChart.vue'
import HalfStarRating from '@/components/common/HalfStarRating.vue'
import type { Release, BreakdownItem } from '@/types'

const releases = ref<Release[]>([])
const error = ref('')
const rating = ref(3.5)

const breakdown: BreakdownItem[] = [
  { label: 'post-rock', count: 12 },
  { label: 'jazz', count: 9 },
  { label: 'electronic', count: 7 },
  { label: 'ambient', count: 5 },
  { label: 'post-punk', count: 3 }
]

const checks = ref<{ name: string; pass: boolean; value: string }[]>([])

onMounted(async () => {
  try {
    releases.value = await invoke<Release[]>('spike_releases')
  } catch (e) {
    error.value = `invoke failed: ${String(e)}`
  }

  // document.fonts settles asynchronously; wait before asserting on it.
  try {
    await document.fonts.ready
  } catch {
    /* not fatal for the spike */
  }

  const ua = navigator.userAgent
  const webkitVersion = /AppleWebKit\/([\d.]+)/.exec(ua)?.[1] ?? 'unknown'

  checks.value = [
    {
      name: 'invoke() round trip',
      pass: releases.value.length === 3,
      value: `${releases.value.length} releases`
    },
    {
      name: 'Syne (display font)',
      pass: document.fonts.check('700 16px Syne'),
      value: document.fonts.check('700 16px Syne') ? 'loaded' : 'FALLBACK'
    },
    {
      name: 'Outfit (body font)',
      pass: document.fonts.check('400 16px Outfit'),
      value: document.fonts.check('400 16px Outfit') ? 'loaded' : 'FALLBACK'
    },
    {
      name: 'backdrop-filter',
      pass:
        CSS.supports('backdrop-filter', 'blur(20px)') ||
        CSS.supports('-webkit-backdrop-filter', 'blur(20px)'),
      value: CSS.supports('backdrop-filter', 'blur(20px)')
        ? 'unprefixed'
        : CSS.supports('-webkit-backdrop-filter', 'blur(20px)')
          ? 'prefixed only'
          : 'UNSUPPORTED'
    },
    {
      name: 'position: sticky',
      pass: CSS.supports('position', 'sticky'),
      value: CSS.supports('position', 'sticky') ? 'yes' : 'NO'
    },
    {
      name: 'CSS custom properties',
      pass: CSS.supports('color', 'var(--tk-accent)'),
      value: getComputedStyle(document.documentElement)
        .getPropertyValue('--tk-accent')
        .trim()
    },
    { name: 'devicePixelRatio', pass: true, value: String(window.devicePixelRatio) },
    { name: 'AppleWebKit build', pass: true, value: webkitVersion }
  ]
})
</script>

<style scoped>
.spike {
  max-width: 1000px;
  margin: 0 auto;
  padding: 1.5rem;
}

.lede {
  color: rgba(226, 228, 240, 0.55);
  margin-bottom: 1.5rem;
}

.panel {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  overflow: hidden;
  box-shadow: inset 3px 0 0 var(--tk-accent);
  margin-bottom: 1.5rem;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.875rem 1.25rem;
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

.panel-body {
  padding: 1.25rem;
}

.check {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.4rem 0;
  font-size: 0.9rem;
}

.check-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
}

.check-dot.ok {
  background: var(--tk-accent);
}

.check-dot.bad {
  background: #f43f5e;
}

.check-name {
  flex: 1;
  color: var(--tk-text);
}

.check-value {
  color: rgba(226, 228, 240, 0.55);
  font-variant-numeric: tabular-nums;
}

.cards {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.hint {
  color: rgba(226, 228, 240, 0.45);
  font-size: 0.85rem;
  margin: 0.75rem 0 0;
}

.filler {
  color: rgba(226, 228, 240, 0.45);
  margin: 0.35rem 0;
}

.error {
  color: #f43f5e;
}
</style>
