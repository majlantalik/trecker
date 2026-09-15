/**
 * The rules behind the genre chips: splitting typed text, matching the genres already in
 * the library, and suggesting from them.
 */

const key = (genre: string) => genre.trim().toLocaleLowerCase()

/** "indie rock, britpop," becomes ["indie rock", "britpop"]. */
export function splitGenres(text: string): string[] {
  return text
    .split(/[,;\n]/)
    .map((g) => g.trim().replace(/\s+/g, ' '))
    .filter(Boolean)
}

/**
 * Adds genres to a list, once each whatever their case. A genre the library already has is
 * written the way the library writes it, so "Indie Rock" typed today does not become a
 * second genre beside "indie rock".
 */
export function addGenres(current: string[], incoming: string[], known: string[]): string[] {
  const canonical = new Map(known.map((g) => [key(g), g]))
  const taken = new Set(current.map(key))
  const next = [...current]
  for (const raw of incoming) {
    const genre = raw.trim().replace(/\s+/g, ' ')
    if (!genre || taken.has(key(genre))) continue
    taken.add(key(genre))
    next.push(canonical.get(key(genre)) ?? genre)
  }
  return next
}

/**
 * Genres from the library that match what is being typed and are not chosen yet. Those that
 * start with the text come first, then those with a word that does, then any other match.
 */
export function suggestGenres(query: string, known: string[], selected: string[], limit = 8): string[] {
  const q = key(query)
  const taken = new Set(selected.map(key))
  const rank = (g: string) => {
    const k = key(g)
    if (k.startsWith(q)) return 0
    if (k.split(/[\s-]/).some((word) => word.startsWith(q))) return 1
    return 2
  }
  return known
    .filter((g) => !taken.has(key(g)) && (!q || key(g).includes(q)))
    .sort((a, b) => rank(a) - rank(b) || a.localeCompare(b))
    .slice(0, limit)
}
