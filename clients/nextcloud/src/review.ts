import { z } from 'zod'
export const reviewSchema = z.object({
  dietary_rules: z
    .array(
      z.object({
        member_id: z.string(),
        preset: z.enum(['unrestricted', 'vegetarian', 'vegan']),
        has_exceptions: z.boolean(),
        excluded_ingredients: z.array(z.string()),
        unknown_ingredients: z.array(z.string()),
        exceptions: z.array(z.string()),
      }),
    )
    .default([]),
  status: z.enum(['blocked', 'review_required']),
  excluded_ingredients: z.array(z.string()),
  allergen_matches: z.array(
    z.object({
      member_id: z.string(),
      ingredient_id: z.string(),
      allergen_id: z.string(),
      source_url: z
        .url()
        .refine((s) => s.startsWith('https://'))
        .nullable(),
      source_kind: z.string().optional(),
    }),
  ),
  review_notes: z.array(
    z.object({
      member_id: z.string(),
      name: z.string(),
      allergies: z.array(z.string()),
      intolerances: z.array(z.string()),
      allergen_ids: z.array(z.string()),
    }),
  ),
  disliked_ingredients: z.array(z.string()),
  unknown_ingredients: z.array(z.string()),
  warning_codes: z.array(z.string()),
})
