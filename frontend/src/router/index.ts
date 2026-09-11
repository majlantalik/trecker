import { createRouter, createWebHistory } from 'vue-router'

// History mode is kept deliberately. Tauri's asset protocol falls back to index.html for
// unmatched paths, verified against a production build in the Phase 0 spike, so the usual
// advice to switch to hash history for Tauri does not apply here.
const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/queue'
    },
    {
      path: '/queue',
      name: 'queue',
      component: () => import('@/views/QueueView.vue')
    },
    {
      path: '/library',
      name: 'library',
      component: () => import('@/views/LibraryView.vue')
    },
    {
      path: '/stats',
      name: 'stats',
      component: () => import('@/views/StatsView.vue')
    },
    {
      path: '/entry/:id',
      name: 'entry',
      component: () => import('@/views/EntryView.vue'),
      props: true
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue')
    }
  ]
})

export default router
