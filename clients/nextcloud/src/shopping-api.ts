import { z } from 'zod'
export const shoppingUnitSchema = z.enum(['g', 'kg', 'ml', 'l', 'piece'])
export const shoppingQuantitySchema = z.object({ amount: z.string(), unit: shoppingUnitSchema })
export type ShoppingQuantity = z.infer<typeof shoppingQuantitySchema>
export const productEvidenceSchema = z.object({
  source_url: z.string().regex(/^https:\/\/world\.openfoodfacts\.org\/product\/\d{8,14}$/),
  licence: z.literal('ODbL-1.0'),
  retrieved_at: z.number().int(),
  modified_at: z.number().int().nullable(),
  original_name: z.string(),
  original_brand: z.string(),
  original_quantity: z.string(),
})
export const productSchema = z.object({
  barcode: z.string(),
  name: z.string(),
  brand: z.string(),
  label_quantity: z.string(),
  quantity: shoppingQuantitySchema,
  ingredient_id: z.string().nullable(),
  evidence: productEvidenceSchema.nullable(),
})
export const productCandidateSchema = z.object({
  id: z.string(),
  barcode: z.string(),
  evidence: productEvidenceSchema,
})
export type Product = z.infer<typeof productSchema>
export type ProductCandidate = z.infer<typeof productCandidateSchema>
export type ProductInput = Omit<Product, 'evidence'> & {
  match_confirmed: boolean
  candidate_id?: string
}
export function canonicalGtin(code: string): string | null {
  if (!/^(?:\d{8}|\d{12,14})$/.test(code) || /^0+$/.test(code)) return null
  const sum = [...code]
    .reverse()
    .reduce((total, digit, i) => total + Number(digit) * (i % 2 === 0 ? 1 : 3), 0)
  return sum % 10 === 0 ? code.padStart(14, '0') : null
}
export const inventoryBindingSchema = z.object({
  ingredient_id: z.string().nullable(),
  name: z.string(),
  translations: z.record(z.string(), z.string()),
  product: productSchema.nullable(),
})
export const offerSchema = z.object({
  id: z.string(),
  shop_id: z.string(),
  shop_name: z.string(),
  binding: inventoryBindingSchema,
  quantity: shoppingQuantitySchema,
  price: z.string(),
  currency: z.enum(['EUR', 'GBP', 'USD', 'CHF']),
  valid_from: z.string(),
  valid_until: z.string(),
  source: z.string(),
  source_url: z.string().nullable(),
  archived: z.boolean(),
})
const recipeFields = {
  recipe_id: z.string(),
  revision: z.number().int(),
  title: z.string(),
  language: z.string(),
  servings: z.number().int(),
}
export const shoppingOriginSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('offer'), offer: offerSchema, packages: z.number().int() }),
  z.object({ kind: z.literal('product'), product: productSchema, packages: z.number().int() }),
  z.object({ kind: z.literal('manual'), label: z.string(), quantity: shoppingQuantitySchema }),
  z.object({ kind: z.literal('recipe'), ...recipeFields }),
  z.object({ kind: z.literal('meal'), date: z.string(), meal: z.string(), ...recipeFields }),
])
export type ShoppingOrigin = z.infer<typeof shoppingOriginSchema>
export const shoppingItemRefSchema = z.object({
  key: z.string(),
  ingredient_id: z.string().nullable(),
  name: z.string(),
  translations: z.record(z.string(), z.string()),
  unit: shoppingUnitSchema,
})
export const shoppingSchema = z.object({
  revision: z.number().int(),
  products: z.record(z.string(), productSchema),
  shops: z.object({
    entries: z.array(z.object({ id: z.string(), name: z.string() })),
    preferences: z.record(
      z.string(),
      z.object({ first: z.string().nullable(), second: z.string().nullable() }),
    ),
    assignments: z.record(z.string(), z.string().nullable()),
    manual_order: z.record(z.string(), z.array(z.string())),
    learning: z.boolean(),
    sessions: z.array(
      z.object({
        id: z.string(),
        shop_id: z.string(),
        completed: z.boolean(),
        events: z.array(z.string()),
        corrected: z.array(z.string()),
      }),
    ),
  }),
  sources: z.array(
    z.object({
      id: z.string(),
      active: z.boolean(),
      origin: shoppingOriginSchema,
      requirements: z.array(z.object({ item: shoppingItemRefSchema, amount: z.string() })),
    }),
  ),
  items: z.array(
    shoppingItemRefSchema.extend({
      shop_id: z.string().nullable(),
      fallback_shop_id: z.string().nullable(),
      order_basis: z.enum(['default', 'uncertain', 'learned', 'manual']),
      samples: z.number().int(),
      remaining: z.string(),
      purchased: z.string(),
      reserved: z.string().default('0'),
      used: z.string().default('0'),
      total: z.string(),
      contributions: z.array(
        z.object({
          source_id: z.string(),
          origin: shoppingOriginSchema,
          required: z.string(),
          purchased: z.string(),
          reserved: z.string().default('0'),
          used: z.string().default('0'),
          receipts: z.array(z.object({ amount: z.string(), origin: shoppingOriginSchema })),
        }),
      ),
    }),
  ),
})
export type Shopping = z.infer<typeof shoppingSchema>
export type ShoppingCommand =
  | ProductCommand
  | ShopCommand
  | {
      operation: 'add_manual'
      source_id?: string
      label: string
      ingredient_id: string | null
      quantity: ShoppingQuantity
    }
  | { operation: 'remove_source'; id: string }
  | { operation: 'check'; key: string; checked: boolean }
  | {
      operation: 'add_recipe'
      source_id?: string
      recipe: { recipe_id: string; revision: number; servings: number }
    }
  | { operation: 'sync_plan'; plan_revision: number; slots: { date: string; meal: string }[] }

export type ShopCommand =
  | { operation: 'save_shop'; id?: string; name: string }
  | { operation: 'remove_shop'; id: string }
  | { operation: 'preference'; ingredient_id: string; first: string | null; second: string | null }
  | { operation: 'assign_shop'; key: string; shop_id: string | null }
  | { operation: 'order'; shop_id: string; keys: string[] }
  | { operation: 'learning'; enabled: boolean }
  | { operation: 'reset_history' }
  | { operation: 'start_session'; shop_id: string }
  | { operation: 'finish_session'; id: string; discard: boolean }

export const productLookupSchema = z.object({
  shopping: shoppingSchema,
  candidate: productCandidateSchema.nullable(),
})
export type ProductCommand =
  | { operation: 'save_product'; product: ProductInput }
  | { operation: 'add_product'; barcode: string; packages: number }
  | { operation: 'remove_product'; barcode: string }
