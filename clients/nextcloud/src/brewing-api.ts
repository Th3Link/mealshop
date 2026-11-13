import { z } from 'zod'
import {
  localTimeSchema,
  stageSchema,
  runSchema,
  cookingDetailSchema,
  permissionSchema,
} from './cooking-api'
export const brewQuantity = z
  .object({ amount: z.string(), unit: z.enum(['g', 'kg', 'ml', 'l', 'piece']) })
  .strict()
export const readingUnit = z.enum(['celsius', 'fahrenheit', 'sg', 'plato', 'brix', 'ml', 'l'])
export const measurementSchema = z.object({ value: z.string(), unit: readingUnit }).strict()
const evidence = z
  .object({
    id: z.string(),
    publisher: z.string(),
    title: z.string(),
    url: z.string(),
    document_version: z.string().nullable(),
    retrieved_on: z.string(),
  })
  .strict()
export const beverage = z.enum(['beer', 'cider', 'wine', 'mead'])
const process = z.enum(['cider_juice', 'white_wine', 'red_wine', 'traditional_mead'])
const strategy = z.enum(['extract', 'extract_steep', 'partial_mash', 'all_grain'])
const layout = z.enum([
  'kettle',
  'bag',
  'top_return',
  'malt_pipe',
  'separate_tun',
  'two_vessel',
  'three_vessel',
])
const heat = z.enum(['manual', 'direct', 'herms', 'rims'])
const sparge = z.enum(['none', 'basket_rinse', 'batch', 'fly'])
const phaseKind = z.enum([
  'water',
  'steep',
  'mash',
  'lauter',
  'bag_lift',
  'basket_lift',
  'malt_pipe_lift',
  'sparge',
  'extract',
  'boil',
  'addition',
  'cool',
  'transfer',
  'pitch',
  'ferment',
  'package',
  'clean',
  'yeast_preparation',
  'fruit_preparation',
  'press',
  'juice',
  'must',
  'macerate',
  'rack',
  'mature',
  'custom',
])
export const templatesSchema = z
  .object({
    schema_version: z.literal(1),
    retrieved_on: z.string(),
    sources: z.array(evidence),
    phases: z.record(
      z.string(),
      z
        .object({
          kind: phaseKind,
          name: z.string(),
          translations: z.record(z.string(), z.string()),
          instruction: z.string(),
          instruction_translations: z.record(z.string(), z.string()),
        })
        .strict(),
    ),
    templates: z.array(
      z
        .object({
          id: z.string(),
          version: z.number().int(),
          beverage,
          name: z.string(),
          translations: z.record(z.string(), z.string()),
          strategy: strategy.optional(),
          layout: layout.optional(),
          heat: heat.optional(),
          sparge: sparge.optional(),
          process: process.optional(),
          sources: z.array(z.string()),
          phases: z.array(z.string()),
        })
        .strict(),
    ),
  })
  .strict()
export type BrewTemplates = z.infer<typeof templatesSchema>
export const brewRecipeSchema = z
  .object({
    beverage,
    title: z.string(),
    language: z.enum(['en', 'de']),
    volume: brewQuantity,
    template: z
      .object({ id: z.string(), version: z.number().int(), sources: z.array(evidence) })
      .strict(),
    method: z.union([
      z
        .object({
          strategy,
          layout,
          heat,
          sparge,
          mash: z.enum(['none', 'infusion', 'step', 'decoction']),
        })
        .strict(),
      z.object({ process }).strict(),
    ]),
    ingredients: z.array(
      z
        .object({
          id: z.string(),
          name: z.string(),
          ingredient_id: z.string().nullable(),
          quantity: brewQuantity.nullable(),
          phase_id: z.string().nullable(),
          note: z.string(),
        })
        .strict(),
    ),
    phases: z.array(
      z
        .object({
          kind: phaseKind,
          timing_known: z.boolean(),
          stage: stageSchema,
          temperature: measurementSchema.nullable(),
          volume: brewQuantity.nullable(),
        })
        .strict(),
    ),
    yeast_preparation: z
      .object({
        days_before: z.number().int(),
        local_time: z.string(),
        fold: z.enum(['reject', 'earlier', 'later']),
        instruction: z.string(),
      })
      .strict()
      .optional(),
    notes: z.string(),
  })
  .strict()
  .refine(
    (r) =>
      'strategy' in r.method
        ? r.beverage === 'beer'
        : (
            {
              cider_juice: 'cider',
              white_wine: 'wine',
              red_wine: 'wine',
              traditional_mead: 'mead',
            } as const
          )[r.method.process] === r.beverage,
    { message: 'Beverage and process must agree' },
  )
export type BrewRecipe = z.infer<typeof brewRecipeSchema>
export const brewRecordSchema = z
  .object({
    id: z.string(),
    revision: z.number().int(),
    recipe: brewRecipeSchema,
    actor: z.string(),
    recorded_at: z.number().int(),
  })
  .strict()
export type BrewRecord = z.infer<typeof brewRecordSchema>
export const equipmentInputSchema = z
  .object({
    name: z.string(),
    kind: z.enum([
      'kettle',
      'mash_tun',
      'hot_liquor_tank',
      'pump',
      'chiller',
      'fermenter',
      'bag',
      'other',
    ]),
    model: z.string(),
    capacity: brewQuantity.nullable(),
    notes: z.string(),
  })
  .strict()
export type BrewEquipmentInput = z.infer<typeof equipmentInputSchema>
export const equipmentSchema = z
  .object({ id: z.string(), details: equipmentInputSchema, archived: z.boolean() })
  .strict()
export const batchDraftSchema = z
  .object({
    name: z.string(),
    recipe_id: z.string(),
    recipe_revision: z.number().int(),
    brew_day: localTimeSchema,
    yeast_preparation: z.discriminatedUnion('choice', [
      z.object({ choice: z.literal('skip') }).strict(),
      z.object({ choice: z.literal('recipe') }).strict(),
      z
        .object({ choice: z.literal('custom'), at: localTimeSchema, instruction: z.string() })
        .strict(),
    ]),
    equipment: z.record(z.string(), z.array(z.string())),
  })
  .strict()
export type BrewBatchDraft = z.infer<typeof batchDraftSchema>
const batchSchema = z
  .object({
    draft: batchDraftSchema,
    equipment: z.array(equipmentSchema),
    actor: z.string(),
    recorded_at: z.number().int(),
    archived: z.boolean(),
  })
  .strict()
export const brewRunSchema = runSchema.omit({ recipe: true, servings: true, kind: true }).extend({
  kind: beverage,
  brew_recipe: z
    .object({
      id: z.string(),
      revision: z.number().int(),
      title: z.string(),
      language: z.string(),
      volume: brewQuantity,
      beverage: beverage.optional(),
    })
    .strict(),
})
export type BrewRun = z.infer<typeof brewRunSchema>
export const readingInputSchema = z
  .object({
    phase_id: z.string().nullable(),
    measurement: measurementSchema,
    measured_at: localTimeSchema,
    source: z.discriminatedUnion('kind', [
      z.object({ kind: z.literal('manual') }).strict(),
      z
        .object({
          kind: z.literal('instrument'),
          device_id: z.string(),
          name: z.string(),
          acquisition: z.enum(['transcribed', 'reported']),
        })
        .strict(),
    ]),
    calibration: z.discriminatedUnion('kind', [
      z.object({ kind: z.literal('unknown') }).strict(),
      z
        .object({
          kind: z.literal('declared'),
          details: z.string(),
          calibrated_at: localTimeSchema.nullable(),
          reference_temperature: measurementSchema.nullable(),
          source_url: z.string().nullable(),
        })
        .strict(),
    ]),
    notes: z.string(),
  })
  .strict()
export type BrewReadingInput = z.infer<typeof readingInputSchema>
const readingSchema = z
  .object({
    id: z.string(),
    run_id: z.string(),
    input: readingInputSchema,
    measured_instant: z.number().int(),
    actor: z.string(),
    recorded_at: z.number().int(),
    corrects: z.string().nullable(),
    reason: z.string().nullable(),
  })
  .strict()
export type BrewReading = z.infer<typeof readingSchema>
export const readingsPageSchema = z
  .object({
    readings: z.array(readingSchema),
    next: z.string().nullable(),
    total: z.number().int(),
  })
  .strict()
export const brewStateSchema = z
  .object({
    revision: z.number().int(),
    server_now: z.number().int(),
    notification_transport: z.boolean(),
    scheduler: cookingDetailSchema.shape.scheduler,
    recipes: z.array(z.object({ record: brewRecordSchema, archived: z.boolean() }).strict()),
    equipment: z.array(equipmentSchema),
    batches: z.array(
      z
        .object({ run: brewRunSchema, batch: batchSchema, reading_count: z.number().int() })
        .strict(),
    ),
  })
  .strict()
export type BrewState = z.infer<typeof brewStateSchema>
export const brewDetailSchema = cookingDetailSchema
  .omit({ run: true, ingredients: true })
  .extend({
    run: brewRunSchema,
    batch: batchSchema,
    recipe: brewRecordSchema,
    measurements: readingsPageSchema,
  })
  .strict()
export type BrewDetail = z.infer<typeof brewDetailSchema>
export const brewPreviewSchema = z
  .object({ expected_revision: z.number().int(), before: brewStateSchema, after: brewStateSchema })
  .strict()
export type BrewPreview = z.infer<typeof brewPreviewSchema>
export const brewResultSchema = z
  .object({ operation_id: z.string(), replayed: z.boolean(), brewing: brewStateSchema })
  .strict()
export type BrewCommand =
  | {
      operation: 'save_recipe'
      id: string | null
      expected_recipe_revision: number | null
      recipe: BrewRecipe
    }
  | { operation: 'archive_recipe'; id: string; archived: boolean }
  | {
      operation: 'save_equipment'
      id: string | null
      expected_equipment: BrewEquipmentInput | null
      equipment: BrewEquipmentInput
    }
  | { operation: 'archive_equipment'; id: string; archived: boolean }
  | { operation: 'create_batch'; draft: BrewBatchDraft }
  | { operation: 'replan'; run_id: string; change: import('./cooking-api').PlanChange }
  | {
      operation: 'timer'
      run_id: string
      step_id: string
      action: 'start' | 'pause' | 'resume' | 'complete'
    }
  | { operation: 'cancel'; run_id: string }
  | { operation: 'archive_batch'; run_id: string; archived: boolean }
  | {
      operation: 'subscribe'
      run_id: string
      enabled: boolean
      language: string
      permission: z.infer<typeof permissionSchema>
    }
  | { operation: 'acknowledge'; notice_id: string }
  | { operation: 'retry'; notice_id: string; duplicate_risk_accepted: boolean }
  | {
      operation: 'add_reading'
      run_id: string
      reading: BrewReadingInput
      corrects: string | null
      reason: string | null
    }
export interface BrewRequest {
  expected_revision: number
  idempotency_key: string
  command: BrewCommand
  reviewed: boolean
}
