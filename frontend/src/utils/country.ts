/**
 * Turning stored country values into something readable.
 *
 * The stored value is usually an ISO 3166-1 alpha-2 code, because that is what
 * MusicBrainz returns. It is not guaranteed to be: the country field is editable, and the
 * resolver falls back to an artist's area name, which is a full name like "England". So
 * everything here degrades to showing the raw string rather than guessing.
 */

const ALPHA2 = /^[A-Za-z]{2}$/

// Intl.DisplayNames does the whole ISO 3166 table for us, in the platform, with no
// shipped data. Constructed once; guarded because a webview without it should show codes
// rather than crash the view.
const regionNames: Intl.DisplayNames | null = (() => {
  try {
    return new Intl.DisplayNames(['en'], { type: 'region', fallback: 'code' })
  } catch {
    return null
  }
})()

/**
 * The flag emoji for a country code, built from regional indicator symbols.
 *
 * Returns null for anything that is not a two-letter code, and for the UN's user-assigned
 * range, which has no flag. Whether the glyph actually renders depends on the system
 * having an emoji font; without one it degrades to two boxed letters, which still reads
 * as the country code.
 */
export function countryFlag(value: string | null | undefined): string | null {
  if (!value || !ALPHA2.test(value)) return null
  const code = value.toUpperCase()
  // XA-XZ and ZZ are user-assigned and have no flag. MusicBrainz uses XW for "worldwide".
  if (code.startsWith('X') || code === 'ZZ') return null
  return String.fromCodePoint(
    ...[...code].map((c) => 0x1f1e6 + c.charCodeAt(0) - 65)
  )
}

/**
 * A readable name for a country value. Codes become names; anything already a name, or
 * unrecognised, comes back untouched.
 */
export function countryName(value: string | null | undefined): string {
  if (!value) return ''
  const trimmed = value.trim()
  if (!ALPHA2.test(trimmed)) return trimmed
  const code = trimmed.toUpperCase()
  const resolved = regionNames?.of(code)
  // `fallback: 'code'` hands back the input when it knows nothing, so an unknown code
  // shows as the code rather than as an empty label.
  return resolved && resolved !== code ? resolved : code
}
