import { describe, it, expect } from 'vitest'
import { addGenres, splitGenres, suggestGenres } from './genres'

describe('splitGenres', () => {
  it('splits on commas and tidies the spaces', () => {
    expect(splitGenres('indie rock, britpop,')).toEqual(['indie rock', 'britpop'])
    expect(splitGenres('  post   rock ;shoegaze\ndream pop')).toEqual(['post rock', 'shoegaze', 'dream pop'])
    expect(splitGenres(' , ')).toEqual([])
  })
})

describe('addGenres', () => {
  const known = ['britpop', 'indie rock', 'shoegaze']

  it('adds each genre once, whatever its case', () => {
    expect(addGenres(['indie rock'], ['Indie Rock', 'britpop', 'BRITPOP'], known)).toEqual(['indie rock', 'britpop'])
  })

  it('writes a known genre the way the library does, and a new one as typed', () => {
    expect(addGenres([], ['Shoegaze', 'Madchester'], known)).toEqual(['shoegaze', 'Madchester'])
  })

  it('keeps the chips already there and in order', () => {
    expect(addGenres(['b', 'a'], ['c'], [])).toEqual(['b', 'a', 'c'])
    expect(addGenres(['a'], ['  '], [])).toEqual(['a'])
  })
})

describe('suggestGenres', () => {
  const known = ['alternative rock', 'indie pop', 'indie rock', 'post-rock', 'rock', 'art rock']

  it('puts genres starting with the text first, then word starts, then the rest', () => {
    expect(suggestGenres('ro', known, [])).toEqual(['rock', 'alternative rock', 'art rock', 'indie rock', 'post-rock'])
    expect(suggestGenres('ind', known, [])).toEqual(['indie pop', 'indie rock'])
  })

  it('leaves out genres already chosen, whatever their case', () => {
    expect(suggestGenres('indie', known, ['Indie Pop'])).toEqual(['indie rock'])
  })

  it('offers everything for an empty query, up to the limit', () => {
    expect(suggestGenres('', known, [], 3)).toHaveLength(3)
  })
})
