import { z } from 'zod'
export const decimalSchema = z
  .string()
  .max(32)
  .regex(/^\d+(?:\.\d{1,6})?$/)
export const unitSchema = z.enum(['g', 'kg', 'ml', 'l', 'tsp', 'tbsp', 'piece'])
export const quantitySchema = z.object({ amount: decimalSchema, unit: unitSchema })
export const evidenceSchema = z.object({
  ingredient_id: z.string(),
  form: z.string(),
  // Empty only for a household-declared weight of one input unit (see input_unit).
  source_url: z.union([z.literal(''), z.url().refine((s) => s.startsWith('https://'))]),
  version: z.string(),
  uncertainty: z.string(),
  grams_per_base_unit: decimalSchema,
  spoon_standard: z.string().nullable(),
  millilitres_per_spoon: decimalSchema.nullable(),
  input_unit: unitSchema.optional(),
  source_record_id: z.string().optional(),
})
export const conversionsSchema = z.object({
  entries: z.array(
    z.object({ id: z.string(), label: z.string(), unit: unitSchema, evidence: evidenceSchema }),
  ),
})
export type Conversions = z.infer<typeof conversionsSchema>
export const ingredientExtensionSchema = z.discriminatedUnion('type', [
  z
    .object({
      type: z.literal('temperature'),
      value: z
        .string()
        .max(32)
        .regex(/^-?\d+(?:\.\d{1,6})?$/),
      unit: z.literal('celsius'),
      original_input: z.string().min(1).max(100),
      provenance: z.literal('recipe_author'),
    })
    .strict(),
])
export type IngredientExtension = z.infer<typeof ingredientExtensionSchema>
export type Unit = z.infer<typeof unitSchema>
export type Evidence = z.infer<typeof evidenceSchema>
export type LineDraft = {
  extensions?: IngredientExtension[]
  id?: string
  part_id?: string | null
  prepared_part?: string | null
  ingredient_id: string
  input: string
  unit: Unit
  evidence: Evidence | null
  mode: string
  density: string
  /** The weight is the general water-density estimate the cook accepted, not a weighed value. */
  estimated?: boolean
  spoon: string
  invalidFields: ('input' | 'density' | 'spoon' | 'temperature')[]
}

/** Version prefix marking an accepted general estimate instead of a household-weighed value. */
export const estimateVersion = 'household estimate'
/** Grams of one unit if the ingredient weighed like water, using metric 5/15 ml spoons.
 * Pieces have no general weight and never get an estimate. */
export function waterEstimate(unit: Unit): string | null {
  return { ml: '1', l: '1000', tsp: '5', tbsp: '15', g: null, kg: null, piece: null }[unit]
}
