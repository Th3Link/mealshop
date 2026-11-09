import { z } from 'zod'
import { quantitySchema, evidenceSchema } from './quantity-api'
export const roleSchema = z.enum(['owner', 'editor', 'reader'])
export const homesSchema = z.object({
  user: z.string(),
  households: z.array(z.object({ id: z.string(), name: z.string(), role: roleSchema })),
  invitations: z.array(z.object({ id: z.string(), name: z.string(), role: roleSchema })),
})
export type Home = z.infer<typeof homesSchema>['households'][number]
export const accessSchema = z.object({
  id: z.string(),
  name: z.string(),
  members: z.record(z.string(), roleSchema),
  invitations: z.record(z.string(), roleSchema).optional(),
})
const sourceSchema = z.object({
  name: z.string(),
  url: z.url().refine((s) => s.startsWith('https://')),
  record_id: z.string(),
  version: z.string(),
  retrieved_on: z.string(),
  license: z.string(),
})
const readingSchema = z.object({ amount: z.string().nullable(), uncertainty: z.string() })
export const ingredientSchema = z.object({
  id: z.string(),
  category: z.string(),
  name: z.string(),
  aliases: z.array(z.string()),
  form: z.string(),
  basis: z.literal('per_100g_edible_portion'),
  source: sourceSchema,
  uncertainty: z.string(),
  nutrients: z.record(z.string(), readingSchema),
  education: z.array(
    z.object({
      component: z.string(),
      title: z.string(),
      summary: z.string(),
      source_url: z.url().refine((s) => s.startsWith('https://')),
      evidence_reference: z.string(),
      conditions: z.string(),
      limitations: z.string(),
    }),
  ),
  translations: z
    .record(
      z.string(),
      z.object({ name: z.string(), aliases: z.array(z.string()), form: z.string().nullable() }),
    )
    .default({}),
})
export const definitionsSchema = z.array(
  z.object({ id: z.string(), label: z.string(), unit: z.string(), group: z.string() }),
)
export const catalogueSchema = z.object({
  catalogue_version: z.string(),
  language: z.string(),
  ingredients: z.array(ingredientSchema),
  nutrient_definitions: definitionsSchema,
})
export type Catalogue = z.infer<typeof catalogueSchema>
export type Ingredient = z.infer<typeof ingredientSchema>
export const nutritionSchema = z.object({
  servings: z.number(),
  nutrients: z.array(
    z.object({
      id: z.string(),
      label: z.string(),
      unit: z.string(),
      per_serving: z.string().nullable(),
      total: z.string().nullable(),
      known_per_serving: z.string(),
      known_total: z.string(),
      status: z.string(),
      missing_ingredients: z.array(z.string()),
      uncertainty: z.array(z.string()),
    }),
  ),
  warning_codes: z.array(z.string()),
  ingredients: z.array(
    z.object({
      ingredient_id: z.string(),
      name: z.string(),
      grams: z.string(),
      source: sourceSchema,
      uncertainty: z.string(),
    }),
  ),
})
export const recipeSchema = z.object({
  title: z.string(),
  language: z.enum(['en', 'de']),
  author: z.string(),
  servings: z.number().int().positive(),
  preparation: z.enum(['as_recorded', 'cooking_unmodelled']),
  lines: z.array(
    z.object({
      ingredient_id: z.string(),
      grams: z.string(),
      quantity: quantitySchema.optional(),
      conversion: evidenceSchema.optional(),
    }),
  ),
  steps: z.array(
    z.object({
      instruction: z.string(),
      active_seconds: z.number().int(),
      passive_seconds: z.number().int(),
    }),
  ),
})
export const displayLinesSchema = z.array(
  z.object({
    ingredient_id: z.string(),
    amount: z
      .string()
      .max(40)
      .regex(/^\d+(?:\.\d{1,9})?$/),
    unit: z.enum(['g', 'kg']),
  }),
)
export const recipeRecordSchema = z.object({
  id: z.string(),
  revision: z.number().int(),
  recipe: recipeSchema,
  display_lines: displayLinesSchema,
  nutrition: nutritionSchema,
})
export const recipesSchema = z.object({
  recipes: z.array(
    z.object({
      id: z.string(),
      revision: z.number().int(),
      title: z.string(),
      language: z.string(),
      servings: z.number().int(),
    }),
  ),
})
export type Recipe = z.infer<typeof recipeSchema>
export const memberSchema = z.object({
  id: z.string(),
  name: z.string(),
  allergies: z.array(z.string()),
  intolerances: z.array(z.string()),
  allergen_ids: z.array(z.string()),
  excluded_ingredients: z.array(z.string()),
  disliked_ingredients: z.array(z.string()),
})
export const profilesSchema = z.object({
  schema_version: z.literal(1),
  revision: z.number().int(),
  members: z.array(memberSchema),
})
export type Member = z.infer<typeof memberSchema>
export const emptySchema = z.object({})
