import { z } from 'zod'
export const historyInputSchema = z
  .object({
    date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    timezone: z.string().min(1).max(80),
    attendees: z.array(z.string()).min(1).max(100),
    active_seconds: z.number().int().min(0).max(604800).nullable(),
    household_rating: z.number().int().min(1).max(5).nullable(),
    ratings: z.record(z.string(), z.number().int().min(1).max(5)),
  })
  .refine(
    (v) =>
      new Set(v.attendees).size === v.attendees.length &&
      Object.keys(v.ratings).every((id) => v.attendees.includes(id)),
  )
export const historyRecipeSchema = z.object({
  id: z.string(),
  revision: z.number().int().positive(),
  title: z.string(),
  language: z.string(),
})
export const historySourceSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('manual') }),
  z.object({ kind: z.literal('session'), run_id: z.string() }),
])
export const historyEntrySchema = z.object({
  id: z.string(),
  source: historySourceSchema,
  recipe: historyRecipeSchema,
  servings: z.number().int().positive(),
  input: historyInputSchema,
  recorded_by: z.string(),
  recorded_at: z.number().int(),
  updated_by: z.string(),
  updated_at: z.number().int(),
  epoch: z.number().int().positive().nullable(),
})
export const historyListSchema = z.object({
  revision: z.number().int(),
  page: z.number().int(),
  per_page: z.number().int(),
  total: z.number().int(),
  entries: z.array(historyEntrySchema),
})
export const historyDetailSchema = z.object({
  revision: z.number().int(),
  entry: historyEntrySchema,
})
export const historySavedSchema = z.object({ revision: z.number().int(), id: z.string() })
export const historyRevisionSchema = z.object({ revision: z.number().int() })
export const learningPolicySchema = z.object({
  enabled: z.boolean(),
  member_ids: z.array(z.string()),
})
export const learningSchema = z.object({
  revision: z.number().int(),
  epoch: z.number().int().positive(),
  policy: learningPolicySchema,
  model_version: z.enum(['rules-v1', 'history-v1']),
  history_records: z.number().int().nonnegative(),
  learned_cooks: z.number().int(),
  learned_recipes: z.number().int(),
})
export const requestedHistorySourceSchema = z.discriminatedUnion('kind', [
  z.object({
    kind: z.literal('manual'),
    recipe_id: z.string().min(1),
    revision: z.number().int().positive(),
    servings: z.number().int().min(1).max(1000000),
  }),
  z.object({ kind: z.literal('session'), run_id: z.string().min(1) }),
])
export const historyDraftSchema = z.object({
  source: requestedHistorySourceSchema,
  input: historyInputSchema,
  confirmed_completed: z.literal(true),
})
export type HistoryEntry = z.infer<typeof historyEntrySchema>
export type HistoryInput = z.infer<typeof historyInputSchema>
export type Learning = z.infer<typeof learningSchema>
export type HistoryDraft = {
  source: z.infer<typeof requestedHistorySourceSchema>
  input: HistoryInput
  confirmed_completed: boolean
}
export function historyPath(id?: string) {
  return `/api/household/history${id ? `/${encodeURIComponent(id)}` : ''}`
}
