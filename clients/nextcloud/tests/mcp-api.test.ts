import { describe, it, expect } from 'vitest'
import { mcpSettingsSchema, mcpActivitySchema } from '../src/mcp-api'
describe('MCP setup contracts', () => {
  it('retains the server effective scope and explicit owner boundary', () => {
    const value = mcpSettingsSchema.parse({
      revision: 1,
      user: 'reader',
      owner: false,
      scope: 'read',
      members: { reader: 'reader' },
      grants: { reader: 'read' },
    })
    expect(value.owner).toBe(false)
    expect(value.scope).toBe('read')
    expect(mcpSettingsSchema.safeParse({ ...value, scope: 'all-households' }).success).toBe(false)
  })
  it('keeps immutable first-revision audit identity separate from credential material', () => {
    const result = mcpActivitySchema.parse({
      page: 1,
      per_page: 20,
      total: 1,
      entries: [
        {
          recipe_id: 'recipe-00000001',
          revision: 1,
          actor: 'alice',
          recorded_at: 100,
          title: 'Synthetic recipe',
          language: 'en',
        },
      ],
    })
    expect(result.entries[0]?.revision).toBe(1)
    expect(
      mcpActivitySchema.safeParse({ ...result, entries: [{ ...result.entries[0], revision: 2 }] })
        .success,
    ).toBe(false)
    expect(mcpActivitySchema.safeParse({ ...result, page: 0 }).success).toBe(false)
  })
})
