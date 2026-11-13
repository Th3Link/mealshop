import { describe, it, expect } from 'vitest'
import catalogue from '../../../data/brewing-processes.json'
import {
  templatesSchema,
  brewRecipeSchema,
  readingInputSchema,
  brewRunSchema,
} from '../src/brewing-api'
import { seedRecipe } from '../src/brewing-format'
import { locale } from '../src/i18n'
describe('brewing source and exact reading contracts', () => {
  it('seeds all sixteen source-linked methods without invented timings, temperatures or ingredient values', () => {
    const c = templatesSchema.parse(catalogue)
    for (const language of ['en', 'de'] as const) {
      locale.value = language
      for (const template of c.templates) {
        const recipe = seedRecipe(c, template.id)
        expect(recipe).toBeDefined()
        expect(recipe?.beverage).toBe(template.beverage)
        if (!recipe) throw new Error('Missing method seed')
        recipe.volume.amount = '20.000001'
        expect('strategy' in recipe.method).toBe(template.beverage === 'beer')
        expect(brewRecipeSchema.parse(recipe).language).toBe(language)
        expect(
          recipe.phases.every(
            (p) =>
              !p.timing_known &&
              p.temperature === null &&
              p.stage.active_seconds === 0 &&
              p.stage.passive_seconds === 0,
          ),
        ).toBe(true)
        expect(recipe.template.sources.map((s) => s.id).sort()).toEqual(
          [...template.sources].sort(),
        )
        expect(recipe.phases.every((p, i) => p.stage.dependencies.length === (i ? 1 : 0))).toBe(
          true,
        )
      }
    }
  })
  it('keeps six-place instrument readings and unknown calibration without assuming conversions', () => {
    const input = {
      phase_id: null,
      measurement: { value: '1.050001', unit: 'sg' },
      measured_at: { local: '2026-11-13T08:00:00', timezone: 'Europe/Berlin', fold: 'reject' },
      source: {
        kind: 'instrument',
        device_id: 'synthetic-device',
        name: 'Synthetic instrument',
        acquisition: 'transcribed',
      },
      calibration: { kind: 'unknown' },
      notes: 'Original transcription',
    }
    expect(readingInputSchema.parse(input)).toEqual(input)
    expect(
      readingInputSchema.safeParse({ ...input, measurement: { value: 1.050001, unit: 'sg' } })
        .success,
    ).toBe(false)
    expect(
      readingInputSchema.safeParse({
        ...input,
        measurement: { value: '1.050001', unit: 'percent' },
      }).success,
    ).toBe(false)
  })
  it('rejects beverage names or food servings masquerading as a brew process', () => {
    const c = templatesSchema.parse(catalogue),
      recipe = seedRecipe(c, 'biab')
    expect(brewRecipeSchema.safeParse({ ...recipe, beverage: 'mead' }).success).toBe(false)
    expect(
      brewRunSchema.safeParse({ kind: 'beer', recipe: { id: 'food-recipe' }, servings: 20 })
        .success,
    ).toBe(false)
  })
})
