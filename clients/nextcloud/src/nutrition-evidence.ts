import type { Ingredient } from './contracts'
import type { z } from 'zod'
import { boundsSchema, type nutritionSchema } from './contracts'
import { compactExact, exact, t } from './i18n'
export type Bounds = z.infer<typeof boundsSchema>

/** Mirror the server's display precedence; calculation remains on the server. */
export function nutrientValue(
  item: Ingredient,
  id: string,
): { amount: string | null; range?: Bounds } {
  const reading = item.nutrients[id]
  if (reading?.amount !== null && reading?.amount !== undefined) return { amount: reading.amount }
  if (reading?.range) return { amount: null, range: reading.range }
  for (const profile of item.nutrition_estimates) {
    const range = profile.values[id]
    if (range)
      return profile.method === 'source_supplement' && range.lower === range.upper
        ? { amount: range.lower }
        : { amount: null, range }
  }
  return { amount: null }
}
export function hasEstimates(item: Ingredient): boolean {
  const ids = new Set([
    ...Object.keys(item.nutrients),
    ...item.nutrition_estimates.flatMap((p) => Object.keys(p.values)),
  ])
  return [...ids].some((id) => nutrientValue(item, id).range !== undefined)
}
export function rangeText(range: Bounds): string {
  return range.lower === range.upper
    ? `≈ ${compactExact(range.lower)}`
    : `${compactExact(range.lower)}–${compactExact(range.upper)}`
}
export function evidenceLabel(method: Ingredient['nutrition_estimates'][number]['method']): string {
  return t(
    method === 'source_supplement'
      ? 'supplementalValues'
      : method === 'group_envelope'
        ? 'groupRange'
        : 'comparisonRange',
  )
}

export function nutrientText(item: Ingredient, id: string): string {
  const value = nutrientValue(item, id)
  return value.range ? rangeText(value.range) : exact(value.amount)
}

function missingCore(covered: (id: string) => boolean): boolean {
  return (
    ['energy_kcal', 'protein', 'fat'].some((id) => !covered(id)) ||
    !(covered('carbohydrate') || covered('available_carbohydrate'))
  )
}
export function hasMissingNutrition(item: Ingredient): boolean {
  return missingCore((id) => {
    const value = nutrientValue(item, id)
    return value.amount !== null || value.range !== undefined
  })
}
export function missingContribution(
  value: {
    nutrients: {
      id: string
      missing_ingredients: string[]
      range?: { estimated_ingredients: string[] }
    }[]
  },
  id: string,
): boolean {
  return missingCore((nutrient) => {
    const row = value.nutrients.find((row) => row.id === nutrient)
    return (
      row !== undefined &&
      (!row.missing_ingredients.includes(id) ||
        Boolean(row.range?.estimated_ingredients.includes(id)))
    )
  })
}

type Nutrient = z.infer<typeof nutritionSchema>['nutrients'][number]
export type Headline = { text: string; state: 'exact' | 'range' | 'partial' | 'unknown' }
/** A rounded reading aid for one nutrient per serving. It never replaces the exact table:
 * partial sums are labelled as a lower bound and unknown values are not shown as zero. */
export function headline(n: Nutrient, language: string): Headline {
  const digits = n.unit === 'kcal' || n.unit === 'kJ' || n.unit === 'mg' ? 0 : 1
  const round = (value: string) =>
    new Intl.NumberFormat(language, { maximumFractionDigits: digits }).format(Number(value))
  if (n.range?.status === 'complete') {
    const { lower, upper } = n.range.per_serving
    const low = round(lower),
      high = round(upper)
    return { text: low === high ? low : `${low}–${high}`, state: 'range' }
  }
  if (n.range) return { text: round(n.range.known_per_serving.lower), state: 'partial' }
  if (n.per_serving !== null) return { text: round(n.per_serving), state: 'exact' }
  return Number(n.known_per_serving) > 0
    ? { text: round(n.known_per_serving), state: 'partial' }
    : { text: '?', state: 'unknown' }
}
