import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { authApi } from '@/api/auth'
import type { UserDto, LoginRequest, RegisterRequest } from '@/types'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<UserDto | null>(null)
  const loading = ref(false)

  const isAuthenticated = computed(() => user.value !== null)

  async function fetchMe() {
    try {
      user.value = await authApi.me()
    } catch {
      // 401 = no valid session — silently ignore
      user.value = null
    }
  }

  async function login(req: LoginRequest) {
    loading.value = true
    try {
      user.value = await authApi.login(req)
    } finally {
      loading.value = false
    }
  }

  async function register(req: RegisterRequest) {
    loading.value = true
    try {
      user.value = await authApi.register(req)
    } finally {
      loading.value = false
    }
  }

  async function logout() {
    await authApi.logout()
    user.value = null
  }

  function $reset() {
    user.value = null
    loading.value = false
  }

  return { user, loading, isAuthenticated, fetchMe, login, register, logout, $reset }
})
