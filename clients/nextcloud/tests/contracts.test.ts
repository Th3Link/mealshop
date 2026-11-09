import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { ingredientSchema } from '../src/contracts'
import { locale, exact, compactExact, parseGrams } from '../src/i18n'
import { quantitySchema } from '../src/quantity-api'

describe('exact localized quantities', () => {
  it('retains six fractional digits and the upper bound without binary floats', () => {
    locale.value = 'en'
    expect(parseGrams('100.000001')).toBe('100.000001')
    expect(parseGrams('1000000000.000000')).toBe('1000000000.000000')
    expect(parseGrams('1000000000.000001')).toBeNull()
    expect(parseGrams('0.000001')).toBe('0.000001')
    expect(parseGrams('0')).toBeNull()
    expect(parseGrams('1e3')).toBeNull()
    expect(parseGrams('1.0000001')).toBeNull()
    expect(parseGrams('9'.repeat(10000))).toBeNull()
  })
  it('does not guess grouped numbers or reinterpret a different input locale', () => {
    locale.value = 'de'
    expect(parseGrams('100,000001')).toBe('100.000001')
    expect(parseGrams('1.000,5')).toBeNull()
    expect(parseGrams('1.5')).toBeNull()
    expect(parseGrams('1.5', 'en')).toBe('1.5')
    expect(exact('100.000001')).toBe('100,000001')
    expect(exact(null)).toBe('Unbekannt')
    locale.value = 'en'
    expect(exact('100.000001')).toBe('100.000001')
    expect(parseGrams('1,000')).toBeNull()
  })
  it('rejects numeric JSON amounts at the boundary', () => {
    expect(quantitySchema.safeParse({ amount: 0.1, unit: 'g' }).success).toBe(false)
    expect(quantitySchema.parse({ amount: '0.100000', unit: 'g' }).amount).toBe('0.100000')
  })
})
it('accepts every actual curated ingredient while preserving unknown readings and provenance', () => {
  const catalogue = JSON.parse(
    readFileSync(new URL('../../../data/catalogue.json', import.meta.url), 'utf8'),
  )
  const ingredients = ingredientSchema.array().parse(catalogue.ingredients)
  expect(ingredients).toHaveLength(313)
  expect(ingredients.some((i) => Object.values(i.nutrients).some((n) => n.amount === null))).toBe(
    true,
  )
  expect(ingredients.every((i) => i.source.version.length > 0 && i.source.license.length > 0)).toBe(
    true,
  )
})

it('removes only redundant display zeros without reducing exact quantity precision', () => {
  locale.value = 'de'
  expect(compactExact('200.000000')).toBe('200')
  expect(compactExact('2.100000')).toBe('2,1')
  expect(compactExact('0.000000001')).toBe('0,000000001')
  expect(compactExact('1000000000')).toBe('1000000000')
  expect(exact('200.000000')).toBe('200,000000')
  locale.value = 'en'
})
