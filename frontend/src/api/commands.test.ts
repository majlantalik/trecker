import { describe, it, expect, beforeEach, vi } from 'vitest'

// Replaces the old axios interceptor test. What matters now is the command contract:
// the Rust side in src-tauri/src/commands.rs must keep these names and argument keys, so
// a rename on either side fails here rather than silently at runtime.
const invoke = vi.fn()
const listen = vi.fn()
vi.mock('@tauri-apps/api/event', () => ({ listen }))
const convertFileSrc = vi.fn((path: string, protocol: string) => `${protocol}://localhost/${encodeURIComponent(path)}`)
vi.mock('@tauri-apps/api/core', () => ({ invoke, convertFileSrc }))

const { releasesApi } = await import('./releases')
const { genresApi, countriesApi } = await import('./genres')
const { statsApi } = await import('./stats')
const { infoApi } = await import('./info')
const { libraryApi } = await import('./library')
const { cacheApi, coverSrc } = await import('./cache')
const { settingsApi } = await import('./settings')
const { appApi } = await import('./app')
const { artistsApi } = await import('./artists')

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

  it('search sends the typed text as "query"', async () => {
    await releasesApi.search('avenged sevenfold nightmare')
    expect(invoke).toHaveBeenCalledWith('releases_search', { query: 'avenged sevenfold nightmare' })
  })

  it('lookup sends the id as "releaseGroupId"', async () => {
    // Tauri maps the camelCase key onto the Rust argument `release_group_id`.
    await releasesApi.lookup('180560ee-2d9d-33cf-8de7-cdaaba610739')
    expect(invoke).toHaveBeenCalledWith('releases_lookup', {
      releaseGroupId: '180560ee-2d9d-33cf-8de7-cdaaba610739'
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

  it('unlinked takes no arguments', async () => {
    await releasesApi.unlinked()
    expect(invoke).toHaveBeenCalledWith('releases_unlinked')
  })

  it('link sends the release and the chosen album', async () => {
    await releasesApi.link('abc', '180560ee-2d9d-33cf-8de7-cdaaba610739')
    expect(invoke).toHaveBeenCalledWith('releases_link', {
      id: 'abc',
      releaseGroupId: '180560ee-2d9d-33cf-8de7-cdaaba610739'
    })
  })

  it('searchCatalog applies the default limit', async () => {
    await releasesApi.searchCatalog('slint')
    expect(invoke).toHaveBeenCalledWith('releases_search_catalog', { q: 'slint', limit: 10 })
  })
})

describe('artists commands', () => {
  it('search sends the typed text as "query"', async () => {
    await artistsApi.search('avenged sevenfold')
    expect(invoke).toHaveBeenCalledWith('artists_search', { query: 'avenged sevenfold' })
  })

  it('add sends the request as "request"', async () => {
    await artistsApi.add({ musicbrainzArtistId: 'mb', note: 'from a friend' })
    expect(invoke).toHaveBeenCalledWith('artists_add', {
      request: { musicbrainzArtistId: 'mb', note: 'from a friend' }
    })
  })

  it('list takes no arguments', async () => {
    await artistsApi.list()
    expect(invoke).toHaveBeenCalledWith('artists_list')
  })

  it('get, delete and discography send "id"', async () => {
    await artistsApi.get('a1')
    expect(invoke).toHaveBeenCalledWith('artists_get', { id: 'a1' })
    await artistsApi.delete('a1')
    expect(invoke).toHaveBeenCalledWith('artists_delete', { id: 'a1' })
    await artistsApi.discography('a1')
    expect(invoke).toHaveBeenCalledWith('artists_discography', { id: 'a1' })
  })

  it('update sends "id" and "request"', async () => {
    await artistsApi.update('a1', { status: 'CHECKED', verdict: 'LIKED' })
    expect(invoke).toHaveBeenCalledWith('artists_update', {
      id: 'a1',
      request: { status: 'CHECKED', verdict: 'LIKED' }
    })
  })
})

describe('genres commands', () => {
  it('getAll takes no arguments', async () => {
    await genresApi.getAll()
    expect(invoke).toHaveBeenCalledWith('genres_list')
  })
})

describe('info commands', () => {
  it('dbInfo takes no arguments', async () => {
    await infoApi.dbInfo()
    expect(invoke).toHaveBeenCalledWith('info_db')
  })
})

describe('countries commands', () => {
  it('getAll takes no arguments', async () => {
    await countriesApi.getAll()
    expect(invoke).toHaveBeenCalledWith('countries_list')
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

  it('yearEnd forwards an absent year as undefined and ranks by listen date', async () => {
    await statsApi.getYearEnd()
    expect(invoke).toHaveBeenCalledWith('stats_year_end', { year: undefined, by: 'listened' })
  })

  it('yearEnd sends the year and the basis', async () => {
    await statsApi.getYearEnd(2022, 'released')
    expect(invoke).toHaveBeenCalledWith('stats_year_end', { year: 2022, by: 'released' })
  })
})

describe('library commands', () => {
  it('export sends the path and the format', async () => {
    await libraryApi.export('/home/me/library.json', 'json')
    expect(invoke).toHaveBeenCalledWith('library_export', {
      path: '/home/me/library.json',
      format: 'json'
    })
  })

  it('import sends the path and the mode', async () => {
    await libraryApi.import('/home/me/library.csv', 'overwrite')
    expect(invoke).toHaveBeenCalledWith('library_import', {
      path: '/home/me/library.csv',
      mode: 'overwrite'
    })
  })
})

describe('cache commands', () => {
  it('maps each call to its command, with no arguments', async () => {
    await cacheApi.covers()
    expect(invoke).toHaveBeenCalledWith('cache_covers_info')

    await cacheApi.clearCovers()
    expect(invoke).toHaveBeenCalledWith('cache_covers_clear')

    await cacheApi.clearWebview()
    expect(invoke).toHaveBeenCalledWith('cache_webview_clear')
  })
})

describe('coverSrc', () => {
  it('routes a stored URL through the cover protocol', () => {
    // The scheme must match `covers::SCHEME` in src-tauri/src/covers.rs.
    const url = 'https://coverartarchive.org/release/x/1-250.jpg'
    expect(coverSrc(url)).toBe(`cover://localhost/${encodeURIComponent(url)}`)
    expect(convertFileSrc).toHaveBeenCalledWith(url, 'cover')
  })

  it('gives no address for a release without a cover', () => {
    expect(coverSrc(null)).toBeUndefined()
    expect(coverSrc(undefined)).toBeUndefined()
    expect(coverSrc('')).toBeUndefined()
  })
})

describe('settings commands', () => {
  it('get takes no arguments', async () => {
    await settingsApi.get()
    expect(invoke).toHaveBeenCalledWith('settings_get')
  })

  it('update sends the settings as "request"', async () => {
    await settingsApi.update({ closeAction: 'tray', queueSort: 'oldest', openLinksIn: 'app' })
    expect(invoke).toHaveBeenCalledWith('settings_update', { request: { closeAction: 'tray', queueSort: 'oldest', openLinksIn: 'app' } })
  })
})

describe('app commands', () => {
  it('maps each call to its command, with no arguments', async () => {
    await appApi.takeLaunchAction()
    expect(invoke).toHaveBeenCalledWith('app_take_launch_action')

    await appApi.quickAddCommand()
    expect(invoke).toHaveBeenCalledWith('app_quick_add_command')
  })

  it('listens for the event the Rust side emits', async () => {
    // Must match QUICK_ADD_EVENT in src-tauri/src/desktop.rs.
    const callback = vi.fn()
    await appApi.onQuickAdd(callback)
    expect(listen).toHaveBeenCalledWith('quick-add', expect.any(Function))
    listen.mock.calls[0][1]({ payload: null })
    expect(callback).toHaveBeenCalledOnce()
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
