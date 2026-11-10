import { describe, it, expect } from 'vitest'
import { canonicalGtin } from '../src/shopping-api'
import {
  locale,
  normalizeLocale,
  servingCount,
  calendarDate,
  mealName,
  compactExact,
  shoppingUnit,
} from '../src/i18n'
describe('shopping identities and locale presentation', () => {
  it('validates barcodes without losing leading zero identity', () => {
    expect(canonicalGtin('034000470693')).toBe('00034000470693')
    expect(canonicalGtin('0034000470693')).toBe('00034000470693')
    for (const code of ['00000000', '12345671', '1e12', 'https://local', '１２３４５６７０'])
      expect(canonicalGtin(code)).toBeNull()
  })
  it('normalizes regional locale tags with English fallback', () => {
    expect(normalizeLocale('de-DE')).toBe('de')
    expect(normalizeLocale('DE_at')).toBe('de')
    expect(normalizeLocale('fr-FR')).toBe('en')
    expect(normalizeLocale('deception')).toBe('en')
  })
  it('formats counts, dates, meals and exact quantities independently', () => {
    locale.value = 'en'
    expect(shoppingUnit('piece', '1.000000')).toBe('piece')
    expect(shoppingUnit('piece', '2')).toBe('pieces')
    expect(servingCount(1)).toBe('1 serving')
    expect(servingCount(2)).toBe('2 servings')
    expect(calendarDate('2026-11-10')).toContain('2026')
    expect(mealName('dinner')).toBe('Dinner')
    locale.value = 'de'
    expect(servingCount(1)).toBe('1 Portion')
    expect(servingCount(2)).toBe('2 Portionen')
    expect(mealName('dinner')).toBe('Abendessen')
    expect(compactExact('999999999.000001')).toBe('999999999,000001')
    locale.value = 'en'
  })
})
