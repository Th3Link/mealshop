import { z } from 'zod'
import { importCandidateSchema } from './recipe-import'
import { parseIngredientLine, type TextRecipe } from './recipe-text'

/** Unreviewed recipe text an assistant or companion handed in; never a saved recipe. */
export const suggestionSchema = z.object({
  id: z.string(),
  actor: z.string(),
  received_at: z.number().int().nonnegative(),
  source_url: z.string().nullable(),
  note: z.string().nullable(),
  original: importCandidateSchema,
})
export const suggestionsSchema = z.object({
  revision: z.number().int().nonnegative(),
  capacity: z.number().int().positive(),
  suggestions: z.array(suggestionSchema),
})
export type Suggestion = z.infer<typeof suggestionSchema>

/** Reads the suggested wording with the same deterministic rules as typed text. */
export function suggestedRecipe(suggestion: Suggestion, fallback: 'en' | 'de'): TextRecipe {
  const original = suggestion.original
  const language =
    original.language === 'de' || original.language === 'en' ? original.language : fallback
  const servings = original.yield_text?.match(/^\D*(\d{1,4})\D*$/)?.[1]
  return {
    title: (original.title ?? '').trim().slice(0, 200),
    servings: servings && Number(servings) >= 1 ? Number(servings) : null,
    ingredients: original.ingredients.map((line) => parseIngredientLine(line.trim(), language)),
    steps: original.steps.map((step) => step.text.trim()).filter(Boolean),
  }
}
export function sourceHost(url: string | null): string {
  if (!url) return ''
  try {
    return new URL(url).host.replace(/^www\./, '')
  } catch {
    return ''
  }
}
