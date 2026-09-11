import { describe, it, expect, beforeEach, vi } from 'vitest'

// Replaces the old axios interceptor test. What matters now is the command contract:
// the Rust side in src-tauri/src/commands.rs must keep these names and argument keys, so
// a rename on either side fails here rather than silently at runtime.
const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke }))

const { releasesApi } = await import('./releases')
const { genresApi } = await import('./genres')
const { statsApi } = await import('./stats')

beforeEach(() => {
  invoke.mockReset()
  invoke.mockResolvedValue(undefined)
})

describe('releases commands', () => {
  it('resolve passes the request through', async () => {
    await releasesApi.resolve({ query: 'Slint - Spiderland' })
    expect(invoke).toHaveBeenCalledWith('releases_resolve', {
      request: { query: 'Slint - Spiderland' }
    })
  })

  it('create passes the request through', async () => {
    await releasesApi.create({ artist: 'Duster', title: 'Stratosphere' })
    expect(invoke).toHaveBeenCalledWith('releases_create', {
      request: { artist: 'Duster', title: 'Stratosphere' }
    })
  })

  it('getAll sends filter params under "params"', async () => {
    await releasesApi.getAll({ status: 'QUEUED', size: 5 })
    expect(invoke).toHaveBeenCalledWith('releases_list', {
      params: { status: 'QUEUED', size: 5 }
    })
  })

  it('getAll defaults to an empty param object', async () => {
    await releasesApi.getAll()
    expect(invoke).toHaveBeenCalledWith('releases_list', { params: {} })
  })

  it('getRandom takes no arguments', async () => {
    await releasesApi.getRandom()
    expect(invoke).toHaveBeenCalledWith('releases_random')
  })

  it('getById sends the id', async () => {
    await releasesApi.getById('abc')
    expect(invoke).toHaveBeenCalledWith('releases_get', { id: 'abc' })
  })

  it('update sends id and request separately', async () => {
    await releasesApi.update('abc', { rating: 4.5 })
    expect(invoke).toHaveBeenCalledWith('releases_update', {
      id: 'abc',
      request: { rating: 4.5 }
    })
  })

  it('delete sends the id', async () => {
    await releasesApi.delete('abc')
    expect(invoke).toHaveBeenCalledWith('releases_delete', { id: 'abc' })
  })

  it('refreshMetadata sends the id', async () => {
    await releasesApi.refreshMetadata('abc')
    expect(invoke).toHaveBeenCalledWith('releases_refresh_metadata', { id: 'abc' })
  })

  it('searchCatalog applies the default limit', async () => {
    await releasesApi.searchCatalog('slint')
    expect(invoke).toHaveBeenCalledWith('releases_search_catalog', { q: 'slint', limit: 10 })
  })
})

describe('genres commands', () => {
  it('getAll takes no arguments', async () => {
    await genresApi.getAll()
    expect(invoke).toHaveBeenCalledWith('genres_list')
  })
})

describe('stats commands', () => {
  it('maps each call to its command', async () => {
    await statsApi.getActivity()
    expect(invoke).toHaveBeenCalledWith('stats_activity')

    await statsApi.getByGenre()
    expect(invoke).toHaveBeenCalledWith('stats_by_genre')

    await statsApi.getByCountry()
    expect(invoke).toHaveBeenCalledWith('stats_by_country')
  })

  it('topRated applies the default limit', async () => {
    await statsApi.getTopRated()
    expect(invoke).toHaveBeenCalledWith('stats_top_rated', { limit: 25 })
  })

  it('yearEnd forwards an absent year as undefined', async () => {
    await statsApi.getYearEnd()
    expect(invoke).toHaveBeenCalledWith('stats_year_end', { year: undefined })
  })
})

describe('error propagation', () => {
  it('surfaces the AppError shape the Rust side serializes', async () => {
    invoke.mockRejectedValue({ code: 'NOT_FOUND', message: 'release not found' })
    await expect(releasesApi.getById('missing')).rejects.toMatchObject({
      code: 'NOT_FOUND'
    })
  })
})
