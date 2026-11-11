import { describe, expect, it } from 'vitest'
import { collectionDraftSchema, recipientSchema, sourceLinkSchema } from '../src/library-api'
import { recipeOriginSchema } from '../src/contracts'

describe('recipe library contracts', () => {
  it('preserves an original HTTPS URL and rejects unsafe or credential-bearing sources', () => {
    const source_url = 'HTTPS://example.org/original?amount=100.500000#step-2'
    expect(recipeOriginSchema.parse({ kind: 'imported', source_url })).toEqual({
      kind: 'imported',
      source_url,
    })
    expect(sourceLinkSchema.parse({ kind: 'source_link', source_url })).toEqual({
      kind: 'source_link',
      source_url,
    })
    for (const value of [
      'http://example.org',
      'javascript:alert(1)',
      'https://user:secret@example.org',
      '',
    ]) {
      expect(recipeOriginSchema.safeParse({ kind: 'imported', source_url: value }).success).toBe(
        false,
      )
    }
  })
  it('rejects cached recipe data in a source-link-only response', () => {
    for (const extra of [
      { title: 'Cached title' },
      { ingredients: ['oats'] },
      { steps: ['Cached instruction'] },
      { image_id: 'image-00000001.png' },
    ]) {
      expect(
        sourceLinkSchema.safeParse({
          kind: 'source_link',
          source_url: 'https://example.org/original',
          ...extra,
        }).success,
      ).toBe(false)
    }
  })
  it('retains pinned revisions and rejects duplicate recipe identities', () => {
    const entry = { recipe_id: 'recipe-00000001', revision: 7 }
    expect(
      collectionDraftSchema.parse({ name: 'Weeknight recipes', description: '', entries: [entry] })
        .entries,
    ).toEqual([entry])
    expect(
      collectionDraftSchema.safeParse({
        name: 'Weeknight recipes',
        description: '',
        entries: [entry, { ...entry, revision: 8 }],
      }).success,
    ).toBe(false)
  })
  it('distinguishes personal grants, household invitations and accepted household grants', () => {
    for (const recipient of [
      { kind: 'account', user: 'alice' },
      { kind: 'household_invitation', user: 'alice' },
      { kind: 'household', home_id: 'home-1' },
    ]) {
      expect(recipientSchema.parse(recipient)).toEqual(recipient)
    }
    expect(recipientSchema.safeParse({ kind: 'household', user: 'alice' }).success).toBe(false)
  })
})
