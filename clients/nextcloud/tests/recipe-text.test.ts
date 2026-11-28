import { describe, expect, it } from 'vitest'
import {
  exactAmount,
  matchIngredient,
  parseIngredientLine,
  parseRecipeText,
  textLines,
} from '../src/recipe-text'
import { z } from 'zod'
import { ingredientSchema } from '../src/contracts'
import { conversionsSchema } from '../src/quantity-api'
import catalogueData from '../../../data/catalogue.json'
import conversionData from '../../../data/conversions.json'

const catalogue = { ingredients: z.array(ingredientSchema).parse(catalogueData.ingredients) }
const conversions = conversionsSchema.parse(conversionData)

describe('recipe text is read exactly or left for review', () => {
  it('writes terminating fractions and local decimals exactly and refuses ambiguity', () => {
    expect(exactAmount('1/2', 'de')).toBe('0.5')
    expect(exactAmount('1 3/4', 'en')).toBe('1.75')
    expect(exactAmount('2,5', 'de')).toBe('2.5')
    expect(exactAmount('1.5', 'de')).toBe('1.5')
    expect(exactAmount('0250', 'de')).toBe('250')
    for (const [text, language] of [
      ['1/3', 'de'],
      ['1.500', 'de'],
      ['1,500', 'en'],
      ['0', 'en'],
      ['1/0', 'en'],
      ['1,1234567', 'de'],
    ] as const)
      expect(exactAmount(text, language)).toBeNull()
  })
  it('separates amount, unit and name without converting units', () => {
    expect(parseIngredientLine('200 g Mehl', 'de')).toEqual({
      raw: '200 g Mehl',
      amount: '200',
      unit: 'g',
      name: 'Mehl',
    })
    expect(parseIngredientLine('2 EL Olivenöl', 'de')).toMatchObject({ amount: '2', unit: 'tbsp' })
    expect(parseIngredientLine('½ TL Salz', 'de')).toMatchObject({ amount: '0.5', unit: 'tsp' })
    expect(parseIngredientLine('1½ l Milch', 'de')).toMatchObject({ amount: '1.5', unit: 'l' })
    expect(parseIngredientLine('3 Eier', 'de')).toMatchObject({
      amount: '3',
      unit: 'piece',
      name: 'Eier',
    })
    expect(parseIngredientLine('1 cup of oats', 'en')).toMatchObject({
      amount: '1',
      unit: 'piece',
      name: 'cup of oats',
    })
    for (const raw of ['1-2 g Salz', '2 bis 3 Eier', 'etwas Salz'])
      expect(parseIngredientLine(raw, 'de')).toMatchObject({ amount: null, unit: null })
  })
  it('recognizes title, servings, headed sections and numbered steps', () => {
    const recipe = parseRecipeText(
      [
        'Pfannkuchen',
        'Für 4 Personen',
        '',
        'Zutaten:',
        '- 250 g Weizenmehl',
        '- 3 Eier',
        '- Salz',
        'Zubereitung',
        '1. Alles verrühren.',
        '2. 20 Minuten ruhen lassen.',
      ].join('\n'),
      'de',
    )
    expect(recipe.title).toBe('Pfannkuchen')
    expect(recipe.servings).toBe(4)
    expect(recipe.ingredients.map((i) => i.raw)).toEqual(['250 g Weizenmehl', '3 Eier', 'Salz'])
    expect(recipe.steps).toEqual(['Alles verrühren.', '20 Minuten ruhen lassen.'])
  })
  it('reads unheaded dictation line by line', () => {
    const recipe = parseRecipeText(
      'Oat bowl\n100 g oats\n200 ml milk\nMix and let it rest for 5 minutes.',
      'en',
    )
    expect(recipe.ingredients).toHaveLength(2)
    expect(recipe.steps).toEqual(['Mix and let it rest for 5 minutes.'])
  })
  it('identifies ingredients only by one exact name or alias', () => {
    const oats = catalogue.ingredients.find((i) => i.id === 'oats')
    expect(oats).toBeDefined()
    expect(matchIngredient(oats?.name ?? '', catalogue.ingredients)).toBe('oats')
    expect(matchIngredient(`${oats?.name}, fine`, catalogue.ingredients)).toBe('oats')
    expect(matchIngredient('synthetic unknown food', catalogue.ingredients)).toBe('')
  })
  it('attaches a bundled reference portion only for its exact ingredient and unit', () => {
    const entry = conversions.entries[0]
    if (!entry) throw new Error('fixture conversion missing')
    const item = catalogue.ingredients.find((i) => i.id === entry.evidence.ingredient_id)
    const sole =
      conversions.entries.filter(
        (e) => e.unit === entry.unit && e.evidence.ingredient_id === entry.evidence.ingredient_id,
      ).length === 1
    const { lines, sourceLines } = textLines(
      {
        title: '',
        servings: null,
        steps: [],
        ingredients: [
          { raw: 'source line', amount: '1', unit: entry.unit, name: item?.name ?? '' },
          { raw: '200 ml synthetic', amount: '200', unit: 'ml', name: 'synthetic unknown food' },
        ],
      },
      catalogue.ingredients,
      conversions,
    )
    expect(lines[0]?.ingredient_id).toBe(entry.evidence.ingredient_id)
    if (sole) expect(lines[0]?.evidence).toEqual(entry.evidence)
    expect(lines[1]).toMatchObject({ ingredient_id: '', evidence: null, unit: 'ml' })
    expect(Object.values(sourceLines)).toEqual(['source line', '200 ml synthetic'])
  })
})
