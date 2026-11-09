import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { appendFileSync, mkdirSync } from 'node:fs'

test.beforeEach(async ({ page }) => {
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  expect(id).toBeTruthy()
  expect(id).not.toBe('home-1')
  mkdirSync('var/nextcloud', { recursive: true, mode: 0o700 })
  appendFileSync(
    'var/nextcloud/browser-fixtures.jsonl',
    JSON.stringify({ id, source: 'shopping.spec.ts' }) + '\n',
    { mode: 0o600 },
  )
})

test('shopping quantities, purchased provenance, recipe rescaling and selected plan sync work together', async ({
  page,
}) => {
  test.setTimeout(180000)
  page.on('dialog', (dialog) => void dialog.accept())
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Shopping acceptance recipe')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.000001')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Prepare test recipe.')
  await page.getByLabel('Active time (seconds)', { exact: true }).fill('60')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Shopping acceptance recipe', exact: true }),
  ).toBeVisible()
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await expect(
    page.getByRole('note', { name: 'Your shopping list is empty', exact: true }),
  ).toBeVisible()
  await page.getByRole('button', { name: 'Add recipe quantities', exact: true }).click()
  const recipeForm = page
    .locator('form')
    .filter({ has: page.getByRole('heading', { name: 'Add recipe quantities', exact: true }) })
  await recipeForm.getByLabel('Servings', { exact: true }).fill('4')
  await recipeForm.getByRole('button', { name: 'Save changes', exact: true }).click()
  const oats = page
    .locator('.shopping-items > li')
    .filter({ has: page.getByRole('checkbox', { name: /Oats/ }) })
  await expect(oats).toContainText('200.000002 g')
  await oats.getByText('Oats · 200.000002 g', { exact: true }).click()
  await page
    .locator('summary')
    .filter({ hasText: /^Purchased/ })
    .click()
  await expect(oats).toContainText('200.000002 g')
  const manual = page.locator('form').filter({ has: page.getByLabel('Name', { exact: true }) })
  await manual.getByLabel('Name', { exact: true }).fill('Test kitchen roll')
  await page.getByLabel('Language', { exact: true }).selectOption('de')
  await manual.getByLabel('Menge', { exact: true }).fill('1.5')
  await expect(
    manual.getByRole('button', { name: 'Artikel hinzufügen', exact: true }),
  ).toBeDisabled()
  await page.getByLabel('Sprache', { exact: true }).selectOption('en')
  await expect(manual.getByRole('button', { name: 'Add item', exact: true })).toBeDisabled()
  await manual.getByLabel('Quantity', { exact: true }).fill('2')
  await manual.getByRole('button', { name: 'Add item', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Test kitchen roll/ })).toBeVisible()
  const sources = page
    .locator('section')
    .filter({ has: page.getByRole('heading', { name: 'List sources', exact: true }) })
  const recipeSource = sources.locator('li').filter({ hasText: 'Shopping acceptance recipe' })
  await recipeSource.getByRole('button', { name: 'Edit recipe', exact: true }).click()
  await recipeForm.getByLabel('Servings', { exact: true }).fill('6')
  await recipeForm.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(oats).toContainText('100.000001 g')
  await expect(oats).toContainText('Purchased: 200.000002 g')
  await oats.getByText('Recipes and contributions', { exact: true }).click()
  await expect(oats).toContainText('Shopping acceptance recipe')
  await recipeSource.getByRole('button', { name: 'Remove', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Test kitchen roll/ })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('checkbox', { name: /Test kitchen roll/ })).toBeVisible()
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).fill('Test attendee')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  await page
    .getByText('I reviewed the unknown dietary data and serving/time uncertainty.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Apply reviewed plan', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Plan a meal', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'From meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Update plan contributions', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Oats/ })).toHaveCount(1)
  await page.getByRole('button', { name: 'Shopping mode', exact: true }).click()
  await expect(page.getByLabel('Name', { exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Manage list', exact: true })).toBeVisible()
  for (const width of [1440, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    expect(
      await page.locator('.workspace').evaluate((node) => node.scrollWidth <= node.clientWidth + 1),
    ).toBe(true)
  }
  expect(
    (
      await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.screenshot({ path: 'test-results/shopping-320.png', animations: 'disabled' })
  expect(errors).toEqual([])
})
