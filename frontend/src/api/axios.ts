import axios from 'axios'
import type { AxiosRequestConfig } from 'axios'

const api = axios.create({
  baseURL: '/api',
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json'
  }
})

let isRefreshing = false
let failedQueue: Array<{ resolve: (value: unknown) => void; reject: (reason?: unknown) => void; config: AxiosRequestConfig }> = []

function processQueue(error: unknown) {
  failedQueue.forEach(({ resolve, reject, config }) => {
    if (error) {
      reject(error)
    } else {
      resolve(api(config))
    }
  })
  failedQueue = []
}

api.interceptors.response.use(
  response => response,
  async error => {
    const originalRequest = error.config as AxiosRequestConfig & { _retry?: boolean }

    const isAuthEndpoint = originalRequest.url?.includes('/auth/refresh') ||
      originalRequest.url?.includes('/auth/login')

    if (error.response?.status === 401 && !isAuthEndpoint && !originalRequest._retry) {
      if (isRefreshing) {
        return new Promise((resolve, reject) => {
          failedQueue.push({ resolve, reject, config: originalRequest })
        })
      }

      originalRequest._retry = true
      isRefreshing = true

      try {
        await api.post('/auth/refresh')
        processQueue(null)
        return api(originalRequest)
      } catch (refreshError) {
        processQueue(refreshError)
        // Dynamic import to avoid circular dependency (auth store imports api)
        const { useAuthStore } = await import('@/stores/auth')
        const router = await import('@/router').then(m => m.default)
        const authStore = useAuthStore()
        authStore.$reset()
        router.push('/login')
        return Promise.reject(refreshError)
      } finally {
        isRefreshing = false
      }
    }

    if (!isAuthEndpoint || error.response?.status !== 401) {
      console.error('API Error:', error.response?.data || error.message)
    }
    return Promise.reject(error)
  }
)

export default api
