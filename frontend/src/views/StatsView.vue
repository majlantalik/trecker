<template>
  <div class="stats-view">
    <div class="view-header">
      <h1>Stats</h1>
    </div>

    <div v-if="loading" class="loading-state">
      <ProgressSpinner />
    </div>

    <div v-else class="stats-grid">
      <div class="stats-full">
        <ActivityChart :data="statsStore.activity" />
      </div>

      <div class="stats-col">
        <BreakdownChart title="By Genre" :data="statsStore.byGenre" />
      </div>
      <div class="stats-col">
        <BreakdownChart title="By Country" :data="statsStore.byCountry" />
      </div>

      <div class="stats-full">
        <TopRatedList :releases="statsStore.topRated" />
      </div>

      <div class="stats-full">
        <YearEndList
          :year="statsStore.selectedYear"
          :entries="statsStore.yearEnd"
          @year-change="statsStore.fetchYearEnd"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import ProgressSpinner from 'primevue/progressspinner'
import { useStatsStore } from '@/stores/stats'
import ActivityChart from '@/components/stats/ActivityChart.vue'
import BreakdownChart from '@/components/stats/BreakdownChart.vue'
import TopRatedList from '@/components/stats/TopRatedList.vue'
import YearEndList from '@/components/stats/YearEndList.vue'

const statsStore = useStatsStore()
const loading = computed(() => statsStore.loading)

onMounted(async () => {
  await statsStore.fetchAll()
  await statsStore.fetchYearEnd()
})
</script>

<style scoped>
.stats-view {
  max-width: 1100px;
}

.view-header {
  margin-bottom: 1.5rem;
}

.view-header h1 {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0;
}

.loading-state {
  display: flex;
  justify-content: center;
  padding: 3rem;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.stats-full {
  grid-column: 1 / -1;
}

.stats-col {
  min-width: 0;
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
  .stats-col {
    grid-column: 1;
  }
}
</style>
