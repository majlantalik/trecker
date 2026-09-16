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
    ...[...code].map((c) => 0x1f1e6 + c.codePointAt(0)! - 65)
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

/**
 * Every country a release can be assigned in the editors.
 *
 * The 249 officially assigned ISO 3166-1 alpha-2 codes, plus `XK` for Kosovo, which is not
 * in ISO 3166 but is the code MusicBrainz uses. Kept as a list because the platform can
 * name a region code but cannot enumerate them, and generating the list from its names
 * also picks up historical states such as Rhodesia and reserved codes such as `UK`.
 */
export const COUNTRY_CODES: readonly string[] = [
  'AD', 'AE', 'AF', 'AG', 'AI', 'AL', 'AM', 'AO', 'AQ', 'AR', 'AS', 'AT', 'AU', 'AW', 'AX',
  'AZ', 'BA', 'BB', 'BD', 'BE', 'BF', 'BG', 'BH', 'BI', 'BJ', 'BL', 'BM', 'BN', 'BO', 'BQ',
  'BR', 'BS', 'BT', 'BV', 'BW', 'BY', 'BZ', 'CA', 'CC', 'CD', 'CF', 'CG', 'CH', 'CI', 'CK',
  'CL', 'CM', 'CN', 'CO', 'CR', 'CU', 'CV', 'CW', 'CX', 'CY', 'CZ', 'DE', 'DJ', 'DK', 'DM',
  'DO', 'DZ', 'EC', 'EE', 'EG', 'EH', 'ER', 'ES', 'ET', 'FI', 'FJ', 'FK', 'FM', 'FO', 'FR',
  'GA', 'GB', 'GD', 'GE', 'GF', 'GG', 'GH', 'GI', 'GL', 'GM', 'GN', 'GP', 'GQ', 'GR', 'GS',
  'GT', 'GU', 'GW', 'GY', 'HK', 'HM', 'HN', 'HR', 'HT', 'HU', 'ID', 'IE', 'IL', 'IM', 'IN',
  'IO', 'IQ', 'IR', 'IS', 'IT', 'JE', 'JM', 'JO', 'JP', 'KE', 'KG', 'KH', 'KI', 'KM', 'KN',
  'KP', 'KR', 'KW', 'KY', 'KZ', 'LA', 'LB', 'LC', 'LI', 'LK', 'LR', 'LS', 'LT', 'LU', 'LV',
  'LY', 'MA', 'MC', 'MD', 'ME', 'MF', 'MG', 'MH', 'MK', 'ML', 'MM', 'MN', 'MO', 'MP', 'MQ',
  'MR', 'MS', 'MT', 'MU', 'MV', 'MW', 'MX', 'MY', 'MZ', 'NA', 'NC', 'NE', 'NF', 'NG', 'NI',
  'NL', 'NO', 'NP', 'NR', 'NU', 'NZ', 'OM', 'PA', 'PE', 'PF', 'PG', 'PH', 'PK', 'PL', 'PM',
  'PN', 'PR', 'PS', 'PT', 'PW', 'PY', 'QA', 'RE', 'RO', 'RS', 'RU', 'RW', 'SA', 'SB', 'SC',
  'SD', 'SE', 'SG', 'SH', 'SI', 'SJ', 'SK', 'SL', 'SM', 'SN', 'SO', 'SR', 'SS', 'ST', 'SV',
  'SX', 'SY', 'SZ', 'TC', 'TD', 'TF', 'TG', 'TH', 'TJ', 'TK', 'TL', 'TM', 'TN', 'TO', 'TR',
  'TT', 'TV', 'TW', 'TZ', 'UA', 'UG', 'UM', 'US', 'UY', 'UZ', 'VA', 'VC', 'VE', 'VG', 'VI',
  'VN', 'VU', 'WF', 'WS', 'YE', 'YT', 'ZA', 'ZM', 'ZW',
  'XK'
]

export interface CountryOption {
  value: string
  label: string
}

/**
 * The options for a country picker, sorted by name.
 *
 * A value that is not on the list, such as a historical code like `SU` from MusicBrainz or
 * an area name like "England", is kept as the first option. Otherwise opening the picker
 * would show nothing selected, and the only way out would be to replace a value that was
 * never wrong.
 */
export function countryOptions(current?: string | null): CountryOption[] {
  const options = COUNTRY_CODES.map((value) => ({ value, label: countryName(value) })).sort(
    (a, b) => a.label.localeCompare(b.label)
  )
  const trimmed = current?.trim()
  if (!trimmed) return options
  if (COUNTRY_CODES.includes(trimmed.toUpperCase())) return options
  return [{ value: trimmed, label: countryName(trimmed) }, ...options]
}
