<template>
  <div class="chart-card">
    <h3>Listening Activity</h3>
    <Bar v-if="chartData" :data="chartData" :options="chartOptions" style="max-height: 280px" />
    <div v-else class="empty-state">No activity data yet.</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Bar } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend
} from 'chart.js'
import type { ActivityDataPoint } from '@/types'

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)

const props = defineProps<{
  data: ActivityDataPoint[]
}>()

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

const chartData = computed(() => {
  if (!props.data.length) return null
  return {
    labels: props.data.map(d => `${MONTHS[d.month - 1]} ${d.year}`),
    datasets: [{
      label: 'Albums listened',
      data: props.data.map(d => d.count),
      backgroundColor: 'rgba(99, 102, 241, 0.7)',
      borderColor: 'rgb(99, 102, 241)',
      borderWidth: 1,
      borderRadius: 4
    }]
  }
})

const chartOptions = {
  responsive: true,
  plugins: {
    legend: { display: false }
  },
  scales: {
    y: {
      beginAtZero: true,
      ticks: { stepSize: 1 }
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
