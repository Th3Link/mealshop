import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
test('direct ingredient creation, optional constituents and branched preparation retain their context', async ({
  page,
}) => {
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await page.getByText('Create recipe', { exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Structured authoring verification')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add recipe part', exact: true }).click()
  await page.getByLabel('Part name', { exact: true }).fill('Preparation')
  await page.getByRole('button', { name: 'Add recipe part', exact: true }).click()
  await page.getByLabel('Part name', { exact: true }).nth(1).fill('Assembly')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('haferflocken')
  await expect(page.getByRole('option', { name: 'Oat flakes', exact: true })).toBeVisible()
  await page.screenshot({
    path: 'test-results/authoring-ingredient-dropdown.png',
    animations: 'disabled',
  })
  await page.getByRole('option', { name: 'Oat flakes', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100')
  await page
    .getByRole('combobox', { name: 'Recipe part', exact: true })
    .selectOption({ label: 'Preparation' })
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  const ingredientName = `Inline constituent mix ${Date.now()}`
  const search = page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).nth(1)
  await search.fill(ingredientName)
  await page.getByRole('option').filter({ hasText: 'Create ingredient' }).click()
  const dialog = page.getByRole('dialog')
  await expect(dialog.getByLabel('Ingredient name', { exact: true })).toHaveValue(ingredientName)
  await dialog.getByLabel('Form or product description', { exact: true }).fill('As declared')
  await dialog
    .getByRole('combobox', { name: 'Nutrient values per 100 g', exact: true })
    .selectOption('own')
  await expect(dialog.getByLabel('Beta-glucan (g)', { exact: true })).toHaveCount(0)
  await dialog.getByRole('button', { name: 'Add constituent', exact: true }).click()
  await dialog.getByLabel('Beta-glucan (g)', { exact: true }).fill('2.5')
  await dialog
    .getByLabel('Nutrient source / evidence note', { exact: true })
    .fill('Synthetic declaration for browser testing')
  await dialog.getByText('Intolerance triggers', { exact: true }).click()
  await dialog.getByText('Sorbitol', { exact: true }).click()
  const audit = await new AxeBuilder({ page })
    .include('[role="dialog"]')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(audit.violations).toEqual([])
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.setViewportSize({ width: 390, height: 844 })
  await dialog.getByLabel('Ingredient name', { exact: true }).scrollIntoViewIfNeeded()
  await page.screenshot({
    path: 'test-results/authoring-dialog-dark-mobile.png',
    animations: 'disabled',
  })
  expect(await dialog.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true)
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.emulateMedia({ colorScheme: 'light' })
  await dialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(dialog).toHaveCount(0)
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(
    'Structured authoring verification',
  )
  await page.getByLabel('Grams', { exact: true }).nth(1).fill('50')
  await page
    .getByRole('combobox', { name: 'Recipe part', exact: true })
    .nth(1)
    .selectOption({ label: 'Assembly' })
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Prepare the first part.')
  let step = page.locator('.step-editor').first()
  await step
    .getByRole('combobox', { name: 'Recipe part', exact: true })
    .selectOption({ label: 'Preparation' })
  await step
    .getByRole('combobox', { name: 'Ingredients used in this step', exact: true })
    .fill('Oat flakes')
  await page.getByRole('option', { name: '1. Oat flakes', exact: true }).click()
  await page.getByRole('heading', { name: 'Preparation steps', exact: true }).click()
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('2. Instruction', { exact: true }).fill('Assemble the prepared parts.')
  step = page.locator('.step-editor').nth(1)
  await step
    .getByRole('combobox', { name: 'Recipe part', exact: true })
    .selectOption({ label: 'Assembly' })
  await step
    .getByRole('combobox', { name: 'Prepared parts used in this step', exact: true })
    .fill('Preparation')
  await page.getByRole('listbox').getByRole('option', { name: 'Preparation', exact: true }).click()
  await page.getByRole('heading', { name: 'Preparation steps', exact: true }).click()
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Structured authoring verification', exact: true }),
  ).toBeVisible()
  await expect(
    page
      .locator('.recipe-part-content')
      .filter({ has: page.getByRole('heading', { name: 'Assembly', exact: true }) }),
  ).toContainText('Prepared parts used in this step: Preparation')
  await expect(page.locator('.constituent-summary')).toContainText('1.25')
  await page.reload()
  await expect(page.locator('.constituent-summary')).toContainText('1.25')
  await page.screenshot({
    path: 'test-results/authoring-structured-recipe.png',
    animations: 'disabled',
  })
  await page.getByText('Edit recipe', { exact: true }).click()
  await expect(page.getByLabel('Part name', { exact: true }).first()).toHaveValue('Preparation')
  await expect(page.locator('.step-editor').nth(1)).toContainText('Preparation')
  for (const width of [1440, 1024, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    await expect
      .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
      .toBeLessThanOrEqual(width)
    await page.getByLabel('Recipe title', { exact: true }).scrollIntoViewIfNeeded()
    if (width === 1440 || width === 390)
      await page.screenshot({
        path: `test-results/authoring-editor-${width}.png`,
        animations: 'disabled',
      })
  }
  expect(errors).toEqual([])
})
