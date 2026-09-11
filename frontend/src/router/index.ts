import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/queue'
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
      meta: { public: true }
    },
    {
      path: '/register',
      name: 'register',
      component: () => import('@/views/RegisterView.vue'),
      meta: { public: true }
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
      path: '/profile',
      name: 'profile',
      component: () => import('@/views/ProfileView.vue')
    },
    {
      // Phase 0 spike. Not public, so it renders inside AppLayout and exercises the
      // sticky header, but exempt from the auth guard since the spike has no backend.
      // Removed in Phase 1 along with the auth guard itself.
      path: '/spike',
      name: 'spike',
      component: () => import('@/views/SpikeView.vue'),
      meta: { spike: true }
    }
  ]
})

router.beforeEach(async (to) => {
  if (to.meta.spike) return

  const authStore = useAuthStore()

  if (!to.meta.public && !authStore.isAuthenticated) {
    return { path: '/login', query: { redirect: to.fullPath } }
  }

  if (to.meta.public && authStore.isAuthenticated) {
    return { path: '/queue' }
  }
})

export default router
