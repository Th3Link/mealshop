import { describe, expect, it } from 'vitest'
import { sourceHost, suggestedRecipe, type Suggestion } from '../src/suggestion-api'

const suggestion = (original: Partial<Suggestion['original']>): Suggestion => ({
  id: 'suggestion-0',
  actor: 'cook',
  received_at: 0,
  source_url: null,
  note: null,
  original: {
    title: null,
    author: null,
    language: null,
    yield_text: null,
    ingredients: [],
    steps: [],
    prep_time: null,
    cook_time: null,
    total_time: null,
    ...original,
  },
})

describe('suggested recipes', () => {
  it('reads suggested wording with the deterministic text rules and guesses nothing', () => {
    const recipe = suggestedRecipe(
      suggestion({
        title: ' Reel pasta ',
        language: 'de',
        yield_text: 'für 4 Personen',
        ingredients: [
          '2 EL Olivenöl',
          '1,5 kg Tomaten',
          'eine Handvoll Basilikum',
          '2-3 Zehen Knoblauch',
        ],
        steps: [{ text: ' Alles anbraten. ', section: null }],
      }),
      'en',
    )
    expect(recipe.title).toBe('Reel pasta')
    expect(recipe.servings).toBe(4)
    expect(recipe.ingredients.map((i) => [i.amount, i.unit])).toEqual([
      ['2', 'tbsp'],
      ['1.5', 'kg'],
      [null, null],
      [null, null],
    ])
    expect(recipe.ingredients[2]?.raw).toBe('eine Handvoll Basilikum')
    expect(recipe.steps).toEqual(['Alles anbraten.'])
  })
  it('leaves an ambiguous yield open and shows only the source host', () => {
    expect(suggestedRecipe(suggestion({ yield_text: '4-6 portions' }), 'en').servings).toBeNull()
    expect(suggestedRecipe(suggestion({ yield_text: '12 muffins' }), 'en').servings).toBe(12)
    expect(sourceHost('https://www.instagram.com/reel/x/')).toBe('instagram.com')
    expect(sourceHost('not a url')).toBe('')
  })
})
