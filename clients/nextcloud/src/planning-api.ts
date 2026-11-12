import { z } from 'zod'
import { nutritionSchema } from './contracts'
import { reviewSchema } from './review'
export const planSchema = z.object({
  allocations: z
    .array(z.object({ date: z.string(), meal: z.string(), servings: z.number().int().positive() }))
    .optional(),
  preparation: z.object({ date: z.string(), time: z.string() }).nullable().optional(),
  date: z.string(),
  meal: z.string(),
  recipe: z.object({ id: z.string(), revision: z.number().int() }),
  attendees: z.array(z.string()).nullable(),
  calendar: z
    .object({
      consent: z.boolean(),
      source: z.string(),
      event_id: z.string(),
      date: z.string(),
      member_ids: z.array(z.string()),
    })
    .nullable(),
  servings: z.number().int(),
  active_seconds: z.number().int(),
  elapsed_seconds: z.number().int(),
  advance_seconds: z.number().int(),
  reviewed_profile_revision: z.number().nullable().optional(),
  reviewed_allergen_version: z.string().nullable().optional(),
})
export type Plan = z.infer<typeof planSchema>
const deltaSchema = z.array(z.object({ ingredient_id: z.string(), grams_delta: z.string() }))
export const previewSchema = z.object({
  plan_revision: z.number().int(),
  profile_revision: z.number().int(),
  feasible: z.boolean(),
  conflicts: z.array(z.string()),
  shopping_delta: deltaSchema,
  nutrition: nutritionSchema.optional(),
  review: reviewSchema.optional(),
  timing: z
    .object({
      reference_servings: z.number().int(),
      active_seconds: z.number().int(),
      elapsed_seconds: z.number().int(),
      advance_preparation_seconds: z.number().int(),
    })
    .optional(),
})
export const plansSchema = z.object({
  revision: z.number().int(),
  profile_revision: z.number().int(),
  meals: z.array(planSchema),
  recipe_revisions: z.array(
    z.object({ id: z.string(), revision: z.number().int(), title: z.string() }),
  ),
  review_required_slots: z.array(z.object({ date: z.string(), meal: z.string() })),
  shopping_requirements: z.array(z.object({ ingredient_id: z.string(), grams: z.string() })),
})

export type PlanReview = { plan: Plan; preview: z.infer<typeof previewSchema>; remove: boolean }

export const recommendationSchema = z.object({
  model_version: z.enum(['rules-v1', 'history-v1']),
  score: z.number().int(),
  baseline_rank: z.number().int().positive(),
  confidence: z.enum(['cold_start', 'limited_history', 'repeated_history']),
  discovery: z.boolean(),
  evidence: z.object({
    cooks: z.number().int().nonnegative(),
    household_ratings: z.number().int().nonnegative(),
    personal_ratings: z.number().int().nonnegative(),
    recent_cooks: z.number().int().nonnegative(),
    earlier_revision_cooks: z.number().int().nonnegative(),
  }),
  reasons: z.array(
    z.enum([
      'declared_dislikes',
      'completed_cooks',
      'household_ratings',
      'attendee_ratings',
      'recent_variety',
      'earlier_revisions',
      'discovery',
    ]),
  ),
})
export const suggestionSchema = previewSchema.extend({
  plan: planSchema,
  recommendation: recommendationSchema,
})
export const suggestionsSchema = z.object({
  suggestions: z.array(suggestionSchema),
  rejected: z.array(
    z.object({ conflicts: z.array(z.string()).optional(), error: z.string().optional() }),
  ),
  reason: z.string().nullable(),
  page: z.number().int().positive(),
  per_page: z.number().int().positive(),
  total: z.number().int().nonnegative(),
  history_revision: z.number().int().nonnegative(),
  learning_epoch: z.number().int().positive(),
  model_version: z.enum(['rules-v1', 'history-v1']),
})
