import { test, expect, type Page, type Locator } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'
const embedded = '/index.php/apps/app_api/embedded/mealshop/main'
const attendee = 'History attendee'
const title = 'Cooking history acceptance recipe'
async function login(page: Page, reader = false) {
  await page.goto(`${embedded}/households`)
  await page
    .locator('#user')
    .fill(
      reader ? (process.env.MEALSHOP_READER_USER ?? '') : (process.env.MEALSHOP_TEST_USER ?? ''),
    )
  await page
    .locator('#password')
    .fill(
      reader
        ? (process.env.MEALSHOP_READER_PASSWORD ?? '')
        : (process.env.MEALSHOP_TEST_PASSWORD ?? ''),
    )
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('#mealshop-language').selectOption('en')
}
async function check(box: Locator, value = true) {
  await expect(box).toBeEnabled()
  if ((await box.isChecked()) !== value) {
    await box.focus()
    await box.press('Space')
  }
  await expect(box).toBeChecked({ checked: value })
}
async function home(page: Page) {
  const name = `Browser verification ${Date.now()}`
  await page.getByLabel('Household name', { exact: true }).fill(name)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'feedback.spec.ts')
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  if (!id) throw new Error('Synthetic history household was not recorded')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).fill(attendee)
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  return { id, name }
}
async function recipe(page: Page, name = title, ingredient = 'Oats') {
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill(name)
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill(ingredient)
  await page.getByRole('option', { name: ingredient, exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.500000')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Prepare history fixture')
  await page.getByRole('combobox', { name: 'Active work · Unit', exact: true }).selectOption('1')
  await page.getByLabel('Active work', { exact: true }).fill('60')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('heading', { name, exact: true })).toBeVisible()
  const id = new URL(page.url()).pathname.match(/\/(recipe-\d+)$/)?.[1]
  if (!id) throw new Error('Synthetic history recipe identity missing')
  return id
}
async function beginRecord(page: Page, name = title, rating = '5') {
  await page.getByRole('link', { name: 'Cooking history', exact: true }).click()
  await page.getByRole('link', { name: 'Record a completed cook', exact: true }).click()
  await page.getByRole('combobox', { name: 'Cooked recipe', exact: true }).fill(name)
  await page.getByRole('option', { name, exact: true }).click()
  await page.getByLabel('Date cooked', { exact: true }).fill('2026-11-01')
  await check(page.getByRole('checkbox', { name: attendee, exact: true }))
  await page.getByRole('combobox', { name: 'Household rating', exact: true }).selectOption(rating)
  await page
    .getByRole('combobox', { name: `Rating for · ${attendee}`, exact: true })
    .selectOption(rating)
}
async function finishRecord(page: Page, name = title) {
  await check(page.getByRole('checkbox', { name: 'This was actually cooked.', exact: true }))
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('heading', { name, exact: true })).toBeVisible()
}
async function learning(page: Page) {
  await page.getByRole('link', { name: 'Cooking history', exact: true }).click()
  await check(
    page.getByRole('checkbox', { name: 'Use completed cooks for suggestions', exact: true }),
  )
  await check(page.getByRole('checkbox', { name: attendee, exact: true }))
  await page.getByRole('button', { name: 'Save learning scope', exact: true }).click()
  await expect(page.getByText('Learning is on', { exact: true })).toBeVisible()
}
async function audit(page: Page, name: string) {
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    for (const colorScheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme, reducedMotion: 'reduce' })
      for (const width of [1440, 390, 320]) {
        await page.setViewportSize({ width, height: 1000 })
        if (name === 'suggestions')
          await page.locator('.suggestion-item').first().scrollIntoViewIfNeeded()
        await expect
          .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(width)
        await page.screenshot({
          path: `test-results/f11-${name}-${language}-${colorScheme}-${width}.png`,
          animations: 'disabled',
        })
      }
      expect(
        (
          await new AxeBuilder({ page })
            .include('.mealshop')
            .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
            .analyze()
        ).violations,
      ).toEqual([])
    }
  }
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.locator('#mealshop-language').selectOption('en')
}

test('completed history keeps exact reports, epoch consent, revisions and conflicting drafts', async ({
  page,
}) => {
  test.setTimeout(240000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await login(page)
  const household = await home(page)
  const id = await recipe(page)
  await beginRecord(page)
  await audit(page, 'history-editor')
  await page
    .getByRole('combobox', { name: 'Actual active effort · Unit', exact: true })
    .selectOption('1')
  await page.getByLabel('Actual active effort', { exact: true }).fill('125')
  await page.getByLabel('Date cooked', { exact: true }).fill('2099-01-01')
  await check(page.getByRole('checkbox', { name: 'This was actually cooked.', exact: true }))
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('future date')
  await expect(page.getByLabel('Actual active effort', { exact: true })).toHaveValue('125')
  await page.getByLabel('Date cooked', { exact: true }).fill('2026-11-01')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible()
  await expect(page.getByText('00:02:05', { exact: true })).toBeVisible()
  const recordPath = new URL(page.url()).pathname
  await learning(page)
  await audit(page, 'history-learning')
  await expect(page.getByText(/Contributing completed cooks: 0/)).toBeVisible()
  await page.locator('.record-list a').filter({ hasText: title }).click()
  await page.getByRole('link', { name: 'Edit cooking record', exact: true }).click()
  await page.getByLabel('Actual active effort', { exact: true }).fill('126')
  await finishRecord(page)
  await page.getByRole('link', { name: 'Cooking history', exact: true }).click()
  await expect(page.getByText(/Contributing completed cooks: 1/)).toBeVisible()
  const other = await page.context().newPage()
  await other.goto(`${recordPath}?edit=1`)
  await other.getByLabel('Actual active effort', { exact: true }).fill('127')
  await page.goto(`${recordPath}?edit=1`)
  await page.getByRole('combobox', { name: 'Household rating', exact: true }).selectOption('4')
  await finishRecord(page)
  await check(other.getByRole('checkbox', { name: 'This was actually cooked.', exact: true }))
  await other.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(other.getByRole('alert')).toContainText('Your draft is retained')
  await expect(other.getByLabel('Actual active effort', { exact: true })).toHaveValue('127')
  other.once('dialog', (d) => void d.dismiss())
  await other.getByRole('button', { name: 'Reload saved history', exact: true }).click()
  await expect(other.getByLabel('Actual active effort', { exact: true })).toHaveValue('127')
  other.once('dialog', (d) => void d.accept())
  await other.getByRole('button', { name: 'Reload saved history', exact: true }).click()
  await expect(other.getByLabel('Actual active effort', { exact: true })).toHaveValue('126')
  await other.close()
  await page.goto(`${embedded}/h/${household.id}/recipes/${id}?edit=1`)
  await page
    .getByLabel('Recipe title', { exact: true })
    .fill('Revised cooking history acceptance recipe')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Revised cooking history acceptance recipe', exact: true }),
  ).toBeVisible()
  await page.goto(recordPath)
  await page.getByRole('link', { name: 'Recipe revision 1', exact: true }).click()
  await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible()
  await expect(page.getByRole('link', { name: 'Open current revision', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Back', exact: true }).click()
  await expect(page).toHaveURL(new RegExp(`${recordPath}$`))
  await audit(page, 'history-detail')
  page.once('dialog', (d) => void d.accept())
  await page.getByRole('button', { name: 'Delete cooking record', exact: true }).click()
  await expect(page.getByText('No completed cooks recorded', { exact: true })).toBeVisible()
  page.once('dialog', (d) => void d.accept())
  await page.getByRole('button', { name: 'Reset and turn off learning', exact: true }).click()
  await expect(page.getByText('Learning is off', { exact: true })).toBeVisible()
  expect(errors).toEqual([])
})

test('only completed sessions seed history and reader roles stay read-only', async ({
  page,
  browser,
}) => {
  test.setTimeout(180000)
  await login(page)
  const household = await home(page)
  await recipe(page)
  await page.getByRole('link', { name: 'Start cooking or baking', exact: true }).click()
  await expect(page.getByLabel('Name', { exact: true })).toHaveValue(title)
  await page.getByLabel('Servings', { exact: true }).fill('3')
  await page.getByRole('button', { name: 'Review schedule', exact: true }).click()
  await check(
    page.getByRole('checkbox', {
      name: 'I reviewed the times, dependencies and uncertainties.',
      exact: true,
    }),
  )
  await page.getByRole('button', { name: 'Save this schedule', exact: true }).click()
  await expect(page).toHaveURL(/\/cooking\/cook-\d+$/)
  await expect(page.locator('.cooking-timeline > li')).toHaveCount(1)
  const cookingPath = new URL(page.url()).pathname,
    run = cookingPath.split('/').at(-1)
  await page.getByRole('link', { name: 'Cooking history', exact: true }).click()
  await expect(page.getByText('No completed cooks recorded', { exact: true })).toBeVisible()
  await page.goto(`${embedded}/h/${household.id}/history/new?session=${run}`)
  await expect(page.getByRole('alert')).toContainText('Complete the cooking session')
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toHaveCount(0)
  await page.goto(cookingPath)
  const step = page.locator('.cooking-timeline > li')
  await step
    .getByRole('button', { name: 'Start step: Prepare history fixture', exact: true })
    .click()
  await step.getByRole('button', { name: 'Mark step complete', exact: true }).click()
  await page.getByRole('link', { name: 'Record a completed cook', exact: true }).click()
  await expect(
    page.getByText('Recipe and servings are pinned to the completed session.', { exact: true }),
  ).toBeVisible()
  await expect(page.getByRole('combobox', { name: 'Cooked recipe', exact: true })).toHaveCount(0)
  await check(page.getByRole('checkbox', { name: attendee, exact: true }))
  await finishRecord(page)
  const recordPath = new URL(page.url()).pathname
  await expect(page.getByText('Effort not recorded', { exact: true })).toBeVisible()
  await page.goto(cookingPath)
  await page.getByRole('link', { name: 'Record a completed cook', exact: true }).click()
  await expect(page).toHaveURL(new RegExp(`${recordPath}$`))
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page
    .getByRole('combobox', { name: 'Person to invite', exact: true })
    .fill(process.env.MEALSHOP_READER_USER ?? '')
  await page
    .getByRole('option')
    .filter({ hasText: process.env.MEALSHOP_READER_USER ?? '' })
    .click()
  await page.getByRole('button', { name: 'Invite a person', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Pending invitations', exact: true }),
  ).toBeVisible()
  const context = await browser.newContext(),
    reader = await context.newPage()
  await login(reader, true)
  await reader
    .locator('.inline-row')
    .filter({ hasText: household.name })
    .getByRole('button', { name: 'Accept invitation', exact: true })
    .click()
  await reader.locator('.record-list a').filter({ hasText: household.name }).click()
  await expect(reader).toHaveURL(new RegExp(`/h/${household.id}/recipes$`))
  await reader.goto(`${embedded}/h/${household.id}/history`)
  await expect(reader.getByRole('heading', { name: 'Cooking history', exact: true })).toBeVisible()
  await expect(
    reader.getByRole('link', { name: 'Record a completed cook', exact: true }),
  ).toHaveCount(0)
  await expect(
    reader.getByText('The household owner manages learning and deletion of all history.', {
      exact: true,
    }),
  ).toBeVisible()
  await reader.locator('.record-list a').filter({ hasText: title }).click()
  await expect(reader.getByRole('link', { name: 'Edit cooking record', exact: true })).toHaveCount(
    0,
  )
  await expect(reader.getByText('Effort not recorded', { exact: true })).toBeVisible()
  await reader.goto(`${embedded}/h/${household.id}/history/new`)
  await expect(reader.getByRole('alert')).toBeVisible()
  await expect(reader.getByRole('button', { name: 'Save changes', exact: true })).toHaveCount(0)
  await context.close()
})

test('learned suggestions retain hard constraints, discovery and reviewed planning after opt-out', async ({
  page,
}) => {
  test.setTimeout(240000)
  await login(page)
  await home(page)
  await recipe(page)
  const good = 'Alternate history acceptance recipe',
    fresh = 'New history acceptance recipe'
  await recipe(page, good, 'Drinking water')
  await learning(page)
  await beginRecord(page, title, '1')
  await finishRecord(page)
  await beginRecord(page, good, '5')
  await finishRecord(page, good)
  await recipe(page, fresh)
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await page.getByRole('button', { name: 'Find suitable recipes', exact: true }).click()
  const rows = page.locator('.suggestion-item')
  await expect(rows).toHaveCount(3)
  await expect(rows.first()).toContainText(good)
  await expect(rows.first()).toContainText('1 completed cook')
  await expect(rows.filter({ hasText: fresh })).toContainText('Try a recipe')
  await audit(page, 'suggestions')
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page
    .getByRole('combobox', { name: 'Excluded ingredients', exact: true })
    .fill('Drinking water')
  await page.getByRole('option', { name: 'Drinking water', exact: true }).click()
  await page.getByRole('combobox', { name: 'Excluded ingredients', exact: true }).press('Escape')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await page.getByRole('button', { name: 'Find suitable recipes', exact: true }).click()
  await expect(rows).toHaveCount(2)
  await expect(rows.filter({ hasText: good })).toHaveCount(0)
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()
  await page.getByRole('link', { name: 'Cooking history', exact: true }).click()
  page.once('dialog', (d) => void d.accept())
  await page.getByRole('button', { name: 'Reset and turn off learning', exact: true }).click()
  await expect(page.getByText('Learning is off', { exact: true })).toBeVisible()
  await expect(page.locator('.record-list li')).toHaveCount(2)
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await page.getByRole('button', { name: 'Find suitable recipes', exact: true }).click()
  await expect(
    page.getByText('Rule-based suggestions; learning is off.', { exact: true }),
  ).toBeVisible()
  await expect(rows.first()).toContainText(title)
  await expect(rows.filter({ hasText: good })).toHaveCount(0)
  await rows.first().getByRole('button', { name: 'Review meal', exact: true }).click()
  await expect(
    page.getByRole('button', { name: 'Apply reviewed plan', exact: true }),
  ).toBeDisabled()
  await check(
    page.getByRole('checkbox', {
      name: 'I reviewed the unknown dietary data and serving/time uncertainty.',
      exact: true,
    }),
  )
  await page.getByRole('button', { name: 'Apply reviewed plan', exact: true }).click()
  await page.getByRole('link', { name: 'Cooking history', exact: true }).click()
  await expect(page.locator('.record-list li')).toHaveCount(2)
})
