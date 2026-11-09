import { describe, it, expect } from 'vitest'
import { matchesSearch } from '../src/ingredient-search'
import { parseNonnegative } from '../src/i18n'
describe('ingredient entry', () => {
  it('finds compound spellings without changing food identity', () => {
    expect(matchesSearch('Hafer Flocken Oat flakes', 'haferflocken')).toBe(true)
    expect(matchesSearch('Haferkorn Oats', 'haferflocken')).toBe(false)
    expect(matchesSearch('Weizenmehl Type 550', 'weizen 550')).toBe(true)
  })
  it('preserves explicit zero and locale precision for declared constituents', () => {
    expect(parseNonnegative('0', 'en')).toBe('0')
    expect(parseNonnegative('0,000000', 'de')).toBe('0.000000')
    expect(parseNonnegative('2,500000', 'de')).toBe('2.500000')
    for (const input of ['-1', '1e3', '0.0', '1,0000001'])
      expect(parseNonnegative(input, 'de')).toBeNull()
  })
})
