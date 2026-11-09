import { z } from 'zod'
export const shoppingUnitSchema = z.enum(['g', 'kg', 'ml', 'l', 'piece'])
export const shoppingQuantitySchema = z.object({ amount: z.string(), unit: shoppingUnitSchema })
export type ShoppingQuantity = z.infer<typeof shoppingQuantitySchema>
const recipeFields = {
  recipe_id: z.string(),
  revision: z.number().int(),
  title: z.string(),
  language: z.string(),
  servings: z.number().int(),
}
export const shoppingOriginSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('manual'), label: z.string(), quantity: shoppingQuantitySchema }),
  z.object({ kind: z.literal('recipe'), ...recipeFields }),
  z.object({ kind: z.literal('meal'), date: z.string(), meal: z.string(), ...recipeFields }),
])
export type ShoppingOrigin = z.infer<typeof shoppingOriginSchema>
const itemSchema = z.object({
  key: z.string(),
  ingredient_id: z.string().nullable(),
  name: z.string(),
  translations: z.record(z.string(), z.string()),
  unit: shoppingUnitSchema,
})
export const shoppingSchema = z.object({
  revision: z.number().int(),
  sources: z.array(
    z.object({
      id: z.string(),
      active: z.boolean(),
      origin: shoppingOriginSchema,
      requirements: z.array(z.object({ item: itemSchema, amount: z.string() })),
    }),
  ),
  items: z.array(
    itemSchema.extend({
      remaining: z.string(),
      purchased: z.string(),
      total: z.string(),
      contributions: z.array(
        z.object({
          source_id: z.string(),
          origin: shoppingOriginSchema,
          required: z.string(),
          purchased: z.string(),
          receipts: z.array(z.object({ amount: z.string(), origin: shoppingOriginSchema })),
        }),
      ),
    }),
  ),
})
export type Shopping = z.infer<typeof shoppingSchema>
export type ShoppingCommand =
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
