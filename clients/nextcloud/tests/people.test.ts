import { describe, expect, it } from 'vitest'
import { memberSchema, organizationSchema, personRulesSchema } from '../src/contracts'
import { reviewSchema } from '../src/review'

describe('person rules and legacy editing', () => {
  const legacy = {
    id: 'person',
    name: 'Synthetic person',
    allergies: [],
    intolerances: [],
    allergen_ids: [],
    excluded_ingredients: [],
    disliked_ingredients: [],
  }
  it('preserves explicit exceptions, hard settings and optional avatar in a profile round trip', () => {
    const rules = {
      version: 1,
      preset: 'vegan',
      exceptions: [{ ingredient_id: 'egg-whole-raw', household_produced_only: true }],
      max_active_seconds: 900,
      availability: [{ weekdays: [0, 4], meal: 'dinner' }],
    }
    expect(memberSchema.parse({ ...legacy, rules, avatar: { image_id: null } })).toMatchObject({
      rules,
      avatar: { image_id: null },
    })
    expect(memberSchema.parse(legacy).rules).toBeUndefined()
    expect(
      personRulesSchema.safeParse({ ...rules, availability: [{ weekdays: [], meal: 'dinner' }] })
        .success,
    ).toBe(false)
  })
  it('distinguishes an omitted production origin from explicitly resetting it to unknown', () => {
    expect(organizationSchema.parse({}).production).toBeUndefined()
    expect(organizationSchema.parse({ production: 'unknown' }).production).toBe('unknown')
    expect(organizationSchema.parse({ production: 'household_produced' }).production).toBe(
      'household_produced',
    )
  })
  it('reads old reviews and retains current per-person rule evidence', () => {
    const legacyReview = {
      status: 'review_required',
      excluded_ingredients: [],
      allergen_matches: [],
      review_notes: [],
      disliked_ingredients: [],
      unknown_ingredients: [],
      warning_codes: [],
    }
    expect(reviewSchema.parse(legacyReview).dietary_rules).toEqual([])
    const rule = {
      member_id: 'person',
      preset: 'vegan',
      has_exceptions: true,
      excluded_ingredients: [],
      unknown_ingredients: ['egg-whole-raw'],
      exceptions: [],
    }
    expect(reviewSchema.parse({ ...legacyReview, dietary_rules: [rule] }).dietary_rules).toEqual([
      rule,
    ])
  })
})
