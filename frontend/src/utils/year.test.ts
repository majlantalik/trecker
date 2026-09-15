import { describe, it, expect } from 'vitest'
import { parseYearInput } from './year'

describe('parseYearInput', () => {
  it('is a year once four digits are in', () => {
    expect(parseYearInput('2025')).toEqual({ text: '2025', year: 2025 })
    expect(parseYearInput('202')).toEqual({ text: '202', year: undefined })
    expect(parseYearInput('')).toEqual({ text: '', year: undefined })
  })

  it('keeps digits only, and no more than four', () => {
    expect(parseYearInput('2,025')).toEqual({ text: '2025', year: 2025 })
    expect(parseYearInput('20256')).toEqual({ text: '2025', year: 2025 })
    expect(parseYearInput('y2k')).toEqual({ text: '2', year: undefined })
  })
})
