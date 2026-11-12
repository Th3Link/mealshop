import { z } from 'zod'
import { roleSchema } from './contracts'
export const mcpScopeSchema = z.enum(['disabled', 'read', 'create'])
export const mcpSettingsSchema = z.object({
  revision: z.number().int().nonnegative(),
  user: z.string(),
  owner: z.boolean(),
  scope: mcpScopeSchema,
  members: z.record(z.string(), roleSchema),
  grants: z.record(z.string(), mcpScopeSchema),
})
export const mcpActivitySchema = z.object({
  page: z.number().int().positive(),
  per_page: z.literal(20),
  total: z.number().int().nonnegative(),
  entries: z.array(
    z.object({
      recipe_id: z.string(),
      revision: z.literal(1),
      actor: z.string(),
      recorded_at: z.number().int().nonnegative(),
      title: z.string(),
      language: z.enum(['en', 'de']),
    }),
  ),
})
export type McpScope = z.infer<typeof mcpScopeSchema>
export type McpSettings = z.infer<typeof mcpSettingsSchema>
