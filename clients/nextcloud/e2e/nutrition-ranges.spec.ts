import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'

test('comparison ranges cover complete recipes and explicit household ranges keep accessible input', async ({
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
  await recordFixture(page, 'nutrition-ranges.spec.ts')
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Nutrition range acceptance')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  for (const [index, name] of ['Oats', 'Psyllium husks'].entries()) {
    await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
    await page
      .getByRole('combobox', { name: 'Choose ingredient', exact: true })
      .nth(index)
      .fill(name)
    await page.getByRole('option', { name, exact: true }).click()
    await page.getByLabel('Grams', { exact: true }).nth(index).fill('100')
  }
  await expect(page.locator('.quantity-line .estimate-badge')).toHaveCount(1)
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Nutrition range acceptance', exact: true }),
  ).toBeVisible()
  await expect(page.locator('.recipe-part-content .estimate-badge')).toHaveCount(1)
  const energy = page
    .locator('tr.estimated-row')
    .filter({ has: page.getByRole('rowheader').filter({ hasText: 'kcal' }) })
  await expect(energy).toContainText('290–298.5')
  await expect(energy).toContainText('580–597')
  await expect(
    page.locator('.nutrition-evidence').filter({ hasText: 'Whole and milled psyllium' }),
  ).toContainText('Mivolis')
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    await expect(energy).toContainText(language === 'de' ? '290–298,5' : '290–298.5')
    for (const scheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
      for (const width of [1440, 390, 320]) {
        await page.setViewportSize({ width, height: 1000 })
        await expect
          .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(width)
        await energy.scrollIntoViewIfNeeded()
        await page.screenshot({
          path: `test-results/nutrition-${language}-${scheme}-${width}.png`,
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
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await page.getByRole('button', { name: 'Create ingredient', exact: true }).click()
  const dialog = page.getByRole('dialog')
  await dialog
    .getByLabel('Ingredient name', { exact: true })
    .fill('Browser nutrition range fixture')
  await dialog
    .getByLabel('Form or product description', { exact: true })
    .fill('Synthetic test input')
  await dialog
    .getByRole('combobox', { name: 'Nutrient values per 100 g', exact: true })
    .selectOption('own')
  await dialog.getByRole('button', { name: 'Enter a range · Energy (kcal)', exact: true }).click()
  const lower = dialog.getByLabel('Energy (kcal) · Lower value', { exact: true })
  const upper = dialog.getByLabel('Energy (kcal) · Upper value', { exact: true })
  await lower.fill('80.987654')
  await upper.fill('75.123456')
  await expect
    .poll(() => upper.evaluate((input) => (input as HTMLInputElement).validity.valid))
    .toBe(false)
  await lower.fill('75.123456')
  await upper.fill('80.987654')
  await expect(upper).toBeFocused()
  await expect(upper).toHaveValue('80.987654')
  await expect
    .poll(() => upper.evaluate((input) => (input as HTMLInputElement).validity.valid))
    .toBe(true)
  await dialog
    .getByLabel('Nutrient source / evidence note', { exact: true })
    .fill('Synthetic declared comparison for browser verification only')
  await page.setViewportSize({ width: 390, height: 844 })
  await lower.scrollIntoViewIfNeeded()
  expect(await dialog.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true)
  expect(
    (
      await new AxeBuilder({ page })
        .include('[role="dialog"]')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.screenshot({
    path: 'test-results/nutrition-range-input-dark-mobile.png',
    animations: 'disabled',
  })
  await dialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(dialog).toHaveCount(0)
  await expect(page.locator('tr.estimated-row').filter({ hasText: 'kcal' })).toContainText(
    '75.123456–80.987654',
  )
  await page.reload()
  await expect(page.locator('tr.estimated-row').filter({ hasText: 'kcal' })).toContainText(
    '75.123456–80.987654',
  )
  await page.setViewportSize({ width: 1440, height: 1000 })
  const homePath = new URL(page.url()).pathname.split('/ingredients')[0]
  await page.goto(`${homePath}/ingredients/custom-group-sweeteners`)
  await expect(
    page
      .getByRole('row')
      .filter({ has: page.getByRole('rowheader', { name: 'Sugars g', exact: true }) }),
  ).toContainText('0–100')
  await page.goto(`${homePath}/ingredients/custom-group-vinegar`)
  await expect(
    page
      .getByRole('row')
      .filter({ has: page.getByRole('rowheader', { name: 'Salt g', exact: true }) }),
  ).toContainText('0.045–0.208')
  expect(errors).toEqual([])
})
