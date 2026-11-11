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
  source_url: z.url().refine((s) => s.startsWith('https://')),
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
  spoon: string
  invalidFields: ('input' | 'density' | 'spoon' | 'temperature')[]
}
