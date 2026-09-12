import { describe, it, expect } from 'vitest'
import { countryFlag, countryName } from './country'

describe('countryFlag', () => {
  it('builds flags from regional indicator symbols', () => {
    expect(countryFlag('US')).toBe('🇺🇸')
    expect(countryFlag('GB')).toBe('🇬🇧')
    expect(countryFlag('JP')).toBe('🇯🇵')
  })

  it('accepts lowercase, since the field is editable', () => {
    expect(countryFlag('us')).toBe(countryFlag('US'))
  })

  it('has no flag for user-assigned codes', () => {
    // MusicBrainz uses XW for "worldwide" and XE for Europe. Neither has a flag, and
    // rendering one would be inventing a country.
    expect(countryFlag('XW')).toBeNull()
    expect(countryFlag('XE')).toBeNull()
    expect(countryFlag('ZZ')).toBeNull()
  })

  it('has no flag for anything that is not a two-letter code', () => {
    // The resolver can store an artist's area name, which is a full name.
    expect(countryFlag('England')).toBeNull()
    expect(countryFlag('USA')).toBeNull()
    expect(countryFlag('U')).toBeNull()
    expect(countryFlag('12')).toBeNull()
    expect(countryFlag('')).toBeNull()
    expect(countryFlag(null)).toBeNull()
    expect(countryFlag(undefined)).toBeNull()
  })
})

describe('countryName', () => {
  it('expands ISO codes to names', () => {
    expect(countryName('US')).toBe('United States')
    expect(countryName('JP')).toBe('Japan')
    expect(countryName('gb')).toBe('United Kingdom')
  })

  it('leaves values that are already names alone', () => {
    expect(countryName('England')).toBe('England')
    expect(countryName('Scotland')).toBe('Scotland')
  })

  it('falls back to the code when it recognises nothing', () => {
    // Better a bare code than an empty cell.
    expect(countryName('XW')).toBe('XW')
    expect(countryName('QQ')).toBe('QQ')
  })

  it('is empty for empty input', () => {
    expect(countryName('')).toBe('')
    expect(countryName(null)).toBe('')
    expect(countryName(undefined)).toBe('')
  })

  it('trims surrounding whitespace', () => {
    expect(countryName('  US  ')).toBe('United States')
  })
})
