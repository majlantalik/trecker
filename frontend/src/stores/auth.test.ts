import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import type { UserDto } from '@/types'

const mockUser: UserDto = { id: 'user-1', email: 'user@test.com' }

const mockAuthApi = {
  me: vi.fn(),
  login: vi.fn(),
  register: vi.fn(),
  logout: vi.fn()
}

vi.mock('@/api/auth', () => ({
  authApi: mockAuthApi
}))

const { useAuthStore } = await import('./auth')

describe('auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('fetchMe sets user on success', async () => {
    mockAuthApi.me.mockResolvedValue(mockUser)

    const store = useAuthStore()
    await store.fetchMe()

    expect(store.user).toEqual(mockUser)
  })

  it('fetchMe stays null on 401 (silent failure)', async () => {
    mockAuthApi.me.mockRejectedValue({ response: { status: 401 } })

    const store = useAuthStore()
    await store.fetchMe()

    expect(store.user).toBeNull()
  })

  it('login sets user and returns it', async () => {
    mockAuthApi.login.mockResolvedValue(mockUser)

    const store = useAuthStore()
    await store.login({ email: 'user@test.com', password: 'pass' })

    expect(store.user).toEqual(mockUser)
  })

  it('logout calls api and resets user', async () => {
    mockAuthApi.logout.mockResolvedValue(undefined)

    const store = useAuthStore()
    store.user = mockUser
    await store.logout()

    expect(store.user).toBeNull()
    expect(mockAuthApi.logout).toHaveBeenCalledOnce()
  })

  it('$reset clears user to null', () => {
    const store = useAuthStore()
    store.user = mockUser
    store.$reset()

    expect(store.user).toBeNull()
    expect(store.loading).toBe(false)
  })

  it('isAuthenticated is true when user is set', () => {
    const store = useAuthStore()
    store.user = mockUser

    expect(store.isAuthenticated).toBe(true)
  })

  it('isAuthenticated is false when user is null', () => {
    const store = useAuthStore()
    expect(store.isAuthenticated).toBe(false)
  })
})
