import { z } from 'zod'
import type { Recipe } from './contracts'
import type { LineDraft } from './quantity-api'
export const importCandidateSchema = z.object({
  title: z.string().nullable(),
  author: z.string().nullable(),
  language: z.string().nullable(),
  yield_text: z.string().nullable(),
  ingredients: z.array(z.string()),
  steps: z.array(z.object({ text: z.string(), section: z.string().nullable() })),
  prep_time: z.string().nullable(),
  cook_time: z.string().nullable(),
  total_time: z.string().nullable(),
})
export const importSourceSchema = z.object({
  requested_url: z.string(),
  final_url: z.string().nullable(),
  method: z.enum(['json_ld', 'manual']),
  retrieved_at: z.number().int().nonnegative().nullable(),
  importer_version: z.string(),
  original: importCandidateSchema,
})
export const importPreviewSchema = z.object({
  preview_id: z.string(),
  expires_at: z.number().int().nonnegative(),
  requested_url: z.string(),
  final_url: z.string().nullable(),
  method: z.enum(['json_ld', 'manual']),
  retrieved_at: z.number().int().nonnegative().nullable(),
  candidates: z.array(importCandidateSchema),
})
export type ImportPreview = z.infer<typeof importPreviewSchema>
export type ImportSource = z.infer<typeof importSourceSchema>
export type ImportCandidate = z.infer<typeof importCandidateSchema>
export type ImportReview = { preview: ImportPreview; candidate: number; key: string }
export function suggestedQuantity(
  raw: string,
  language: string | null,
): { amount: string; unit: 'g' | 'kg' | 'ml' | 'l' } | null {
  const match = raw.match(/^\s*(\d+(?:[.,]\d{1,6})?)\s*(kg|ml|g|l)\b/i)
  if (!match?.[1] || !match[2]) return null
  const amount = match[1],
    base = language?.toLowerCase().split(/[-_]/)[0]
  if ((amount.includes(',') && base !== 'de') || (amount.includes('.') && base !== 'en'))
    return null
  const canonical = amount.replace(',', '.')
  if (Number(canonical) <= 0 || Number(canonical) > 1_000_000_000) return null
  return { amount: canonical, unit: match[2].toLowerCase() as 'g' | 'kg' | 'ml' | 'l' }
}
export function reviewedDraft(review: ImportReview, fallbackLanguage: 'en' | 'de') {
  const source = review.preview.candidates[review.candidate]
  if (!source) throw new Error('Import candidate unavailable')
  const language = source.language?.toLowerCase().split(/[-_]/)[0]
  const yieldMatch = source.yield_text?.match(
    /^(\d+)\s+(?:servings?|portions?|Portionen?|Personen?)$/i,
  )
  const servings = yieldMatch?.[1] ? Number(yieldMatch[1]) : 0
  const sourceLines: Record<string, string> = {}
  const lines: LineDraft[] = source.ingredients.map((raw) => {
    const id = `line-${crypto.randomUUID()}`
    sourceLines[id] = raw
    const quantity = suggestedQuantity(raw, source.language)
    return {
      id,
      part_id: null,
      prepared_part: null,
      extensions: [],
      ingredient_id: '',
      input: quantity?.amount ?? '',
      unit: quantity?.unit ?? 'g',
      evidence: null,
      mode: 'manual',
      density: '',
      spoon: '',
      invalidFields: [],
    }
  })
  const recipe: Recipe = {
    title: source.title ?? '',
    author: source.author ?? '',
    language: language === 'en' || language === 'de' ? language : fallbackLanguage,
    servings: servings > 0 && servings <= 1_000_000 ? servings : 0,
    preparation: 'as_recorded',
    origin: { kind: 'imported', source_url: review.preview.requested_url },
    lines: [],
    parts: [],
    steps: source.steps.map((step) => ({
      part_id: null,
      instruction: step.section ? `${step.section}: ${step.text}` : step.text,
      active_seconds: 0,
      passive_seconds: 0,
      ingredient_line_ids: [],
      prepared_parts: [],
    })),
  }
  return { recipe, lines, sourceLines }
}
