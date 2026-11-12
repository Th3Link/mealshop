import { describe, expect, it } from 'vitest'
import { suggestedQuantity, reviewedDraft } from '../src/recipe-import'
describe('source quantities remain explicit suggestions', () => {
  it('accepts unambiguous metric values and refuses fractions, ranges and ambiguous separators', () => {
    expect(suggestedQuantity('100,500 g Haferkorn', 'de-DE')).toEqual({
      amount: '100.500',
      unit: 'g',
    })
    expect(suggestedQuantity('0.100500 kg oats', 'en')).toEqual({ amount: '0.100500', unit: 'kg' })
    expect(suggestedQuantity('200 ml water', null)).toEqual({ amount: '200', unit: 'ml' })
    for (const raw of [
      '½ cup flour',
      '1–2 g salt',
      '1/2 g salt',
      '1.000 g salt',
      '1,000 g salt',
      '0 g salt',
      'a pinch of salt',
    ])
      expect(suggestedQuantity(raw, null)).toBeNull()
  })
  it('requires canonical servings and identity review while retaining source lines and unknown times', () => {
    const original = {
      title: 'Synthetic source',
      author: null,
      language: null,
      yield_text: 'one loaf',
      ingredients: ['some flour'],
      steps: [{ text: 'Mix', section: null }],
      prep_time: null,
      cook_time: null,
      total_time: null,
    }
    const review = {
      candidate: 0,
      key: 'synthetic-review-key',
      preview: {
        preview_id: 'synthetic',
        expires_at: 10,
        requested_url: 'https://example.org/original',
        final_url: null,
        method: 'manual' as const,
        retrieved_at: null,
        candidates: [original],
      },
    }
    const draft = reviewedDraft(review, 'de')
    expect(draft.recipe.servings).toBe(0)
    expect(draft.recipe.origin).toEqual({
      kind: 'imported',
      source_url: review.preview.requested_url,
    })
    expect(draft.lines[0]?.ingredient_id).toBe('')
    expect(draft.lines[0]?.input).toBe('')
    expect(Object.values(draft.sourceLines)).toEqual(['some flour'])
    expect(original.prep_time).toBeNull()
  })
})
