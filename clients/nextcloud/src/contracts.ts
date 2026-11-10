import { ingredientExtensionSchema } from './quantity-api'
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
const sourceSchema = z
  .object({
    kind: z.enum(['primary', 'household_declaration']).default('primary'),
    name: z.string(),
    url: z.union([z.literal(''), z.url().refine((s) => s.startsWith('https://'))]),
    record_id: z.string(),
    version: z.string(),
    retrieved_on: z.string(),
    license: z.string(),
  })
  .refine((source) => source.kind === 'household_declaration' || source.url.startsWith('https://'))
export const constituentSchema = z.object({
  name: z.string(),
  translations: z.record(z.string(), z.string()).default({}),
  unit: z.enum(['g', 'mg', 'µg']),
  amount: z.string().nullable(),
})
export type Constituent = z.infer<typeof constituentSchema>
const readingSchema = z.object({ amount: z.string().nullable(), uncertainty: z.string() })
export const organizationSchema = z.object({
  is_group: z.boolean().default(false),
  intolerance_triggers: z.array(z.literal('sorbitol')).default([]),
  group_id: z.string().nullable().default(null),
  diet: z.enum(['unknown', 'vegan', 'vegetarian', 'meat_fish']).default('unknown'),
  origin: z.enum(['unknown', 'plant', 'animal', 'mixed']).default('unknown'),
  nutrition: z
    .discriminatedUnion('mode', [
      z.object({ mode: z.literal('reference') }),
      z.object({ mode: z.literal('inherit') }),
      z.object({
        mode: z.literal('own'),
        values: z.record(z.string(), readingSchema),
        constituents: z.record(z.string(), constituentSchema).default({}),
        note: z.string(),
        source_url: z.string(),
      }),
    ])
    .default({ mode: 'reference' }),
})
export type Organization = z.infer<typeof organizationSchema>
export const ingredientSchema = z.object({
  constituents: z.record(z.string(), constituentSchema).default({}),
  organization: organizationSchema.default({
    is_group: false,
    intolerance_triggers: [],
    group_id: null,
    diet: 'unknown',
    origin: 'unknown',
    nutrition: { mode: 'reference' },
  }),
  reference_source: sourceSchema.nullable().optional(),
  nutrition_inherited_from: z.string().nullable().optional(),
  id: z.string(),
  category: z.string(),
  tags: z.array(z.string()).default([]),
  components: z.array(z.string()).default([]),
  declared_allergens: z.array(z.string()).default([]),
  archived: z.boolean().default(false),
  favorite: z.boolean().default(false),
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
  revision: z.number().int().default(0),
  catalogue_version: z.string(),
  language: z.string(),
  ingredients: z.array(ingredientSchema),
  nutrient_definitions: definitionsSchema,
})
export type Catalogue = z.infer<typeof catalogueSchema>
export type Ingredient = z.infer<typeof ingredientSchema>
export const nutritionSchema = z.object({
  constituents: z
    .array(
      z.object({
        id: z.string(),
        label: z.string(),
        translations: z.record(z.string(), z.string()).default({}),
        unit: z.string(),
        total: z.string().nullable(),
        per_serving: z.string().nullable(),
        known_total: z.string(),
        known_per_serving: z.string(),
        status: z.string(),
        missing_ingredients: z.array(z.string()),
      }),
    )
    .default([]),
  servings: z.number(),
  nutrients: z.array(
    z.object({
      id: z.string(),
      label: z.string(),
      group: z.string(),
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
      translations: z
        .record(
          z.string(),
          z.object({
            name: z.string(),
            aliases: z.array(z.string()).default([]),
            form: z.string().nullable().optional(),
          }),
        )
        .default({}),
      source: sourceSchema,
      uncertainty: z.string(),
    }),
  ),
})
export const recipePartSchema = z.object({
  id: z.string(),
  name: z.string(),
  uses_parts: z.array(z.string()).default([]),
  starts_after: z.array(z.string()).optional(),
})
export const recipeSchema = z.object({
  parts: z.array(recipePartSchema).default([]),
  image_id: z.string().nullable().optional(),
  title: z.string(),
  language: z.enum(['en', 'de']),
  author: z.string(),
  servings: z.number().int().positive(),
  preparation: z.enum(['as_recorded', 'cooking_unmodelled']),
  lines: z.array(
    z.object({
      id: z.string().optional(),
      part_id: z.string().nullable().optional(),
      ingredient_id: z.string(),
      extensions: z.array(ingredientExtensionSchema).max(1).optional(),
      grams: z.string(),
      quantity: quantitySchema.optional(),
      conversion: evidenceSchema.optional(),
    }),
  ),
  steps: z.array(
    z.object({
      part_id: z.string().nullable().default(null),
      ingredient_line_ids: z.array(z.string()).default([]),
      prepared_parts: z.array(z.string()).default([]),
      instruction: z.string(),
      active_seconds: z.number().int(),
      passive_seconds: z.number().int(),
      passive_max_seconds: z.number().int().optional(),
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
      image_id: z.string().nullable().optional(),
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
  excluded_tags: z.array(z.string()).default([]),
  disliked_ingredients: z.array(z.string()),
})
export const profilesSchema = z.object({
  schema_version: z.literal(1),
  revision: z.number().int(),
  members: z.array(memberSchema),
})
export type Member = z.infer<typeof memberSchema>
export const emptySchema = z.object({})
