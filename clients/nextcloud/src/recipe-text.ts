import type { Ingredient } from './contracts'
import type { Conversions, LineDraft, Unit } from './quantity-api'
import { foldSearch } from './ingredient-search'
import { exact } from './i18n'

/** Deterministic reading of pasted, typed or dictated recipe text. Nothing is guessed:
 * amounts that cannot be written as an exact decimal and ingredient names without an exact
 * catalogue match stay unresolved and keep their original line for review. */
export type TextIngredient = { raw: string; amount: string | null; unit: Unit | null; name: string }
export type TextRecipe = {
  title: string
  servings: number | null
  ingredients: TextIngredient[]
  steps: string[]
}

const units: Record<string, Unit> = {
  g: 'g',
  gr: 'g',
  gramm: 'g',
  gram: 'g',
  grams: 'g',
  kg: 'kg',
  kilo: 'kg',
  kilogramm: 'kg',
  ml: 'ml',
  milliliter: 'ml',
  l: 'l',
  liter: 'l',
  litre: 'l',
  tl: 'tsp',
  teeloffel: 'tsp',
  tsp: 'tsp',
  teaspoon: 'tsp',
  teaspoons: 'tsp',
  el: 'tbsp',
  essloffel: 'tbsp',
  tbsp: 'tbsp',
  tablespoon: 'tbsp',
  tablespoons: 'tbsp',
  stuck: 'piece',
  stk: 'piece',
  piece: 'piece',
  pieces: 'piece',
}
const glyphs: Record<string, string> = { '½': '1/2', '¼': '1/4', '¾': '3/4', '⅛': '1/8' }
const ingredientHeading = /^(zutaten|ingredients)\s*:?$/i
const stepHeading =
  /^(zubereitung|anleitung|schritte|so geht'?s|instructions|method|directions|preparation|steps)\s*:?$/i
const servingsLine =
  /^(?:(?:für|fuer|for|serves|ergibt|makes)\s+)?(\d{1,6})\s*(?:portion(?:en)?|personen?|servings?|people)\s*:?$/i
const quantity = /^(\d+\s+\d+\s*\/\s*\d+|\d+\s*\/\s*\d+|\d+(?:[.,]\d+)?)\s*([\p{L}]+\.?)?\s*(.*)$/u

function fold(value: string) {
  return value.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase().replace(/\.$/, '')
}
/** Exact decimal with at most six fractional digits, or null when the text is ambiguous. */
export function exactAmount(text: string, language: 'en' | 'de'): string | null {
  const mixed = text.match(/^(?:(\d+)\s+)?(\d+)\s*\/\s*(\d+)$/)
  if (mixed) {
    const whole = BigInt(mixed[1] ?? '0'),
      top = BigInt(mixed[2] ?? '0'),
      bottom = BigInt(mixed[3] ?? '0')
    if (bottom === 0n) return null
    const scaled = (whole * bottom + top) * 1_000_000n
    if (scaled % bottom !== 0n) return null
    const micro = scaled / bottom
    if (micro <= 0n || micro > 1_000_000_000_000_000n) return null
    const fraction = (micro % 1_000_000n).toString().padStart(6, '0').replace(/0+$/, '')
    return `${micro / 1_000_000n}${fraction ? `.${fraction}` : ''}`
  }
  const decimal = text.match(/^(\d+)(?:([.,])(\d+))?$/)
  if (!decimal?.[1]) return null
  const separator = decimal[2],
    digits = decimal[3] ?? ''
  const native = language === 'de' ? ',' : '.'
  // "1.500" in German or "1,500" in English is a grouped thousand as often as a decimal.
  if (digits.length > 6 || (separator && separator !== native && digits.length === 3)) return null
  const value = `${decimal[1].replace(/^0+(?=\d)/, '')}${digits ? `.${digits}` : ''}`
  return Number(value) > 0 && Number(value) <= 1_000_000_000 ? value : null
}
export function parseIngredientLine(raw: string, language: 'en' | 'de'): TextIngredient {
  let text = raw.trim()
  for (const [glyph, plain] of Object.entries(glyphs))
    text = text.replace(new RegExp(`(\\d)?\\s*${glyph}`), (_, whole?: string) =>
      whole ? `${whole} ${plain}` : plain,
    )
  const match = text.match(quantity)
  if (!match?.[1]) return { raw, amount: null, unit: null, name: text }
  const word = match[2] ?? '',
    unit = units[fold(word)]
  const rest = (unit ? (match[3] ?? '') : `${word} ${match[3] ?? ''}`).trim()
  const name = rest.replace(/^(?:of|von)\s+/i, '')
  // A range such as "1-2 g" or "2 bis 3" is not one quantity.
  if (/^(?:[-–—]|bis\b|to\b)/i.test(rest) || !name)
    return { raw, amount: null, unit: null, name: text }
  return { raw, amount: exactAmount(match[1], language), unit: unit ?? 'piece', name }
}
export function parseRecipeText(text: string, language: 'en' | 'de'): TextRecipe {
  const result: TextRecipe = { title: '', servings: null, ingredients: [], steps: [] }
  let mode: 'open' | 'ingredients' | 'steps' = 'open'
  for (const original of text.split(/\r?\n/)) {
    const bullet = original.trim().replace(/^(?:[-•*·]\s+|\d{1,2}[.)]\s+)/, '')
    const numbered = bullet !== original.trim() && /^\d/.test(original.trim())
    const line = bullet.trim()
    if (!line) continue
    if (ingredientHeading.test(line)) mode = 'ingredients'
    else if (stepHeading.test(line)) mode = 'steps'
    else if (result.servings === null && servingsLine.test(line))
      result.servings = Number(line.match(servingsLine)?.[1])
    else if (mode === 'steps' || numbered) result.steps.push(line)
    else if (mode === 'ingredients') result.ingredients.push(parseIngredientLine(line, language))
    else if (/^[\d½¼¾⅛]/.test(line) && line.length <= 80 && !/[.!?]$/.test(line))
      result.ingredients.push(parseIngredientLine(line, language))
    else if (!result.title && !result.ingredients.length && !result.steps.length)
      result.title = line.slice(0, 200)
    else result.steps.push(line)
  }
  if (result.servings !== null && (result.servings < 1 || result.servings > 1_000_000))
    result.servings = null
  return result
}
/** Only an exact name or alias identifies an ingredient; a similar food is never substituted. */
export function matchIngredient(name: string, ingredients: Ingredient[]): string {
  for (const text of [name, name.split(/[,(]/)[0] ?? '']) {
    const wanted = foldSearch(text)
    if (!wanted) continue
    const matches = ingredients.filter(
      (item) =>
        !item.archived &&
        [
          item.name,
          ...item.aliases,
          ...Object.values(item.translations).flatMap((tr) => [tr.name, ...tr.aliases]),
        ].some((candidate) => foldSearch(candidate) === wanted),
    )
    if (matches.length === 1) return matches[0]?.id ?? ''
    if (matches.length) return ''
  }
  return ''
}
/** Attach the bundled reference portion when exactly one fits this exact ingredient record. */
export function applyReference(line: LineDraft, ingredients: Ingredient[], known: Conversions) {
  if (line.unit === 'g' || line.unit === 'kg' || line.evidence) return
  const item = ingredients.find((i) => i.id === line.ingredient_id)
  const fitting = known.entries.filter(
    (e) =>
      item &&
      e.unit === line.unit &&
      e.evidence.ingredient_id === item.id &&
      e.evidence.form === item.form &&
      e.evidence.source_record_id === item.source.record_id,
  )
  const only = fitting.length === 1 ? fitting[0] : undefined
  if (!only) {
    // No bundled record: the cook states what one unit weighs instead of filing evidence.
    if (line.mode === 'manual') line.mode = 'own'
    return
  }
  line.mode = only.id
  line.evidence = structuredClone(only.evidence)
  line.density = exact(only.evidence.grams_per_base_unit)
  line.spoon = only.evidence.millilitres_per_spoon ? exact(only.evidence.millilitres_per_spoon) : ''
}
export function textLines(
  recipe: TextRecipe,
  ingredients: Ingredient[],
  known: Conversions,
): { lines: LineDraft[]; sourceLines: Record<string, string> } {
  const sourceLines: Record<string, string> = {}
  const lines = recipe.ingredients.map((item) => {
    const id = `line-${crypto.randomUUID()}`
    sourceLines[id] = item.raw
    const line: LineDraft = {
      id,
      part_id: null,
      prepared_part: null,
      extensions: [],
      ingredient_id: matchIngredient(item.name, ingredients),
      input: item.amount ? exact(item.amount) : '',
      unit: item.unit ?? 'g',
      evidence: null,
      mode: 'manual',
      density: '',
      spoon: '',
      invalidFields: [],
    }
    applyReference(line, ingredients, known)
    return line
  })
  return { lines, sourceLines }
}
