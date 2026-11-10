import { z } from 'zod'
import { quantitySchema, ingredientExtensionSchema } from './quantity-api'
export const localTimeSchema = z.object({
  local: z.string(),
  timezone: z.string(),
  fold: z.enum(['reject', 'earlier', 'later']),
})
export type LocalTime = z.infer<typeof localTimeSchema>
export const kindSchema = z.enum(['cooking', 'bread', 'rolls', 'pizza', 'baguette'])
export const stageSchema = z.object({
  start_after_work: z.boolean().optional(),
  start_at: localTimeSchema.nullable().optional(),
  id: z.string(),
  origin: z.discriminatedUnion('kind', [
    z.object({ kind: z.literal('recipe_step'), index: z.number().int() }),
    z.object({ kind: z.literal('custom'), note: z.string() }),
  ]),
  name: z.string(),
  instruction: z.string(),
  dependencies: z.array(z.string()),
  active_seconds: z.number().int(),
  passive_seconds: z.number().int(),
  passive_min_seconds: z.number().int(),
  passive_max_seconds: z.number().int(),
  equipment: z.array(z.string()),
  lead_seconds: z.number().int(),
  repeat: z.number().int(),
  wait_between_seconds: z.number().int(),
  uncertainty: z.string(),
})
export type Stage = z.infer<typeof stageSchema>
export const planSlotSchema = z.object({ date: z.string(), meal: z.string() })
export const draftSchema = z.object({
  plan_slot: planSlotSchema.nullable().optional(),
  recipe_id: z.string(),
  recipe_revision: z.number().int(),
  servings: z.number().int(),
  name: z.string(),
  kind: kindSchema,
  timezone: z.string(),
  target: localTimeSchema.nullable(),
  stages: z.array(stageSchema),
})
export type CookingDraft = z.infer<typeof draftSchema>
const recipeRefSchema = z.object({
  id: z.string(),
  revision: z.number().int(),
  title: z.string(),
  language: z.string(),
  reference_servings: z.number().int(),
})
export const permissionSchema = z.enum(['default', 'granted', 'denied', 'unsupported'])
export type DevicePermission = z.infer<typeof permissionSchema>
const subscriptionSchema = z.object({
  enabled: z.boolean(),
  language: z.string(),
  permission: permissionSchema,
  reported_at: z.number().int(),
})
export const timerSchema = z.discriminatedUnion('state', [
  z.object({ state: z.literal('pending') }),
  z.object({
    state: z.literal('running'),
    started_at: z.number().int(),
    active_until: z.number().int(),
    deadline: z.number().int(),
  }),
  z.object({
    state: z.literal('paused'),
    started_at: z.number().int(),
    paused_at: z.number().int(),
    active_remaining: z.number().int(),
    remaining: z.number().int(),
  }),
  z.object({
    state: z.literal('completed'),
    started_at: z.number().int().nullable(),
    completed_at: z.number().int(),
  }),
  z.object({ state: z.literal('cancelled') }),
])
const timingSchema = z.object({
  id: z.string(),
  start: z.number().int(),
  active_until: z.number().int(),
  finish: z.number().int(),
  earliest: z.number().int(),
  latest: z.number().int(),
})
export const runSchema = z.object({
  plan_slot: planSlotSchema.nullable().optional(),
  id: z.string(),
  name: z.string(),
  kind: kindSchema,
  recipe: recipeRefSchema,
  servings: z.number().int(),
  timezone: z.string(),
  target: localTimeSchema.nullable(),
  created_at: z.number().int(),
  status: z.enum(['active', 'completed', 'cancelled']),
  steps: z.array(
    z.object({
      stage: stageSchema,
      cycle: z.number().int(),
      lag_seconds: z.number().int(),
      timer: timerSchema,
      generation: z.number().int(),
      ready: z.boolean(),
      available_at: z.number().int().nullable(),
    }),
  ),
  schedule: z.array(timingSchema),
  subscription: subscriptionSchema.nullable(),
  local_schedule: z.array(
    z.object({
      id: z.string(),
      start: z.string(),
      active_until: z.string(),
      finish: z.string(),
      earliest: z.string(),
      latest: z.string(),
    }),
  ),
})
export type CookingRun = z.infer<typeof runSchema>
const deliverySchema = z.discriminatedUnion('state', [
  z.object({ state: z.literal('queued'), retry_at: z.number().int() }),
  z.object({ state: z.literal('sending'), attempted_at: z.number().int() }),
  z.object({ state: z.literal('accepted'), accepted_at: z.number().int() }),
  z.object({ state: z.literal('failed'), code: z.string(), failed_at: z.number().int() }),
  z.object({ state: z.literal('unknown'), attempted_at: z.number().int() }),
  z.object({ state: z.literal('missed'), detected_at: z.number().int() }),
  z.object({ state: z.literal('cancelled') }),
])
const noticeSchema = z.object({
  id: z.string(),
  run_id: z.string(),
  step_id: z.string(),
  generation: z.number().int(),
  event: z.enum(['prepare', 'start', 'active_ended', 'timer_ended']),
  due: z.number().int(),
  attempts: z.number().int(),
  delivery: deliverySchema,
  acknowledged_at: z.number().int().nullable(),
})
export type CookingNotice = z.infer<typeof noticeSchema>
const healthSchema = z.object({
  last_check_at: z.number().int().nullable(),
  error: z.string().nullable(),
  writable: z.boolean(),
})
const common = {
  revision: z.number().int(),
  server_now: z.number().int(),
  notification_transport: z.boolean(),
  scheduler: healthSchema,
}
export const cookingListSchema = z.object({
  ...common,
  runs: z.array(
    z.object({
      id: z.string(),
      name: z.string(),
      kind: kindSchema,
      recipe: recipeRefSchema,
      servings: z.number().int(),
      timezone: z.string(),
      status: z.enum(['active', 'completed', 'cancelled']),
      created_at: z.number().int(),
      plan_slot: planSlotSchema.nullable().optional(),
      calendar_entries: z
        .array(z.object({ step_id: z.string(), at: z.number().int(), name: z.string().nullable() }))
        .optional(),
      running: z.number().int(),
      overdue: z.number().int(),
      next: z
        .object({ step_id: z.string(), at: z.number().int(), name: z.string().nullable() })
        .nullable(),
    }),
  ),
})
export type CookingList = z.infer<typeof cookingListSchema>
export const cookingDetailSchema = z.object({
  ...common,
  run: runSchema,
  notices: z.array(noticeSchema),
  ingredients: z.array(
    z.object({
      ingredient_id: z.string(),
      name: z.string(),
      translations: z.record(z.string(), z.object({ name: z.string() })),
      grams: z.string(),
      original_quantity: quantitySchema.nullable(),
      extensions: z.array(ingredientExtensionSchema).optional(),
      part_id: z.string().nullable(),
      prepared_part_name: z.string().nullable().optional(),
    }),
  ),
})
export type CookingDetail = z.infer<typeof cookingDetailSchema>
export const templateSchema = z.object({
  revision: z.number().int(),
  server_now: z.number().int(),
  notification_transport: z.boolean(),
  draft: draftSchema,
})
export const previewSchema = z.object({
  preview_id: z.string(),
  revision: z.number().int(),
  expires_at: z.number().int(),
  run: runSchema,
  previous: runSchema.nullable(),
  proposal: z.object({
    schedule: z.array(timingSchema),
    warnings: z.array(z.object({ code: z.string(), steps: z.array(z.string()) })),
    finish: z.number().int(),
  }),
})
export type CookingPreview = z.infer<typeof previewSchema>
export const cookingMutationSchema = z.object({
  revision: z.number().int(),
  run_id: z.string().nullable(),
})
export type PlanChange =
  | { kind: 'target'; target: LocalTime }
  | { kind: 'delay'; step_id: string; start_after: LocalTime }
  | { kind: 'timer'; step_id: string; deadline: LocalTime }
export type Proposal =
  { kind: 'create'; draft: CookingDraft } | { kind: 'replan'; run_id: string; change: PlanChange }
export type CookingCommand =
  | { operation: 'apply'; preview_id: string; reviewed: boolean }
  | {
      operation: 'timer'
      run_id: string
      step_id: string
      action: 'start' | 'pause' | 'resume' | 'complete'
    }
  | { operation: 'cancel' | 'remove'; run_id: string }
  | {
      operation: 'subscribe'
      run_id: string
      enabled: boolean
      language: string
      permission: DevicePermission
    }
  | { operation: 'acknowledge'; notice_id: string }
  | { operation: 'retry'; notice_id: string; duplicate_risk_accepted: boolean }
