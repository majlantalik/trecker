import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import axios from 'axios'
import MockAdapter from 'axios-mock-adapter'

// Mock the dynamic imports used in the interceptor on refresh failure
const mockReset = vi.fn()
const mockPush = vi.fn()

vi.mock('@/stores/auth', () => ({
  useAuthStore: vi.fn(() => ({ $reset: mockReset }))
}))

vi.mock('@/router', () => ({
  default: { push: mockPush }
}))

// Import api AFTER mocks are set up so the interceptor captures the mocked modules
const { default: api } = await import('./axios')

describe('axios interceptor', () => {
  let mock: MockAdapter

  beforeEach(() => {
    mock = new MockAdapter(api)
    mockReset.mockClear()
    mockPush.mockClear()
  })

  afterEach(() => {
    mock.restore()
  })

  it('single 401 triggers one refresh and retries original request', async () => {
    let releasesCallCount = 0

    mock.onGet('/releases').reply(() => {
      releasesCallCount++
      // First call 401, retry succeeds
      return releasesCallCount === 1 ? [401] : [200, []]
    })
    mock.onPost('/auth/refresh').reply(200)

    const result = await api.get('/releases')
    expect(result.status).toBe(200)
  })

  it('multiple concurrent 401s trigger only one refresh call', async () => {
    let refreshCallCount = 0
    let releasesCallCount = 0

    mock.onGet('/releases').reply(() => {
      releasesCallCount++
      // First 3 calls get 401 (the concurrent originals), rest get 200 (the retries)
      return releasesCallCount <= 3 ? [401] : [200, []]
    })
    mock.onPost('/auth/refresh').reply(() => {
      refreshCallCount++
      return [200]
    })

    await Promise.all([
      api.get('/releases'),
      api.get('/releases'),
      api.get('/releases')
    ])

    expect(refreshCallCount).toBe(1)
  })

  it('refresh failure calls authStore.$reset and navigates to /login', async () => {
    mock.onGet('/releases').reply(401)
    mock.onPost('/auth/refresh').reply(401)

    await expect(api.get('/releases')).rejects.toThrow()

    expect(mockReset).toHaveBeenCalledOnce()
    expect(mockPush).toHaveBeenCalledWith('/login')
  })

  it('401 on /auth/login does not trigger refresh', async () => {
    let refreshCalled = false
    mock.onPost('/auth/login').reply(401)
    mock.onPost('/auth/refresh').reply(() => {
      refreshCalled = true
      return [200]
    })

    await expect(api.post('/auth/login', {})).rejects.toThrow()

    expect(refreshCalled).toBe(false)
  })

  it('401 on /auth/refresh does not trigger infinite refresh loop', async () => {
    let refreshCallCount = 0
    mock.onPost('/auth/refresh').reply(() => {
      refreshCallCount++
      return [401]
    })

    await expect(api.post('/auth/refresh')).rejects.toThrow()

    // Should have been called exactly once — no looping
    expect(refreshCallCount).toBe(1)
  })
})
