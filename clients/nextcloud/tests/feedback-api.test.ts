import { describe, it, expect } from 'vitest'
import { historyDraftSchema, historyInputSchema, historyEntrySchema } from '../src/feedback-api'
const input = {
  date: '2026-11-01',
  timezone: 'Europe/Berlin',
  attendees: ['person-a'],
  active_seconds: null,
  household_rating: null,
  ratings: {},
}
describe('completed history contracts', () => {
  it('keeps unknown effort and ratings unknown and requires explicit completion', () => {
    const draft = {
      source: { kind: 'manual', recipe_id: 'recipe-00000001', revision: 7, servings: 3 },
      input,
      confirmed_completed: true,
    }
    expect(historyDraftSchema.parse(draft)).toEqual(draft)
    expect(historyDraftSchema.safeParse({ ...draft, confirmed_completed: false }).success).toBe(
      false,
    )
    expect(historyInputSchema.safeParse({ ...input, active_seconds: -1 }).success).toBe(false)
    expect(historyInputSchema.safeParse({ ...input, active_seconds: 125 }).success).toBe(true)
  })
  it('rejects duplicate attendees, non-attendee ratings and invented zero ratings', () => {
    for (const invalid of [
      { ...input, attendees: ['person-a', 'person-a'] },
      { ...input, ratings: { other: 5 } },
      { ...input, household_rating: 0 },
    ])
      expect(historyInputSchema.safeParse(invalid).success).toBe(false)
  })
  it('retains pinned history identity, original source and actor provenance', () => {
    const entry = {
      id: 'cook-1',
      source: { kind: 'session', run_id: 'cooking-1' },
      recipe: { id: 'recipe-00000001', revision: 2, title: 'Original title', language: 'en' },
      servings: 3,
      input,
      recorded_by: 'alice',
      recorded_at: 100,
      updated_by: 'bob',
      updated_at: 101,
      epoch: null,
    }
    expect(historyEntrySchema.parse(entry)).toEqual(entry)
  })
})

import { recommendationSchema } from '../src/planning-api'
describe('recommendation evidence contracts', () => {
  const value = {
    model_version: 'history-v1',
    score: 71,
    baseline_rank: 2,
    confidence: 'limited_history',
    discovery: false,
    evidence: {
      cooks: 1,
      household_ratings: 1,
      personal_ratings: 0,
      recent_cooks: 0,
      earlier_revision_cooks: 1,
    },
    reasons: ['completed_cooks', 'household_ratings', 'earlier_revisions'],
  }
  it('preserves evidence counts and explicitly labels earlier recipe revisions', () => {
    expect(recommendationSchema.parse(value)).toEqual(value)
  })
  it('rejects probability-like confidence and negative observation counts', () => {
    expect(recommendationSchema.safeParse({ ...value, confidence: 0.9 }).success).toBe(false)
    expect(
      recommendationSchema.safeParse({ ...value, evidence: { ...value.evidence, cooks: -1 } })
        .success,
    ).toBe(false)
    expect(
      recommendationSchema.safeParse({ ...value, model_version: 'unreviewed-model' }).success,
    ).toBe(false)
  })
})
