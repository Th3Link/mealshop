import { z } from 'zod'
import { generateUrl } from '@nextcloud/router'
import {
  displayLinesSchema,
  nutritionSchema,
  recipeRecordSchema,
  originalRecipeUrlSchema,
} from './contracts'

export const recipeRefSchema = z.object({
  recipe_id: z.string(),
  revision: z.number().int().positive(),
})
export const collectionDraftSchema = z
  .object({
    name: z.string().trim().min(1).max(200),
    description: z.string().max(2000),
    entries: z.array(recipeRefSchema).max(200),
  })
  .refine((value) => new Set(value.entries.map((r) => r.recipe_id)).size === value.entries.length)
export type CollectionDraft = z.infer<typeof collectionDraftSchema>
export type RecipeReference = z.infer<typeof recipeRefSchema>
export const collectionSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  entries: z.array(recipeRefSchema),
})
export const collectionListSchema = z.object({
  revision: z.number().int().nonnegative(),
  total: z.number().int().nonnegative(),
  page: z.number().int().positive(),
  per_page: z.number().int().positive(),
  collections: z.array(
    z.object({
      id: z.string(),
      name: z.string(),
      description: z.string(),
      count: z.number().int().nonnegative(),
      editable: z.boolean(),
    }),
  ),
})
export const collectionDetailSchema = z.object({
  revision: z.number().int().nonnegative(),
  collection: collectionSchema,
  editable: z.boolean(),
  page: z.number().int().positive(),
  per_page: z.number().int().positive(),
  items: z.array(
    z.object({
      recipe_id: z.string(),
      revision: z.number().int().positive(),
      title: z.string(),
      language: z.string(),
      servings: z.number().int().positive(),
      image_id: z.string().nullable(),
    }),
  ),
})
export const librarySavedSchema = z.object({
  id: z.string(),
  revision: z.number().int().nonnegative(),
})
export const libraryRevisionSchema = z.object({ revision: z.number().int().nonnegative() })
export const resourceSchema = z.discriminatedUnion('kind', [
  z.object({
    kind: z.literal('recipe'),
    recipe_id: z.string(),
    revision: z.number().int().positive(),
  }),
  z.object({ kind: z.literal('collection'), collection_id: z.string() }),
])
export type ShareResource = z.infer<typeof resourceSchema>
export const recipientSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('account'), user: z.string() }),
  z.object({ kind: z.literal('household_invitation'), user: z.string() }),
  z.object({ kind: z.literal('household'), home_id: z.string() }),
])
export type ShareRecipient = z.infer<typeof recipientSchema>
export const grantSchema = z.object({
  id: z.string(),
  version: z.number().int().positive(),
  resource: resourceSchema,
  recipient: recipientSchema,
})
export const grantsSchema = z.object({
  revision: z.number().int().nonnegative(),
  total: z.number().int().nonnegative(),
  page: z.number().int().positive(),
  per_page: z.number().int().positive(),
  shares: z.array(z.object({ grant: grantSchema, recipient_label: z.string() })),
})
export const shareMetadataSchema = z.object({
  id: z.string(),
  source_home: z.string(),
  version: z.number().int().positive(),
  owner: z.string(),
  recipient: recipientSchema,
  recipient_label: z.string(),
  pending: z.boolean(),
  resource_kind: z.enum(['recipe', 'collection']),
})
export type ShareMetadata = z.infer<typeof shareMetadataSchema>
export const sourceLinkSchema = z.strictObject({
  kind: z.literal('source_link'),
  source_url: originalRecipeUrlSchema,
})
const recipeSummarySchema = z.object({
  kind: z.literal('recipe'),
  title: z.string(),
  language: z.string(),
  revision: z.number().int().positive(),
  servings: z.number().int().positive(),
  has_image: z.boolean(),
})
const collectionSummarySchema = z.object({
  kind: z.literal('collection'),
  name: z.string(),
  description: z.string(),
  count: z.number().int().nonnegative(),
})
export const sharedListSchema = z.object({
  total: z.number().int().nonnegative(),
  page: z.number().int().positive(),
  per_page: z.number().int().positive(),
  items: z.array(
    z.object({
      share: shareMetadataSchema,
      content: z
        .discriminatedUnion('kind', [
          sourceLinkSchema,
          recipeSummarySchema,
          collectionSummarySchema,
        ])
        .nullable(),
    }),
  ),
})
const scaledSchema = z.object({
  servings: z.number().int().positive(),
  display_lines: displayLinesSchema,
  nutrition: nutritionSchema,
})
export const sharedRecipeSchema = z.object({
  kind: z.literal('recipe'),
  record: recipeRecordSchema,
  scaled: scaledSchema.nullable(),
})
export type SharedRecipe = z.infer<typeof sharedRecipeSchema>
export const sharedDetailSchema = z.object({
  share: shareMetadataSchema,
  content: z.discriminatedUnion('kind', [
    sourceLinkSchema,
    sharedRecipeSchema,
    z.object({
      kind: z.literal('collection'),
      name: z.string(),
      description: z.string(),
      count: z.number().int().nonnegative(),
      page: z.number().int().positive(),
      per_page: z.number().int().positive(),
      items: z.array(
        z.object({
          recipe_id: z.string(),
          revision: z.number().int().positive(),
          content: z.discriminatedUnion('kind', [sourceLinkSchema, recipeSummarySchema]),
        }),
      ),
    }),
  ]),
})
export function collectionPath(home: string, id?: string) {
  return `/api/households/${encodeURIComponent(home)}/collections${id ? '/' + encodeURIComponent(id) : ''}`
}
export function grantPath(home: string, id?: string) {
  return `/api/households/${encodeURIComponent(home)}/shares${id ? '/' + encodeURIComponent(id) : ''}`
}
export function sharedPath(home: string, id: string, recipe?: string) {
  return `/api/shared/${encodeURIComponent(home)}/${encodeURIComponent(id)}${recipe ? '/recipes/' + encodeURIComponent(recipe) : ''}`
}
export function sharedRoute(home: string, id: string, recipe?: string) {
  return `/shared/${encodeURIComponent(home)}/${encodeURIComponent(id)}${recipe ? '/recipes/' + encodeURIComponent(recipe) : ''}`
}
export function sharedImageUrl(home: string, id: string, recipe?: string, revision?: number) {
  return generateUrl(
    `/apps/app_api/proxy/mealshop${sharedPath(home, id, recipe)}/image${revision ? '?revision=' + revision : ''}`,
  )
}
