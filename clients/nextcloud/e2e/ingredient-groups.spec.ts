import { openFixture } from './fixtures'
import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
test('groups, declared nutrient values and explicit subtype inheritance persist', async ({
  page,
}) => {
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await openFixture(page, 'authoring-experience.spec.ts')
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await page.getByLabel('Search ingredients', { exact: true }).fill('Wheat flour')
  await expect(page.locator('.record-list')).toContainText('Wheat flour')
  const stamp = Date.now(),
    group = `Flour group ${stamp}`,
    child = `Type test ${stamp}`
  await page.getByRole('button', { name: 'Create ingredient', exact: true }).click()
  let dialog = page.getByRole('dialog')
  await dialog.getByLabel('Ingredient name', { exact: true }).fill(group)
  await dialog.getByLabel('Form or product description', { exact: true }).fill('Flour')
  await dialog.getByText('Classification and composition', { exact: true }).click()
  await dialog.getByText('This is a group', { exact: true }).click()
  await dialog
    .getByRole('combobox', { name: 'Dietary classification', exact: true })
    .selectOption('vegan')
  await dialog
    .getByRole('combobox', { name: 'Ingredient origin', exact: true })
    .selectOption('plant')
  await dialog
    .getByRole('combobox', { name: 'Nutrient values per 100 g', exact: true })
    .selectOption('own')
  await dialog
    .getByLabel('Nutrient source / evidence note', { exact: true })
    .fill('Fictional packaging for automated testing')
  await dialog.getByLabel('Protein (g)', { exact: true }).fill('12.5')
  await dialog.getByLabel('Fat (g)', { exact: true }).fill('-1')
  await expect(dialog.getByLabel('Fat (g)', { exact: true })).toHaveJSProperty(
    'validationMessage',
    'Use zero or a positive decimal value with at most six decimal places, without grouping separators.',
  )
  await dialog.getByLabel('Fat (g)', { exact: true }).fill('0')
  await expect(dialog.getByLabel('Fat (g)', { exact: true })).toHaveJSProperty(
    'validationMessage',
    '',
  )
  const audit = await new AxeBuilder({ page })
    .include('[role="dialog"]')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(audit.violations).toEqual([])
  await page.setViewportSize({ width: 390, height: 844 })
  await page.screenshot({
    path: 'test-results/ingredient-editor-mobile.png',
    animations: 'disabled',
  })
  expect(await dialog.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true)
  await page.setViewportSize({ width: 1440, height: 1000 })
  await dialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(dialog).toHaveCount(0)
  await expect(page.getByRole('heading', { name: group, exact: true })).toBeVisible()
  const homePath = new URL(page.url()).pathname.split('/ingredients')[0]
  await page.goto(homePath + '/ingredients')
  await page.getByRole('button', { name: 'Create ingredient', exact: true }).click()
  dialog = page.getByRole('dialog')
  await dialog.getByLabel('Ingredient name', { exact: true }).fill(child)
  await dialog.getByLabel('Form or product description', { exact: true }).fill('Flour subtype')
  await dialog.getByText('Classification and composition', { exact: true }).click()
  await dialog
    .getByRole('combobox', { name: 'Ingredient group', exact: true })
    .selectOption({ label: group })
  await dialog
    .getByRole('combobox', { name: 'Nutrient values per 100 g', exact: true })
    .selectOption('inherit')
  await dialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(dialog).toHaveCount(0)
  await expect(
    page.getByText(`Nutrient values inherited from: ${group}`, { exact: false }),
  ).toBeVisible()
  await expect(page.getByRole('row').filter({ hasText: 'Protein' })).toContainText('12.5')
  await page.reload()
  await expect(page.getByRole('row').filter({ hasText: 'Protein' })).toContainText('12.5')
  await page.getByRole('button', { name: 'Edit ingredient', exact: true }).click()
  dialog = page.getByRole('dialog')
  await expect(
    dialog.getByRole('combobox', { name: 'Nutrient values per 100 g', exact: true }),
  ).toHaveValue('inherit')
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click()
  await page.goto(homePath + '/ingredients')
  await page.getByLabel('Search ingredients', { exact: true }).fill(group)
  await page.getByRole('link', { name: group, exact: true }).first().click()
  page.once('dialog', (prompt) => prompt.accept())
  await page.getByRole('button', { name: 'Archive ingredient', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Restore ingredient', exact: true })).toBeVisible()
  await page.goto(homePath + '/ingredients')
  await expect(page.getByRole('link').filter({ hasText: child })).toBeVisible()
  await page.getByRole('link').filter({ hasText: child }).click()
  await page.getByRole('button', { name: 'Edit ingredient', exact: true }).click()
  dialog = page.getByRole('dialog')
  await dialog.getByText('Classification and composition', { exact: true }).click()
  await expect(dialog.getByRole('combobox', { name: 'Ingredient group', exact: true })).toHaveValue(
    /custom-/,
  )
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click()
  expect(errors).toEqual([])
  await page.screenshot({
    path: 'test-results/ingredient-inheritance.png',
    fullPage: true,
    animations: 'disabled',
  })
})
