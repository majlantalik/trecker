<template>
  <div class="chart-card">
    <h3>{{ title }}</h3>
    <Doughnut v-if="chartData" :data="chartData" :options="chartOptions" style="max-height: 240px" />
    <div v-else class="empty-state">No data yet.</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Doughnut } from 'vue-chartjs'
import {
  Chart as ChartJS,
  ArcElement,
  Tooltip,
  Legend
} from 'chart.js'
import type { BreakdownItem } from '@/types'
import { countryFlag, countryName } from '@/utils/country'

ChartJS.register(ArcElement, Tooltip, Legend)

const props = withDefaults(
  defineProps<{
    title: string
    data: BreakdownItem[]
    /** Labels arrive as ISO country codes; render them as names in the legend. */
    labelAs?: 'raw' | 'country'
  }>(),
  { labelAs: 'raw' }
)

function label(value: string) {
  if (props.labelAs !== 'country') return value
  const flag = countryFlag(value)
  return flag ? `${flag} ${countryName(value)}` : countryName(value)
}

const COLORS = [
  '#6366f1', '#8b5cf6', '#ec4899', '#f43f5e', '#f97316',
  '#eab308', '#22c55e', '#06b6d4', '#3b82f6', '#a855f7'
]

const chartData = computed(() => {
  if (!props.data.length) return null
  const top = props.data.slice(0, 10)
  return {
    labels: top.map(d => label(d.label)),
    datasets: [{
      data: top.map(d => d.count),
      backgroundColor: COLORS.slice(0, top.length),
      borderWidth: 1
    }]
  }
})

const chartOptions = {
  responsive: true,
  plugins: {
    legend: {
      position: 'right' as const,
      labels: { boxWidth: 12, font: { size: 11 } }
    }
  }
}
</script>

<style scoped>
.chart-card {
  background: var(--p-surface-900);
  border: 1px solid var(--p-surface-700);
  border-radius: 8px;
  padding: 1.25rem;
}

.chart-card h3 {
  margin: 0 0 1rem;
  font-size: 1rem;
  color: var(--p-text-muted-color);
}

.empty-state {
  color: var(--p-text-muted-color);
  text-align: center;
  padding: 2rem;
}
</style>
