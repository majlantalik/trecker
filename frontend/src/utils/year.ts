/**
 * What the year filter's text field holds: digits only, four at most, and a year once all
 * four are there.
 */
export function parseYearInput(raw: string): { text: string; year: number | undefined } {
  const text = raw.replace(/\D/g, '').slice(0, 4)
  return { text, year: text.length === 4 ? Number(text) : undefined }
}
