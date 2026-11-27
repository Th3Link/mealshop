import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'

test('pasted recipe text is reviewed, saved with an own unit weight and rated', async ({
  page,
}) => {
  test.setTimeout(180000)
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'guided-capture.spec.ts')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).last().fill('Capture attendee')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()

  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.getByRole('link', { name: 'Add recipe', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Add recipe', exact: true })).toBeVisible()
  const chooser = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(chooser.violations).toEqual([])
  await page.getByRole('button', { name: /^Paste or dictate/ }).click()
  await page
    .getByLabel('Recipe text', { exact: true })
    .fill(
      [
        'Synthetic capture bowl',
        'for 2 people',
        '',
        'Ingredients',
        '1,500 g Oats',
        '1/2 tbsp Oats',
        '1/3 g Oats',
        '2 synthetic unknown food',
        '',
        'Method',
        '1. Mix everything.',
      ].join('\n'),
    )
  const found = page.locator('.capture-found')
  await expect(found).toContainText('Synthetic capture bowl · 2 servings')
  await expect(found).toContainText('0.5 tbsp Oats')
  // Neither a grouped-looking decimal nor a repeating fraction is turned into a number.
  await expect(found).toContainText('1,500 g Oats')
  await expect(found).toContainText('1/3 g Oats')
  await page.getByRole('button', { name: 'Review recipe', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Review and save', exact: true })).toBeVisible()
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(
    'Synthetic capture bowl',
  )
  await expect(page.locator('.import-source-line')).toHaveCount(4)
  const rows = page.locator('.ingredient-entry')
  await expect(rows.nth(0).getByLabel('Grams', { exact: true })).toHaveValue('')
  await rows.nth(0).getByLabel('Grams', { exact: true }).fill('100.5')
  await rows.nth(2).getByLabel('Grams', { exact: true }).fill('0.333333')
  // The unidentified line cannot be saved: it is identified or removed by the cook.
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Review and save', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Remove 4', exact: true }).click()
  await expect(page.getByLabel('Weight of 1 tbsp in grams', { exact: true })).toHaveAttribute(
    'required',
  )
  await page.getByLabel('Weight of 1 tbsp in grams', { exact: true }).fill('7.5')
  const review = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(review.violations).toEqual([])
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Synthetic capture bowl', exact: true }),
  ).toBeVisible()
  await expect(page.getByText(/^3\.75 g · Oats/)).toContainText('own weight')
  await expect(page.locator('.nutrition-tiles li')).toHaveCount(5)

  await page.getByRole('button', { name: 'Cooked it? Rate it', exact: true }).click()
  await page
    .getByRole('group', { name: 'Household rating', exact: true })
    .getByRole('radio', { name: '4 / 5', exact: true })
    .check({ force: true })
  await page.getByRole('button', { name: 'Save rating', exact: true }).click()
  await expect(page.getByText('Saved to your cooking history.', { exact: true })).toBeVisible()
  await expect(page.getByRole('img', { name: '4 out of 5 stars', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  const card = page.locator('.recipe-cards li').filter({ hasText: 'Synthetic capture bowl' })
  await expect(card.getByRole('img', { name: '4 out of 5 stars', exact: true })).toBeVisible()
  await expect(card).toContainText('Cooked 1×')
  for (const width of [390, 1440]) {
    await page.setViewportSize({ width, height: 900 })
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    ).toBe(true)
  }
  expect(errors).toEqual([])
})
