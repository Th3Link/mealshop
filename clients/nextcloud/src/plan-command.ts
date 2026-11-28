/** Reads a typed or spoken meal-plan sentence deterministically. No model is involved: fixed
 * German and English vocabulary, calendar arithmetic and title matching. A command never changes
 * the plan; the caller opens the planner and the cook confirms the usual review. */

export type PlanIntent = 'plan' | 'remove' | 'suggest' | 'show'
export type PlanMeal = 'breakfast' | 'lunch' | 'snack' | 'dinner'
export type RecipeChoice = { id: string; revision: number; title: string }
export type PlanCommand = {
  intent: PlanIntent
  /** ISO day, or null when the sentence names none. */
  date: string | null
  meal: PlanMeal | null
  servings: number | null
  /** Matching recipes, best first. Exactly one means it was understood. */
  recipes: RecipeChoice[]
  /** The words left for the recipe title, as heard. */
  recipeText: string
}

/** Lower case without diacritics, so "Frühstück", "Fruehstueck" and "fruhstuck" agree. */
export function fold(value: string): string {
  return value
    .toLowerCase()
    .replace(/ä/g, 'ae')
    .replace(/ö/g, 'oe')
    .replace(/ü/g, 'ue')
    .replace(/ß/g, 'ss')
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
}

const weekdays: Record<string, number> = {
  montag: 1,
  dienstag: 2,
  mittwoch: 3,
  donnerstag: 4,
  freitag: 5,
  samstag: 6,
  sonnabend: 6,
  sonntag: 0,
  monday: 1,
  tuesday: 2,
  wednesday: 3,
  thursday: 4,
  friday: 5,
  saturday: 6,
  sunday: 0,
}
const meals: Record<string, PlanMeal> = {
  fruehstueck: 'breakfast',
  morgens: 'breakfast',
  frueh: 'breakfast',
  vormittag: 'breakfast',
  breakfast: 'breakfast',
  morning: 'breakfast',
  mittag: 'lunch',
  mittags: 'lunch',
  mittagessen: 'lunch',
  lunch: 'lunch',
  noon: 'lunch',
  midday: 'lunch',
  snack: 'snack',
  nachmittag: 'snack',
  nachmittags: 'snack',
  zwischenmahlzeit: 'snack',
  afternoon: 'snack',
  abend: 'dinner',
  abends: 'dinner',
  abendessen: 'dinner',
  abendbrot: 'dinner',
  dinner: 'dinner',
  supper: 'dinner',
  evening: 'dinner',
  tonight: 'dinner',
}
const numbers: Record<string, number> = {
  ein: 1,
  eine: 1,
  einen: 1,
  zwei: 2,
  drei: 3,
  vier: 4,
  fuenf: 5,
  sechs: 6,
  sieben: 7,
  acht: 8,
  neun: 9,
  zehn: 10,
  elf: 11,
  zwoelf: 12,
  one: 1,
  two: 2,
  three: 3,
  four: 4,
  five: 5,
  six: 6,
  seven: 7,
  eight: 8,
  nine: 9,
  ten: 10,
  eleven: 11,
  twelve: 12,
}
const servingWords = new Set([
  'personen',
  'person',
  'leute',
  'portionen',
  'portion',
  'esser',
  'people',
  'persons',
  'servings',
  'serving',
  'portions',
])
const intents: [PlanIntent, string[]][] = [
  [
    'remove',
    [
      'loesche',
      'loesch',
      'loeschen',
      'entferne',
      'entfernen',
      'streiche',
      'streichen',
      'raus',
    ].concat(['remove', 'delete', 'cancel', 'absagen']),
  ],
  [
    'suggest',
    ['vorschlag', 'vorschlaege', 'vorschlagen', 'schlag', 'schlage', 'idee', 'ideen', 'empfiehl']
      .concat(['suggest', 'suggestion', 'suggestions', 'recommend', 'idea', 'ideas'])
      .concat(['passendes', 'suitable']),
  ],
  ['show', ['zeig', 'zeige', 'zeigen', 'show', 'oeffne', 'open', 'gehe', 'go', 'spring']],
]
/** Words that carry no recipe meaning once intent, day, meal and servings are read. */
const filler = new Set(
  (
    'plan plane planen koche kochen mach mache machen setze setz trag trage ein eintragen nimm ' +
    'gibt gib es wir ich mir uns bitte mal noch auf am an im in den dem der die das zum zur zu ' +
    'fuer mit und vom von speiseplan essensplan mahlzeit essen heute morgen uebermorgen ' +
    'naechsten naechste naechster kommenden kommende diesen diese dieses woche vor etwas was ' +
    'add cook make put schedule set the a on for to of at please meal plan this next coming ' +
    'today tomorrow week something what is whats there we i me us have having lets let'
  ).split(' '),
)

function isoDay(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}
function shifted(today: string, days: number): string {
  const date = new Date(`${today}T12:00:00`)
  date.setDate(date.getDate() + days)
  return isoDay(date)
}
function validDay(year: number, month: number, day: number): string | null {
  const date = new Date(year, month - 1, day, 12)
  return date.getFullYear() === year && date.getMonth() === month - 1 && date.getDate() === day
    ? isoDay(date)
    : null
}
/** One edit apart, for a single misheard letter in a longer word. */
function near(a: string, b: string): boolean {
  if (a === b) return true
  if (Math.min(a.length, b.length) < 5 || Math.abs(a.length - b.length) > 1) return false
  let i = 0
  while (i < a.length && i < b.length && a[i] === b[i]) i += 1
  if (a.length === b.length) return a.slice(i + 1) === b.slice(i + 1)
  const [long, short] = a.length > b.length ? [a, b] : [b, a]
  return long.slice(i + 1) === short.slice(i)
}
function wordsMatch(heard: string, titled: string): boolean {
  return (
    heard === titled ||
    (heard.length >= 4 && titled.startsWith(heard)) ||
    (titled.length >= 4 && heard.startsWith(titled)) ||
    near(heard, titled)
  )
}

export function matchRecipes(words: string[], recipes: RecipeChoice[]): RecipeChoice[] {
  if (!words.length) return []
  const scored = recipes
    .map((recipe) => {
      const title = fold(recipe.title)
        .split(/[^a-z0-9]+/)
        .filter(Boolean)
      const hits = words.filter((word) => title.some((part) => wordsMatch(word, part))).length
      return {
        recipe,
        hits,
        extra: title.length - hits,
        exact: title.join(' ') === words.join(' '),
      }
    })
    .filter((entry) => entry.hits > 0)
  // Every heard word must be found; a title that only shares one of several words is not offered.
  const best = Math.max(0, ...scored.map((entry) => entry.hits))
  if (best < words.length && best < Math.ceil(words.length / 2)) return []
  const kept = scored
    .filter((entry) => entry.hits === best)
    .sort(
      (a, b) =>
        Number(b.exact) - Number(a.exact) ||
        a.extra - b.extra ||
        a.recipe.title.localeCompare(b.recipe.title),
    )
  const exact = kept.filter((entry) => entry.exact)
  return (exact.length === 1 ? exact : kept).slice(0, 5).map((entry) => entry.recipe)
}

export function readPlanCommand(
  text: string,
  today: string,
  recipes: RecipeChoice[],
): PlanCommand | null {
  let rest = fold(text).replace(/['’]/g, '')
  if (!rest.trim()) return null
  let date: string | null = null
  const year = Number(today.slice(0, 4))
  // Written dates first, before the dots and dashes are treated as separators.
  rest = rest.replace(/\b(\d{4})-(\d{2})-(\d{2})\b/, (whole, y, m, d) => {
    date = validDay(Number(y), Number(m), Number(d))
    return date ? ' ' : whole
  })
  if (!date)
    rest = rest.replace(/\b(\d{1,2})\.\s?(\d{1,2})\.(\d{4})?/, (whole, d, m, y) => {
      const candidate = validDay(y ? Number(y) : year, Number(m), Number(d))
      // A day and month without a year that already passed means next year.
      date =
        candidate && !y && candidate < today ? validDay(year + 1, Number(m), Number(d)) : candidate
      return date ? ' ' : whole
    })
  // "Freitagabend", "Sonntagmorgen": a weekday with the meal attached.
  rest = rest.replace(
    new RegExp(
      `\\b(${Object.keys(weekdays).join('|')})(abends?|mittags?|morgens?|frueh|nachmittags?)\\b`,
      'g',
    ),
    (_, day: string, meal: string) => `${day} ${meal.startsWith('morgen') ? 'morgens' : meal}`,
  )
  rest = rest
    .replace(/\bday after tomorrow\b/g, ' uebermorgen ')
    .replace(/\b(heute|today|morgen|tomorrow|uebermorgen) (frueh|morgen)\b/g, '$1 morgens')
    .replace(/\bin the (morning|afternoon|evening)\b/g, ' $1 ')
  let words = rest.split(/[^a-z0-9]+/).filter(Boolean)

  let intent: PlanIntent = 'plan'
  for (const [name, vocabulary] of intents) {
    if (words.some((word) => vocabulary.includes(word))) {
      intent = name
      words = words.filter((word) => !vocabulary.includes(word))
      break
    }
  }

  let meal: PlanMeal | null = null
  let next = false
  const kept: string[] = []
  for (const word of words) {
    if (word === 'naechsten' || word === 'naechste' || word === 'naechster' || word === 'next') {
      next = true
      continue
    }
    if (meals[word] && !meal) {
      meal = meals[word]
      if (word === 'tonight' && !date) date = today
      continue
    }
    if (!date && (word === 'heute' || word === 'today')) date = today
    else if (!date && (word === 'morgen' || word === 'tomorrow')) date = shifted(today, 1)
    else if (!date && word === 'uebermorgen') date = shifted(today, 2)
    else if (!date && weekdays[word] !== undefined) {
      const current = new Date(`${today}T12:00:00`).getDay()
      let ahead = (weekdays[word] - current + 7) % 7
      // "Next Friday" is the Friday of the following calendar week.
      if (next) {
        const untilMonday = (8 - current) % 7 || 7
        ahead = untilMonday + ((weekdays[word] + 6) % 7)
      }
      date = shifted(today, ahead)
    } else kept.push(word)
  }
  words = kept

  let servings: number | null = null
  const count = (word: string | undefined) =>
    word === undefined ? null : /^\d{1,3}$/.test(word) ? Number(word) : (numbers[word] ?? null)
  for (let index = 0; index < words.length && servings === null; index += 1) {
    const value = count(words[index])
    const before = words[index - 1],
      after = words[index + 1]
    const counted = after !== undefined && servingWords.has(after)
    // "für 4", "for four people", "4 Portionen"; a bare number inside a title is left alone.
    const closes = after === undefined || filler.has(after)
    if (value && value > 0 && (counted || ((before === 'fuer' || before === 'for') && closes))) {
      servings = value
      words.splice(index, counted ? 2 : 1)
    }
  }

  const title = words.filter((word) => !filler.has(word))
  return {
    intent,
    date,
    meal,
    servings,
    recipes: matchRecipes(title, recipes),
    recipeText: title.join(' '),
  }
}
