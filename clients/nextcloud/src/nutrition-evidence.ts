import type { Ingredient } from './contracts'
import type { z } from 'zod'
import { boundsSchema } from './contracts'
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
