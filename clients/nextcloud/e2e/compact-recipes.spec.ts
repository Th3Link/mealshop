import { test, expect, type Dialog } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'

test('compact open recipe parts preserve exact homemade inputs through editing and shopping', async ({
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
  await recordFixture(page, 'compact-recipes.spec.ts')
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Compact recipe acceptance')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  const search = page.getByRole('combobox', { name: 'Choose ingredient', exact: true })
  await search.fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('60.000001')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Use the prepared component.')
  await expect(page.getByLabel('Temperature (°C)', { exact: true })).toHaveCount(0)
  const extras = page.getByRole('button', { name: 'Ingredient options 1', exact: true })
  await extras.focus()
  await page.keyboard.press('Enter')
  await expect(page.getByRole('menuitem', { name: 'Temperature (°C)', exact: true })).toBeFocused()
  await page.getByRole('menuitem', { name: 'Temperature (°C)', exact: true }).click()
  await expect(page.getByRole('menuitem')).toHaveCount(0)
  await page.getByLabel('Temperature (°C)', { exact: true }).fill('22.000001')
  await extras.click()
  await page.getByRole('menuitem', { name: 'Prepare at home', exact: true }).click()
  const producer = page.getByRole('region', { name: 'Oats', exact: true })
  await expect(producer).toBeVisible()
  await expect(producer.getByLabel('Grams', { exact: true })).toHaveValue('')
  await expect(
    producer.getByRole('combobox', { name: 'Choose ingredient', exact: true }),
  ).toBeFocused()
  await producer
    .getByRole('textbox', { name: 'Part name', exact: true })
    .fill('Measured preparation')
  const part = page.getByRole('region', { name: 'Measured preparation', exact: true })
  await part.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('chia')
  await page.getByRole('option', { name: 'Chia seeds', exact: true }).click()
  await part.getByLabel('Grams', { exact: true }).fill('100.000001')
  await part
    .getByLabel('2. Instruction', { exact: true })
    .fill('Prepare the measured test ingredient. No real recipe is implied.')
  const unexpectedDialogs: string[] = []
  const rejectUnexpected = (dialog: Dialog) => {
    unexpectedDialogs.push(dialog.message())
    void dialog.dismiss()
  }
  page.on('dialog', rejectUnexpected)
  const partId = await part.getAttribute('id')
  await page.locator('.prepared-reference').click()
  await expect(page).toHaveURL(new RegExp(`#${partId}$`))
  expect(unexpectedDialogs).toEqual([])
  await expect(part).toBeFocused()
  await expect(part.getByLabel('Grams', { exact: true })).toHaveValue('100.000001')
  page.off('dialog', rejectUnexpected)
  const usedParts = page
    .locator('.step-editor')
    .first()
    .getByRole('combobox', { name: 'Prepared parts used in this step', exact: true })
  await usedParts.fill('Measured preparation')
  await page
    .getByRole('listbox')
    .getByRole('option', { name: 'Measured preparation', exact: true })
    .click()
  await page.keyboard.press('Escape')
  await page.getByRole('button', { name: 'Step options 1', exact: true }).click()
  await expect(
    page.getByRole('menuitem', { name: 'Move to recipe part · Measured preparation', exact: true }),
  ).toBeDisabled()
  await page.keyboard.press('Escape')
  await expect(page.getByRole('menuitem')).toHaveCount(0)
  await extras.click()
  await page.getByRole('menuitem', { name: 'Use purchased ingredient', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Resolve')
  await expect(part).toBeVisible()
  await expect(page.getByRole('menuitem')).toHaveCount(0)
  await usedParts.focus()
  await page.keyboard.press('Backspace')
  await page.keyboard.press('Escape')
  await expect(page.locator('.recipe-editor details')).toHaveCount(0)
  const label = page.locator('.ingredient-select > label').first()
  expect(await label.evaluate((e) => getComputedStyle(e).position)).toBe('absolute')
  const columns = page.locator('.recipe-composition').first()
  const bounds = await columns
    .locator(':scope > div')
    .evaluateAll((elements) =>
      elements.map((e) => ({ x: e.getBoundingClientRect().x, y: e.getBoundingClientRect().y })),
    )
  expect(bounds[1]?.x).toBeGreaterThan(bounds[0]?.x ?? 0)
  expect(bounds[1]?.y).toBe(bounds[0]?.y)
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    for (const scheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
      for (const width of [1440, 768, 390, 320]) {
        await page.setViewportSize({ width, height: 1000 })
        await expect
          .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(width)
        await page.locator('.recipe-title-row').scrollIntoViewIfNeeded()
        await page.screenshot({
          path: `test-results/compact-${language}-${scheme}-${width}.png`,
          fullPage: true,
          animations: 'disabled',
        })
      }
      const axe = await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
      expect(axe.violations).toEqual([])
    }
  }
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.locator('#mealshop-language').selectOption('en')
  await page.emulateMedia({ colorScheme: 'light' })
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Compact recipe acceptance', exact: true }),
  ).toBeVisible()
  await expect(page.locator('.prepared-reference')).toContainText('Homemade · Measured preparation')
  await expect(page.locator('.recipe-part-content')).toHaveCount(2)
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    for (const scheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
      for (const width of [1440, 390]) {
        await page.setViewportSize({ width, height: 1000 })
        await expect
          .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(width)
        await page.screenshot({
          path: `test-results/compact-read-${language}-${scheme}-${width}.png`,
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
  await page.emulateMedia({ colorScheme: 'light' })
  await page.screenshot({
    path: 'test-results/compact-reading.png',
    fullPage: true,
    animations: 'disabled',
  })
  await page.getByRole('link', { name: 'Edit recipe', exact: true }).click()
  await expect(page.getByLabel('Grams', { exact: true }).first()).toHaveValue('60.000001')
  await expect(page.getByLabel('Grams', { exact: true }).nth(1)).toHaveValue('100.000001')
  await expect(page.getByLabel('Temperature (°C)', { exact: true })).toHaveValue('22.000001')
  // Cancellation retains both measured inputs and instructions.
  page.once('dialog', (dialog) => void dialog.dismiss())
  await extras.click()
  await page.getByRole('menuitem', { name: 'Use purchased ingredient', exact: true }).click()
  await expect(part).toBeVisible()
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'Manage list', exact: true }).click()
  await page.getByRole('button', { name: 'Add recipe quantities', exact: true }).click()
  await page.getByLabel('Servings', { exact: true }).fill('4')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('checkbox', { name: 'Chia seeds · 200.000002 g', exact: true }),
  ).toBeVisible()
  await expect(page.getByRole('checkbox', { name: /Oats ·/ })).toHaveCount(0)
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.locator('.record-list a').filter({ hasText: 'Compact recipe acceptance' }).click()
  await page.getByRole('link', { name: 'Edit recipe', exact: true }).click()
  page.once('dialog', (dialog) => void dialog.accept())
  await extras.click()
  await page.getByRole('menuitem', { name: 'Use purchased ingredient', exact: true }).click()
  await expect(part).toHaveCount(0)
  await expect(page.getByLabel('Grams', { exact: true })).toHaveValue('60.000001')
  await expect(page.getByLabel('1. Instruction', { exact: true })).toHaveValue(
    'Use the prepared component.',
  )
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Compact recipe acceptance', exact: true }),
  ).toBeVisible()
  await page.reload()
  await expect(page.locator('.prepared-reference')).toHaveCount(0)
  await expect(page.locator('.recipe-part-content')).toHaveCount(1)
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Ingredients', exact: true })).toBeVisible()
  expect(errors).toEqual([])
})
