import { z } from 'zod'
import {
  shoppingQuantitySchema,
  shoppingUnitSchema,
  shoppingOriginSchema,
  shoppingItemRefSchema,
  inventoryBindingSchema,
  offerSchema,
} from './shopping-api'
export const stockTargetSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('ingredient'), ingredient_id: z.string() }),
  z.object({ kind: z.literal('product'), barcode: z.string() }),
  z.object({ kind: z.literal('unmatched'), label: z.string() }),
])
export const stockMetadataSchema = z.object({
  location: z.string(),
  batch: z.string(),
  expiry: z.string().nullable(),
})
const lotSchema = z.object({
  id: z.string(),
  binding: inventoryBindingSchema,
  original_quantity: shoppingQuantitySchema,
  unit: shoppingUnitSchema,
  metadata: stockMetadataSchema,
  archived: z.boolean(),
  remaining: z.string(),
  reserved: z.string(),
  available: z.string(),
})
const allocationSchema = z.object({
  id: z.string(),
  lot_id: z.string(),
  source_id: z.string(),
  key: z.string(),
  origin: shoppingOriginSchema,
  use_on: z.string(),
  amount: z.string(),
  state: z.enum(['held', 'consumed', 'released']),
  movement_id: z.string().nullable(),
})
const entrySchema = z.object({
  lot_id: z.string(),
  direction: z.enum(['in', 'out']),
  amount: z.string(),
})
const requirementSchema = z.object({ item: shoppingItemRefSchema, amount: z.string() })
const purchaseSchema = z.object({
  id: z.string(),
  source_id: z.string(),
  item: shoppingItemRefSchema,
  amount: z.string(),
  origin: shoppingOriginSchema,
  reversed: z.boolean(),
})
const movementKindSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('opening') }),
  z.object({ kind: z.literal('adjustment'), reason: z.string(), observed: shoppingQuantitySchema }),
  z.object({ kind: z.literal('purchase'), purchases: z.array(purchaseSchema) }),
  z.object({
    kind: z.literal('cooking'),
    run_id: z.string(),
    recipe_id: z.string(),
    revision: z.number().int(),
    servings: z.number().int(),
    requirements: z.array(requirementSchema),
    unbooked: z.array(requirementSchema),
    use_on: z.string(),
  }),
  z.object({ kind: z.literal('reversal'), movement_id: z.string(), reason: z.string() }),
])
const movementSchema = z.object({
  id: z.string(),
  kind: movementKindSchema,
  entries: z.array(entrySchema),
})
const offerInputSchema = offerSchema
  .omit({ binding: true, shop_name: true, archived: true })
  .extend({ id: z.string().nullable(), target: stockTargetSchema })
export const inventoryCommandSchema = z.discriminatedUnion('operation', [
  z.object({
    operation: z.literal('create_lot'),
    target: stockTargetSchema,
    quantity: shoppingQuantitySchema,
    metadata: stockMetadataSchema,
  }),
  z.object({
    operation: z.literal('update_lot'),
    lot_id: z.string(),
    metadata: stockMetadataSchema,
  }),
  z.object({
    operation: z.literal('adjust_lot'),
    lot_id: z.string(),
    quantity: shoppingQuantitySchema,
    reason: z.string(),
  }),
  z.object({ operation: z.literal('archive_lot'), lot_id: z.string() }),
  z.object({ operation: z.literal('reserve'), source_ids: z.array(z.string()), as_of: z.string() }),
  z.object({ operation: z.literal('release'), allocation_ids: z.array(z.string()) }),
  z.object({
    operation: z.literal('receive'),
    purchase_ids: z.array(z.string()),
    metadata: stockMetadataSchema,
  }),
  z.object({
    operation: z.literal('consume'),
    run_id: z.string(),
    source_ids: z.array(z.string()),
    include_unreserved: z.boolean(),
    as_of: z.string(),
  }),
  z.object({ operation: z.literal('reverse'), movement_id: z.string(), reason: z.string() }),
  z.object({ operation: z.literal('save_offer'), offer: offerInputSchema }),
  z.object({ operation: z.literal('archive_offer'), offer_id: z.string() }),
  z.object({
    operation: z.literal('add_offer'),
    offer_id: z.string(),
    packages: z.number().int(),
    as_of: z.string(),
    member_ids: z.array(z.string()),
  }),
])
export const inventorySchema = z.object({
  revision: z.number().int(),
  offset: z.number().int(),
  limit: z.number().int(),
  total: z.number().int(),
  lots: z.array(lotSchema),
  allocations: z.array(allocationSchema),
  movements: z.array(
    movementSchema.extend({
      reversed: z.boolean(),
      actor: z.string().nullable(),
      recorded_at: z.number().int().nullable(),
    }),
  ),
  movement_total: z.number().int(),
  movement_offset: z.number().int(),
  movement_limit: z.number().int(),
  offers: z.array(
    z.object({
      offer: offerSchema,
      safety: z.object({
        status: z.enum(['blocked', 'review_required']),
        unknown: z.literal(true),
        profile_revision: z.number().int(),
      }),
      shop_available: z.boolean(),
    }),
  ),
  purchases: z.array(purchaseSchema),
  cooking_selection: z
    .object({
      run_id: z.string(),
      requirements: z.array(requirementSchema),
      source_ids: z.array(z.string()),
    })
    .nullable(),
  completed_runs: z.array(
    z.object({
      id: z.string(),
      name: z.string(),
      recipe: z.object({
        id: z.string(),
        revision: z.number().int(),
        title: z.string(),
        language: z.string(),
        reference_servings: z.number().int(),
      }),
      servings: z.number().int(),
      plan_slot: z.object({ date: z.string(), meal: z.string() }).nullable().optional(),
      booked: z.boolean(),
    }),
  ),
})
const shoppingDeltaSchema = shoppingItemRefSchema.extend({
  remaining: z.string(),
  purchased: z.string(),
  reserved: z.string(),
  used: z.string(),
})
export const inventoryPreviewSchema = z.object({
  expected_revision: z.number().int(),
  changes: z.array(z.object({ before: lotSchema.nullable(), after: lotSchema })),
  shopping: z.array(
    z.object({ before: shoppingDeltaSchema.nullable(), after: shoppingDeltaSchema }),
  ),
  allocations: z.array(allocationSchema),
  movements: z.array(movementSchema),
  offers: z.array(offerSchema),
})
export const inventoryResultSchema = z.object({
  operation_id: z.string(),
  inventory: inventorySchema,
  replayed: z.boolean(),
})
export type Inventory = z.infer<typeof inventorySchema>
export type InventoryCommand = z.infer<typeof inventoryCommandSchema>
export type InventoryPreview = z.infer<typeof inventoryPreviewSchema>
export type StockLot = z.infer<typeof lotSchema>
export type StockTarget = z.infer<typeof stockTargetSchema>
export type StockMetadata = z.infer<typeof stockMetadataSchema>
export type StockMovement = Inventory['movements'][number]
export type InventoryRequest = {
  expected_revision: number
  idempotency_key: string
  command: InventoryCommand
  reviewed: boolean
}
