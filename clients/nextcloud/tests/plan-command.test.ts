import { describe, expect, it } from 'vitest'
import { fold, matchRecipes, readPlanCommand } from '../src/plan-command'

// 2026-11-28 is a Saturday.
const today = '2026-11-28'
const recipes = [
  { id: 'recipe-1', revision: 2, title: 'Chili sin Carne' },
  { id: 'recipe-2', revision: 1, title: 'Spaghetti Bolognese' },
  { id: 'recipe-3', revision: 1, title: 'Spaghetti mit Pesto' },
  { id: 'recipe-4', revision: 1, title: 'Overnight Oats' },
  { id: 'recipe-5', revision: 3, title: 'Käsespätzle' },
]
const read = (text: string) => readPlanCommand(text, today, recipes)
const ids = (text: string) => read(text)?.recipes.map((recipe) => recipe.id)

describe('meal plan sentences are read without a model', () => {
  it('reads German planning sentences with day, meal, servings and recipe', () => {
    expect(read('Plane Chili sin Carne am Freitagabend für 4 Personen')).toEqual({
      intent: 'plan',
      date: '2026-12-04',
      meal: 'dinner',
      servings: 4,
      recipes: [recipes[0]],
      recipeText: 'chili sin carne',
    })
    expect(read('Morgen Mittag gibt es Käsespätzle für zwei')).toMatchObject({
      intent: 'plan',
      date: '2026-11-29',
      meal: 'lunch',
      servings: 2,
      recipes: [recipes[4]],
    })
    expect(read('overnight oats morgen früh')).toMatchObject({
      date: '2026-11-29',
      meal: 'breakfast',
      recipes: [recipes[3]],
    })
    expect(read('heute abend chili')).toMatchObject({ date: today, meal: 'dinner' })
    expect(read('Kaesespaetzle am 3.12. mittags')).toMatchObject({
      date: '2026-12-03',
      meal: 'lunch',
      recipes: [recipes[4]],
    })
  })
  it('reads English sentences and written dates', () => {
    expect(read('Add spaghetti bolognese on Tuesday for dinner for 3 people')).toMatchObject({
      intent: 'plan',
      date: '2026-12-01',
      meal: 'dinner',
      servings: 3,
      recipes: [recipes[1]],
    })
    expect(read('cook chili tonight')).toMatchObject({ date: today, meal: 'dinner' })
    expect(read('overnight oats 2026-12-24 breakfast')).toMatchObject({
      date: '2026-12-24',
      meal: 'breakfast',
    })
    // A day and month that already passed this year means next year; an impossible date is none.
    expect(read('chili am 1.2.')?.date).toBe('2027-02-01')
    expect(read('chili am 31.2.')?.date).toBeNull()
  })
  it('keeps weekday arithmetic explicit', () => {
    // The same weekday means today; "next" means the following calendar week.
    expect(read('chili am samstag')?.date).toBe(today)
    expect(read('chili am sonntag')?.date).toBe('2026-11-29')
    expect(read('chili am montag')?.date).toBe('2026-11-30')
    expect(read('chili nächsten montag')?.date).toBe('2026-11-30')
    expect(read('chili next sunday')?.date).toBe('2026-12-06')
    expect(readPlanCommand('chili nächsten freitag', '2026-11-30', recipes)?.date).toBe(
      '2026-12-11',
    )
  })
  it('distinguishes removing, suggesting and showing from planning', () => {
    expect(read('Lösche Freitag Abendessen')).toMatchObject({
      intent: 'remove',
      date: '2026-12-04',
      meal: 'dinner',
      recipes: [],
      recipeText: '',
    })
    expect(read('remove dinner on friday')).toMatchObject({ intent: 'remove', meal: 'dinner' })
    expect(read('Schlag mir etwas für Sonntag Mittag vor')).toMatchObject({
      intent: 'suggest',
      date: '2026-11-29',
      meal: 'lunch',
      recipeText: '',
    })
    expect(read('suggest something for tomorrow evening')).toMatchObject({
      intent: 'suggest',
      date: '2026-11-29',
      meal: 'dinner',
    })
    expect(read('Zeig nächsten Mittwoch')).toMatchObject({ intent: 'show', date: '2026-12-02' })
  })
  it('offers several recipes instead of guessing and finds none honestly', () => {
    expect(ids('spaghetti am montag')).toEqual(['recipe-2', 'recipe-3'])
    expect(ids('spaghetti pesto am montag')).toEqual(['recipe-3'])
    // One misheard letter in a longer word still finds the title; a short word must be exact.
    expect(ids('spagetti bolognese')).toEqual(['recipe-2'])
    expect(ids('chilli')).toEqual(['recipe-1'])
    expect(read('lasagne am montag')).toMatchObject({ recipes: [], recipeText: 'lasagne' })
    expect(read('  ')).toBeNull()
    expect(read('für 4')).toMatchObject({ servings: 4, recipes: [], recipeText: '' })
  })
  it('does not take a number in a title for servings', () => {
    const numbered = [{ id: 'recipe-9', revision: 1, title: '5 Minuten Brot' }]
    expect(readPlanCommand('5 Minuten Brot morgen', today, numbered)).toMatchObject({
      servings: null,
      recipes: numbered,
    })
    expect(matchRecipes(['brot'], numbered)).toEqual(numbered)
    expect(fold('Frühstück für Jürgen')).toBe('fruehstueck fuer juergen')
  })
})
