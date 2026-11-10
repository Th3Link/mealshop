import { recordFixture, openFixture } from './fixtures'
import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
test.describe.configure({ mode: 'serial' })
test('native household, recipe, exact serving preview, profiles and plan review', async ({
  page,
}) => {
  const errors: string[] = []
  let catalogueRequests = 0
  page.on('request', (request) => {
    if (request.url().endsWith('/api/ingredients')) catalogueRequests++
  })
  page.on('pageerror', (e) => {
    errors.push(e.message)
    console.error(e.stack)
  })
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text())
  })
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click({ timeout: 30000 })
  await expect(page.getByRole('heading', { name: 'Your households', exact: true })).toBeVisible()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'nextcloud.spec.ts')
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
  const homePath = new URL(page.url()).pathname.replace(/\/recipes$/, '')
  await page.getByText('Create recipe', { exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Exact oat breakfast')
  await page.getByText('Optional details', { exact: true }).click()
  await page.getByLabel('Author', { exact: true }).fill('Browser verification')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.000001')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Mix and serve.')
  await page.getByRole('combobox', { name: 'Active work · Unit', exact: true }).selectOption('1')
  await page.getByLabel('Active work', { exact: true }).fill('60')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Exact oat breakfast', exact: true }),
  ).toBeVisible()
  await expect(page.getByText(/^100\.000001 g · Oats/)).toBeVisible()
  await page.getByLabel('Preview servings', { exact: true }).fill('3')
  await Promise.all([
    page.waitForResponse((r) => r.url().includes('/scale?') && r.status() === 200),
    page.getByRole('button', { name: 'Preview', exact: true }).click(),
  ])
  await expect(page.getByText(/^300\.000003 g · Oats/)).toBeVisible()
  await expect(page.locator('.mealshop [role="alert"]')).toHaveCount(0)
  await page.getByRole('combobox', { name: 'Recipe mass display', exact: true }).selectOption('kg')
  await expect(page.getByText(/^0\.300000003 kg · Oats/)).toBeVisible()
  await page.getByRole('combobox', { name: 'Recipe mass display', exact: true }).selectOption('g')
  await expect(page.getByText(/^300\.000003 g · Oats/)).toBeVisible()
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).fill('Test attendee')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await expect(page.getByRole('button', { name: 'Add dietary profile', exact: true })).toBeEnabled()
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Shopping quantity changes', exact: true }),
  ).toBeVisible()
  await page
    .getByText('I reviewed the unknown dietary data and serving/time uncertainty.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Apply reviewed plan', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Plan a meal', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await page.getByLabel('Search ingredients', { exact: true }).fill('oats')
  await page.locator('.record-list').getByRole('link').first().click()
  await expect(page.getByRole('heading', { name: 'Nutrient values', exact: true })).toBeVisible()
  expect(catalogueRequests).toBe(0)
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Nutrient values', exact: true })).toBeVisible()
  for (const width of [1440, 1024, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    await expect
      .poll(() => page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth))
      .toBe(true)
    await expect
      .poll(() =>
        page
          .locator('.detail-columns > section')
          .evaluateAll((es) =>
            es.every((e) => e.getBoundingClientRect().right <= window.innerWidth),
          ),
      )
      .toBe(true)
    await page.screenshot({ path: `test-results/catalogue-${width}.png`, fullPage: true })
  }
  await page.setViewportSize({ width: 1440, height: 1000 })
  const axe = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(axe.violations).toEqual([])
  expect(errors).toEqual([])
  await page.goto(homePath + '/recipes')
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
})

test('addressed invitation, native reader access, revocation and stale revision recovery', async ({
  browser,
  page,
}) => {
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click({ timeout: 30000 })
  await expect(page.getByRole('heading', { name: 'Your households', exact: true })).toBeVisible()
  await openFixture(page, 'nextcloud.spec.ts')
  const homePath = new URL(page.url()).pathname.replace(/\/recipes$/, '')
  const homeName = (await page.locator('.context').innerText()).split('·')[0]?.trim() ?? ''
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  page.on('dialog', (dialog) => dialog.accept())
  const oldAccess = page
    .locator('.access-list li')
    .filter({ hasText: process.env.MEALSHOP_READER_USER ?? '' })
  if (await oldAccess.count())
    await oldAccess.getByRole('button', { name: 'Revoke access', exact: true }).click()
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
  const readerContext = await browser.newContext()
  const reader = await readerContext.newPage()
  await reader.goto(
    (process.env.MEALSHOP_TEST_URL ?? '') +
      '/index.php/apps/app_api/embedded/mealshop/main/households',
  )
  await reader.locator('#user').fill(process.env.MEALSHOP_READER_USER ?? '')
  await reader.locator('#password').fill(process.env.MEALSHOP_READER_PASSWORD ?? '')
  await reader.getByRole('button', { name: 'Log in', exact: true }).click({ timeout: 30000 })
  await reader
    .locator('[role=dialog]#firstrunwizard')
    .waitFor({ state: 'visible', timeout: 5000 })
    .catch(() => {})
  if (await reader.locator('[role=dialog]#firstrunwizard').isVisible())
    await reader
      .locator('[role=dialog]#firstrunwizard')
      .getByRole('button', { name: 'Close', exact: true })
      .click()
  await reader
    .locator('.inline-row')
    .filter({ hasText: homeName })
    .getByRole('button', { name: 'Accept invitation', exact: true })
    .click()
  await reader.locator('.record-list').getByRole('link').filter({ hasText: homeName }).click()
  await expect(
    reader.getByText('You have read-only access to this household.', { exact: true }),
  ).toBeVisible()
  await expect(reader.getByText('Create recipe', { exact: true })).toHaveCount(0)
  await reader.locator('.record-list').getByRole('link').first().click()
  await expect(reader.getByText('Edit recipe', { exact: true })).toHaveCount(0)
  await reader.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await expect(reader.getByRole('heading', { name: 'Shopping list', exact: true })).toBeVisible()
  await expect(reader.getByRole('button', { name: 'Add item', exact: true })).toHaveCount(0)
  await expect(
    reader.getByRole('button', { name: 'Add packaged product', exact: true }),
  ).toHaveCount(0)
  await reader.getByRole('link', { name: 'Household', exact: true }).click()
  await expect(reader.getByLabel('Name', { exact: true })).toBeDisabled()
  await expect(reader.getByText('Invite a person', { exact: true })).toHaveCount(0)
  await page.getByRole('button', { name: 'Refresh access', exact: true }).click()
  await page
    .locator('.access-list li')
    .filter({ hasText: process.env.MEALSHOP_READER_USER ?? '' })
    .getByRole('button', { name: 'Revoke access', exact: true })
    .click()
  await expect(
    page.locator('.access-list li').filter({ hasText: process.env.MEALSHOP_READER_USER ?? '' }),
  ).toHaveCount(0)
  await reader.reload()
  await expect(reader.getByRole('alert')).toContainText(
    'You no longer have access to this household, or your role does not permit this action.',
  )
  await readerContext.close()
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.locator('.record-list').getByRole('link').first().click()
  const recipeUrl = page.url()
  await page.getByText('Edit recipe', { exact: true }).click()
  const stale = await page.context().newPage()
  await stale.goto(recipeUrl + '?edit=1')
  await expect(stale.getByLabel('Recipe title', { exact: true })).toHaveValue('Exact oat breakfast')
  await page.getByLabel('Recipe title', { exact: true }).fill('Revised oat breakfast')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Revised oat breakfast', exact: true }),
  ).toBeVisible()
  await stale.getByLabel('Recipe title', { exact: true }).fill('Stale draft retained')
  await stale.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(stale.getByRole('alert')).toContainText(
    'The data changed in another session. Reload and review the current revision before saving.',
  )
  await expect(stale.getByLabel('Recipe title', { exact: true })).toHaveValue(
    'Stale draft retained',
  )
  await stale.close()
  await page.goto(homePath + '/recipes')
})

test('German quantities, curated source portions and hard dietary conflicts remain explicit', async ({
  page,
}) => {
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click({ timeout: 30000 })
  await openFixture(page, 'nextcloud.spec.ts')
  await page.getByText('Create recipe', { exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Quantity verification')
  await page.getByText('Optional details', { exact: true }).click()
  await page.getByLabel('Author', { exact: true }).fill('Browser verification')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.000001')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Mix and serve.')
  await page.getByRole('combobox', { name: 'Active work · Unit', exact: true }).selectOption('1')
  await page.getByLabel('Active work', { exact: true }).fill('60')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Quantity verification', exact: true }),
  ).toBeVisible()
  await page.getByText('Edit recipe', { exact: true }).click()
  await page.getByRole('combobox', { name: 'Language', exact: true }).first().selectOption('de')
  await expect(page.getByLabel('Gramm', { exact: true })).toHaveValue('100,000001')
  await page.getByRole('combobox', { name: 'Einheit', exact: true }).selectOption('kg')
  await page.getByLabel('Menge', { exact: true }).fill('0,100001')
  await page.getByRole('button', { name: 'Änderungen speichern', exact: true }).click()
  await expect(page.getByText(/100,001 g · Haferkorn/)).toBeVisible()
  await expect(page.getByText('Originaleingabe: 0,100001 kg', { exact: false })).toBeVisible()
  await page.getByText('Rezept bearbeiten', { exact: true }).click()
  await page.getByRole('combobox', { name: 'Zutat auswählen', exact: true }).fill('Kümmel')
  await page.getByRole('option', { name: 'Kümmel Samen', exact: true }).click()
  await page.getByRole('combobox', { name: 'Einheit', exact: true }).selectOption('tsp')
  await page.getByLabel('Menge', { exact: true }).fill('1')
  await page
    .getByRole('combobox', { name: 'Umrechnungsbeleg', exact: true })
    .selectOption({ index: 1 })
  await page.getByRole('button', { name: 'Änderungen speichern', exact: true }).click()
  await expect(page.locator('.plain-list').getByText(/^2,1 g/)).toBeVisible()
  await page.reload()
  await expect(page.getByText('Originaleingabe: 1 tsp', { exact: false })).toBeVisible()
  await page.getByRole('link', { name: 'Haushalt', exact: true }).click()
  const exclusions = page.getByRole('combobox', { name: 'Ausgeschlossene Zutaten', exact: true })
  await exclusions.fill('Kümmel')
  await page.getByRole('option', { name: 'Kümmel Samen', exact: true }).click()
  const savedProfiles = page.waitForResponse(
    (r) => r.url().endsWith('/api/household/profiles') && r.request().method() === 'PUT',
  )
  await page.getByRole('button', { name: 'Änderungen speichern', exact: true }).click()
  expect((await (await savedProfiles).json()).members[0].excluded_ingredients).toContain(
    'caraway-seed',
  )

  await expect(
    page.getByRole('button', { name: 'Änderungen speichern', exact: true }),
  ).toBeDisabled()
  await expect(
    page.getByRole('button', { name: 'Ernährungsprofil hinzufügen', exact: true }),
  ).toBeEnabled()
  await page.getByRole('link', { name: 'Speiseplan', exact: true }).click()
  await page.getByRole('button', { name: 'Mahlzeit planen', exact: true }).click()
  await page
    .getByRole('combobox', { name: 'Rezepte', exact: true })
    .selectOption({ label: 'Quantity verification · Version 3' })
  await page.getByRole('button', { name: 'Mahlzeit prüfen', exact: true }).click()
  await expect(
    page.getByText(
      'Für eine mitessende Person enthält dieses Rezept einen harten Ernährungsausschluss.',
      { exact: true },
    ),
  ).toBeVisible()
  await expect(
    page.getByRole('button', { name: 'Geprüften Plan übernehmen', exact: true }),
  ).toBeDisabled()
  expect(errors).toEqual([])
})
