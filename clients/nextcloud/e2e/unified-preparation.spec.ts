import { test, expect, type Page } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'
async function applyPlan(page: Page) {
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  const consent = page.getByText(
    'I reviewed the unknown dietary data and serving/time uncertainty.',
    { exact: true },
  )
  await expect(consent).toBeVisible()
  await consent.click()
  await page.getByRole('button', { name: 'Apply reviewed plan', exact: true }).click()
  await expect(page.locator('.mealplan-details')).toHaveCount(0)
}
test('multipart mealprep keeps exact temperatures, one production, allocations and actual-action waiting timers', async ({
  page,
}) => {
  test.setTimeout(180000)
  page.on('dialog', (d) => void d.accept())
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
  await recordFixture(page, 'unified-preparation.spec.ts')
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Preparation acceptance recipe')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add recipe part', exact: true }).click()
  await page.getByLabel('Part name', { exact: true }).fill('Starter')
  await page.getByRole('button', { name: 'Add recipe part', exact: true }).click()
  await page.getByLabel('Part name', { exact: true }).nth(1).fill('Dough')
  const starter = page
    .locator('.part-editor')
    .filter({ has: page.locator('summary').filter({ hasText: /^Starter$/ }) })
  const dough = page
    .locator('.part-editor')
    .filter({ has: page.locator('summary').filter({ hasText: /^Dough$/ }) })
  await starter.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await starter.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await starter.getByLabel('Grams', { exact: true }).fill('50.000001')
  await starter.getByRole('button', { name: 'Add temperature', exact: true }).click()
  await starter.getByLabel('Temperature (°C)', { exact: true }).fill('60.000001')
  await starter.getByRole('button', { name: 'Add step', exact: true }).click()
  await starter.getByLabel('1. Instruction', { exact: true }).fill('Mix test preparation')
  await starter.getByRole('combobox', { name: 'Active work · Unit', exact: true }).selectOption('1')
  await starter.getByLabel('Active work', { exact: true }).fill('120')
  await starter
    .getByRole('combobox', { name: 'Rest or waiting time · Unit', exact: true })
    .selectOption('1')
  await starter.getByLabel('Rest or waiting time', { exact: true }).fill('1')
  await dough.getByRole('combobox', { name: 'Uses prepared recipe parts', exact: true }).click()
  await page
    .getByRole('listbox', { name: 'Options', exact: true })
    .getByRole('option', { name: 'Starter', exact: true })
    .click()
  await page.keyboard.press('Escape')
  await dough.getByRole('button', { name: 'Add step', exact: true }).click()
  await dough.getByLabel('2. Instruction', { exact: true }).fill('Finish test preparation')
  await dough.getByRole('combobox', { name: 'Active work · Unit', exact: true }).selectOption('1')
  await dough.getByLabel('Active work', { exact: true }).fill('1')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Preparation acceptance recipe', exact: true }),
  ).toBeVisible()
  await expect(page.locator('.mealshop')).toContainText('60.000001 °C')
  await page.getByRole('link', { name: 'Edit recipe', exact: true }).click()
  await page
    .locator('summary')
    .filter({ hasText: /^Starter$/ })
    .click()
  await expect(page.getByLabel('Temperature (°C)', { exact: true })).toHaveValue('60.000001')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).fill('Test attendee')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Plan a meal', exact: true })).toBeFocused()
  let form = page.locator('.mealplan-details')
  await form.getByLabel('Date', { exact: true }).fill('2026-11-11')
  await form.locator('select').first().selectOption('preparation')
  await form.getByLabel('Production servings', { exact: true }).fill('4')
  await form.getByRole('button', { name: 'Prepare', exact: true }).click()
  await form.getByLabel('Preparation time', { exact: true }).fill('07:00')
  for (const date of ['2026-11-12', '2026-11-13']) {
    await form.getByRole('button', { name: 'Add eating day', exact: true }).click()
    const row = form.getByRole('group', { name: 'Eat on', exact: true }).locator('.form-row').last()
    await row.locator('input[type="date"]').fill(date)
    await row.locator('select').selectOption('lunch')
    await row.getByLabel('Servings', { exact: true }).fill('2')
  }
  await applyPlan(page)
  const thursday = page.locator('.meal-slot[data-date="2026-11-12"][data-meal="lunch"]')
  const friday = page.locator('.meal-slot[data-date="2026-11-13"][data-meal="lunch"]')
  await expect(thursday).toContainText('Preparation acceptance recipe')
  await expect(friday).toContainText('2 Servings')
  await thursday.getByRole('button', { name: /^View details/ }).click()
  form = page.locator('.mealplan-details')
  await form
    .getByRole('group', { name: 'Eat on', exact: true })
    .locator('.form-row')
    .last()
    .locator('input[type="date"]')
    .fill('2026-11-14')
  await applyPlan(page)
  await expect(friday).not.toContainText('Preparation acceptance recipe')
  await expect(page.locator('.meal-slot[data-date="2026-11-14"][data-meal="lunch"]')).toContainText(
    'Preparation acceptance recipe',
  )
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'Manage list', exact: true }).click()
  await page.getByRole('button', { name: 'From meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Update plan contributions', exact: true }).click()
  await expect(
    page.getByRole('checkbox', { name: 'Oats · 100.000002 g', exact: true }),
  ).toHaveCount(1)
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page
    .locator('.meal-slot[data-date="2026-11-11"][data-meal="preparation"]')
    .getByRole('button', { name: 'View details', exact: true })
    .click()
  await page
    .locator('.mealplan-details')
    .getByRole('link', { name: 'Cooking & baking', exact: true })
    .click()
  await expect(page.getByLabel('Name', { exact: true })).toHaveValue(
    'Preparation acceptance recipe',
  )
  await page.getByRole('button', { name: 'Review schedule', exact: true }).click()
  await page
    .getByText('I reviewed the times, dependencies and uncertainties.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save this schedule', exact: true }).click()
  await expect(page).toHaveURL(/\/cooking\/cook-\d+$/)
  const start = page.waitForResponse(
    (r) => r.request().method() === 'POST' && r.url().endsWith('/api/household/cooking'),
  )
  await page
    .locator('[data-cooking-step="s1-1"]')
    .getByRole('button', { name: 'Start step: Starter', exact: true })
    .click()
  expect((await start).ok()).toBe(true)
  await expect(page.locator('[data-cooking-step="s1-1"]')).toContainText('Time elapsed', {
    timeout: 8000,
  })
  await expect(
    page
      .locator('[data-cooking-step="s2-1"]')
      .getByRole('button', { name: 'Start step: Dough', exact: true }),
  ).toBeEnabled({ timeout: 8000 })
  await page
    .locator('[data-cooking-step="s2-1"]')
    .getByRole('button', { name: 'Start step: Dough', exact: true })
    .click()
  await expect(page.locator('[data-cooking-step="s1-1"]')).toContainText('Completed')
  await expect(page.locator('.cooking-view')).toContainText('60.000001 °C')
  for (const width of [1440, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    await expect(page.locator('#app-navigation-vue')).toHaveAttribute(
      'aria-hidden',
      String(width < 1024),
    )
    expect(await page.locator('.workspace').evaluate((e) => e.scrollWidth <= e.clientWidth)).toBe(
      true,
    )
    await page.screenshot({ path: `test-results/preparation-${width}.png`, animations: 'disabled' })
  }
  const axe = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(axe.violations).toEqual([])
  expect(errors).toEqual([])
})
