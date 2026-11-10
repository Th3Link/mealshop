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

it('retains typed author temperature and original spelling across recipe parsing', async () => {
  const { recipeSchema } = await import('../src/contracts')
  const extension = {
    type: 'temperature',
    value: '60.00',
    unit: 'celsius',
    original_input: '60,00 °C',
    provenance: 'recipe_author',
  }
  const recipe = recipeSchema.parse({
    title: 'Water instruction',
    language: 'en',
    author: '',
    servings: 1,
    preparation: 'as_recorded',
    parts: [],
    lines: [{ ingredient_id: 'water', grams: '150', extensions: [extension] }],
    steps: [],
  })
  expect(recipe.lines[0]?.extensions).toEqual([extension])
  expect(recipe.lines[0]?.grams).toBe('150')
  expect(
    recipeSchema.safeParse({
      ...recipe,
      lines: [{ ...recipe.lines[0], extensions: [{ ...extension, unit: 'g' }] }],
    }).success,
  ).toBe(false)
  expect(
    recipeSchema.safeParse({
      ...recipe,
      lines: [{ ...recipe.lines[0], extensions: [{ ...extension, value: 60 }] }],
    }).success,
  ).toBe(false)
})
