import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { ingredientSchema, nutritionSchema } from '../src/contracts'
import {
  hasEstimates,
  hasMissingNutrition,
  missingContribution,
  nutrientValue,
  rangeText,
} from '../src/nutrition-evidence'
import { locale } from '../src/i18n'
const data = JSON.parse(
  readFileSync(new URL('../../../data/catalogue.json', import.meta.url), 'utf8'),
)
const registry = JSON.parse(
  readFileSync(new URL('../../../data/nutrition-estimates-v1.json', import.meta.url), 'utf8'),
)
describe('nutrition evidence display contract', () => {
  it('retains every frozen evidence profile and distinguishes supplements from estimates', () => {
    for (const raw of data.ingredients) {
      const item = ingredientSchema.parse({
        ...raw,
        nutrition_estimates: registry.profiles[raw.id].profiles,
      })
      expect(hasMissingNutrition(item), item.id).toBe(false)
      for (const id of ['energy_kcal', 'protein', 'fat']) {
        const value = nutrientValue(item, id)
        expect(value.amount !== null || value.range !== undefined, `${item.id}: ${id}`).toBe(true)
      }
    }
    const oats = ingredientSchema.parse({
      ...data.ingredients[0],
      nutrition_estimates: registry.profiles.oats.profiles,
    })
    expect(hasEstimates(oats)).toBe(false)
    expect(nutrientValue(oats, 'energy_kcal').amount).toBe('389')
    const psyllium = ingredientSchema.parse({
      ...data.ingredients.find((i: { id: string }) => i.id === 'psyllium-husks'),
      nutrition_estimates: registry.profiles['psyllium-husks'].profiles,
    })
    expect(hasEstimates(psyllium)).toBe(true)
    expect(nutrientValue(psyllium, 'energy_kcal').range).toEqual({ lower: '191', upper: '208' })
    psyllium.nutrients.energy_kcal = { amount: '200', uncertainty: 'Explicit test label' }
    expect(nutrientValue(psyllium, 'energy_kcal')).toEqual({ amount: '200' })
  })
  it('formats complete, degenerate and partial ranges without binary numeric conversions', () => {
    locale.value = 'de'
    expect(rangeText({ lower: '0.000001', upper: '200.500000' })).toBe('0,000001–200,5')
    expect(rangeText({ lower: '0.00', upper: '0.00' })).toBe('≈ 0')
    const result = {
      servings: 1,
      constituents: [],
      ingredients: [],
      warning_codes: [],
      nutrients: [
        {
          id: 'energy_kcal',
          label: 'Energy',
          group: 'energy',
          unit: 'kcal',
          per_serving: null,
          total: null,
          known_total: '100.00',
          known_per_serving: '100.00',
          status: 'partial',
          missing_ingredients: ['a'],
          uncertainty: [],
          range: {
            status: 'partial',
            known_total: { lower: '110.00', upper: '120.00' },
            known_per_serving: { lower: '110.00', upper: '120.00' },
            estimated_ingredients: ['b'],
            missing_ingredients: ['a'],
          },
        },
      ],
    }
    expect(nutritionSchema.parse(result).nutrients[0]?.range?.status).toBe('partial')
    expect(
      nutritionSchema.safeParse({
        ...result,
        nutrients: [{ ...result.nutrients[0], range: { status: 'complete' } }],
      }).success,
    ).toBe(false)
    locale.value = 'en'
    expect(rangeText({ lower: '100.01', upper: '100.02' })).toBe('100.01–100.02')
  })
})

it('marks unsupported ingredient contributions separately from supported estimates', () => {
  const unknown = ingredientSchema.parse(
    data.ingredients.find((i: { id: string }) => i.id === 'psyllium-husks'),
  )
  expect(hasMissingNutrition(unknown)).toBe(true)
  const result = {
    nutrients: ['energy_kcal', 'protein', 'fat', 'available_carbohydrate'].map((id) => ({
      id,
      missing_ingredients: ['synthetic'],
      range: { estimated_ingredients: ['synthetic'] },
    })),
  }
  expect(missingContribution(result, 'synthetic')).toBe(false)
  const protein = result.nutrients.find((n) => n.id === 'protein')
  if (!protein) throw new Error('Missing test protein row')
  protein.range.estimated_ingredients = []
  expect(missingContribution(result, 'synthetic')).toBe(true)
})
