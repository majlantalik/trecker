import { describe, it, expect } from 'vitest'

// Every .vue file's source, read as text.
const sources = import.meta.glob('../**/*.vue', { query: '?raw', import: 'default', eager: true }) as Record<string, string>

describe('dialog styling', () => {
  // Each opening <Dialog> or <ConfirmDialog> tag. `(?<!=)>` skips the ">" of an arrow
  // function inside an attribute, so the match ends at the tag's own closing bracket.
  const tags = Object.entries(sources).flatMap(([file, text]) =>
    [...text.matchAll(/<(?:Dialog|ConfirmDialog)\b[\s\S]*?(?<!=)>/g)].map((m) => ({ file, tag: m[0] }))
  )

  it('finds the dialogs it is guarding', () => {
    // If the pattern stopped matching, the test below would pass by checking nothing.
    expect(tags.length).toBeGreaterThanOrEqual(8)
  })

  it("gives every dialog the app's dialog style", () => {
    // Without the class a dialog renders on PrimeVue's grey default, which looks like a
    // different app. Nothing else fails when one is added without it.
    const unstyled = tags.filter(({ tag }) => !tag.includes('tk-dialog')).map(({ file }) => file)
    expect(unstyled).toEqual([])
  })
})
