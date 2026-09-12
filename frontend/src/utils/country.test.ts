import { describe, it, expect } from 'vitest'
import { countryFlag, countryName, COUNTRY_CODES, countryOptions } from './country'

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

describe('COUNTRY_CODES', () => {
  it('holds the 249 ISO 3166-1 countries plus Kosovo, once each', () => {
    expect(COUNTRY_CODES).toHaveLength(250)
    expect(new Set(COUNTRY_CODES).size).toBe(250)
    expect(COUNTRY_CODES).toContain('XK')
  })

  it('holds only two-letter codes the platform can name', () => {
    for (const code of COUNTRY_CODES) {
      expect(code).toMatch(/^[A-Z]{2}$/)
      expect(countryName(code), code).not.toBe(code)
    }
  })

  it('leaves out historical states and reserved codes', () => {
    // The platform still names these, which is why the list is not generated from it.
    for (const code of ['UK', 'SU', 'YU', 'RH', 'EU', 'UN', 'ZZ']) {
      expect(COUNTRY_CODES).not.toContain(code)
    }
  })
})

describe('countryOptions', () => {
  it('sorts by name, not by code', () => {
    const labels = countryOptions().map((o) => o.label)
    expect(labels).toEqual([...labels].sort((a, b) => a.localeCompare(b)))
    // Germany is DE and Denmark is DK, but by name Denmark comes first.
    expect(labels.indexOf('Denmark')).toBeLessThan(labels.indexOf('Germany'))
  })

  it('does not duplicate a current value that is already on the list', () => {
    expect(countryOptions('US')).toHaveLength(COUNTRY_CODES.length)
    expect(countryOptions('us')).toHaveLength(COUNTRY_CODES.length)
  })

  it('keeps a current value that is not on the list, first', () => {
    // From MusicBrainz: a dissolved state's code, and an area name with no ISO code.
    expect(countryOptions('SU')[0].value).toBe('SU')
    expect(countryOptions('England')[0]).toEqual({ value: 'England', label: 'England' })
    expect(countryOptions('England')).toHaveLength(COUNTRY_CODES.length + 1)
  })
})
