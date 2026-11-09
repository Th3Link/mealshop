import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

test.beforeEach(async ({ page }) => {
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('.record-list').getByRole('link').last().click()
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
})

test('search, filter recovery and localized ingredient detail remain usable', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  const search = page.getByLabel('Search recipes', { exact: true })
  await search.fill('no-such-recipe-ux-review')
  await expect(page.locator('.record-list li')).toHaveCount(0)
  await page.getByRole('button', { name: 'Clear search', exact: true }).last().click()
  const title = await page.locator('.record-list strong').first().innerText()
  await search.fill(title)
  await page.locator('.record-list a').first().click()
  await expect(page.getByRole('heading', { level: 1, name: title })).toBeFocused()
  await page.getByRole('link', { name: 'Back', exact: true }).click()
  await expect(search).toHaveValue(title)
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await page.getByText('Favourites only', { exact: true }).click()
  await page.getByLabel('Search ingredients', { exact: true }).fill('no-such-ingredient-ux-review')
  await page.getByRole('button', { name: 'Reset filters', exact: true }).click()
  await expect(page.getByLabel('Favourites only', { exact: true })).not.toBeChecked()
  await expect(page.getByLabel('Search ingredients', { exact: true })).toHaveValue('')
  await expect(page.locator('.record-list li').first()).toBeVisible()
  await page.getByLabel('Language', { exact: true }).selectOption('de')
  await page.getByLabel('Zutaten suchen', { exact: true }).fill('haferflocken')
  await page.locator('.record-list a').first().click()
  await expect(page.getByRole('rowheader', { name: 'Eiweiß g', exact: true })).toBeVisible()
  await expect(page.getByRole('rowheader', { name: 'Protein g', exact: true })).toHaveCount(0)
  for (const width of [1440, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 950 })
    expect(
      await page.locator('.workspace').evaluate((node) => node.scrollWidth <= node.clientWidth + 1),
    ).toBe(true)
  }
  expect(errors).toEqual([])
})

test('profile and meal drafts survive rejected discard and household switching', async ({
  page,
}) => {
  const homePath = new URL(page.url()).pathname.replace(/\/recipes$/, '')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  const name = page.getByLabel('Name', { exact: true }).first()
  const original = await name.inputValue()
  await name.fill(original + ' unsaved')
  await page.locator('.household-switcher').getByRole('combobox').click()
  page.once('dialog', (dialog) => void dialog.dismiss())
  await page.getByRole('listbox').getByRole('option').first().click()
  await expect(name).toHaveValue(original + ' unsaved')
  await expect(page).toHaveURL(homePath + '/household')
  page.once('dialog', (dialog) => void dialog.dismiss())
  await page.getByRole('button', { name: 'Discard changes', exact: true }).click()
  await expect(name).toHaveValue(original + ' unsaved')
  page.once('dialog', (dialog) => void dialog.accept())
  await page.getByRole('button', { name: 'Discard changes', exact: true }).click()
  await expect(name).toHaveValue(original)
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await page.getByLabel('Servings', { exact: true }).fill('7')
  page.once('dialog', (dialog) => void dialog.dismiss())
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()
  await expect(page.getByLabel('Servings', { exact: true })).toHaveValue('7')
  page.once('dialog', (dialog) => void dialog.accept())
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()
  await expect(page.getByLabel('Servings', { exact: true })).toHaveCount(0)
  const existing = page.getByRole('button', { name: /^View details/ }).first()
  await existing.click()
  await expect(page.getByRole('heading', { name: 'Meal details', exact: true })).toBeFocused()
  let release: (() => void) | undefined
  const pending = new Promise<void>((resolve) => {
    release = resolve
  })
  await page.route('**/api/household/mealplan/preview', async (route) => {
    await pending
    await route.continue()
  })
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  try {
    await expect(page.getByLabel('Servings', { exact: true })).toBeDisabled()
    await expect(page.getByRole('button', { name: 'Cancel', exact: true })).toBeDisabled()
  } finally {
    release?.()
  }
  await expect(page.locator('#mealshop-review-title')).toBeFocused()
  await page.unroute('**/api/household/mealplan/preview')
  const acknowledgement = 'I reviewed the unknown dietary data and serving/time uncertainty.'
  await page.getByText(acknowledgement, { exact: true }).click()
  await expect(page.getByRole('checkbox', { name: acknowledgement, exact: true })).toBeChecked()
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: acknowledgement, exact: true })).not.toBeChecked()
  const axe = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(axe.violations).toEqual([])
  await page.screenshot({ path: 'test-results/usability-meal-review.png' })
})

test('failed profile save retains inputs and reload requires explicit discard', async ({
  page,
}) => {
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  const name = page.getByLabel('Name', { exact: true }).first()
  const original = await name.inputValue()
  await name.fill(original + ' retained')
  await page.route('**/api/household/profiles', async (route) => {
    if (route.request().method() === 'PUT') await route.fulfill({ status: 503, body: '{}' })
    else await route.continue()
  })
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('alert')).toBeVisible()
  page.once('dialog', (dialog) => void dialog.dismiss())
  await page.getByRole('button', { name: 'Reload saved version', exact: true }).click()
  await expect(name).toHaveValue(original + ' retained')
  await page.unroute('**/api/household/profiles')
  await name.fill(original)
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('status')).toContainText('Changes saved.')
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  const sorbitol = page.getByRole('checkbox', { name: 'Sorbitol', exact: true }).first()
  const originalSorbitol = await sorbitol.isChecked()
  const notesDisclosure = page.getByText('Notes and excluded tags', { exact: true }).first()
  const notes = page.getByLabel(/^Intolerance notes/).first()
  if (!(await notes.isVisible())) await notesDisclosure.click()
  const originalNotes = await notes.inputValue()
  if (!originalSorbitol) await page.getByText('Sorbitol', { exact: true }).first().click()
  await notes.fill([originalNotes, 'Browser verification note'].filter(Boolean).join('\n'))
  const savedProfile = page.waitForResponse(
    (response) =>
      response.url().endsWith('/api/household/profiles') && response.request().method() === 'PUT',
  )
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  const body: unknown = await (await savedProfile).json()
  expect(body).toMatchObject({
    members: expect.arrayContaining([
      expect.objectContaining({
        name: original,
        intolerances: expect.arrayContaining(['sorbitol', 'Browser verification note']),
      }),
    ]),
  })
  await expect(notes).not.toHaveValue(/sorbitol/)
  await notes.fill(originalNotes)
  if (!originalSorbitol) await page.getByText('Sorbitol', { exact: true }).first().click()
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  for (const width of [1440, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 950 })
    expect(
      await page.locator('.workspace').evaluate((node) => node.scrollWidth <= node.clientWidth + 1),
    ).toBe(true)
    await expect(page.locator('#app-navigation-vue')).toHaveAttribute(
      'aria-hidden',
      String(width < 1024),
    )
  }
  for (const scheme of ['light', 'dark'] satisfies ('light' | 'dark')[]) {
    await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
    for (const width of [1440, 390]) {
      await page.setViewportSize({ width, height: 950 })
      await page.getByRole('heading', { name: 'Household', exact: true }).scrollIntoViewIfNeeded()
      await page.screenshot({
        path: `test-results/usability-profiles-${scheme}-${width}.png`,
        animations: 'disabled',
      })
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
  await page.setViewportSize({ width: 1440, height: 950 })
  await expect(page.locator('#app-navigation-vue')).toHaveAttribute('aria-hidden', 'false')
  await page.getByRole('link', { name: 'Your households', exact: true }).click()
  await page.route('**/api/households', async (route) => {
    if (route.request().method() === 'POST')
      await route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({ code: 'membership_capacity' }),
      })
    else await route.continue()
  })
  await page.getByLabel('Household name', { exact: true }).fill('Capacity verification')
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText(
    'The household or membership limit has been reached.',
  )
  await expect(page.getByLabel('Household name', { exact: true })).toHaveValue(
    'Capacity verification',
  )
  await page.unroute('**/api/households')
})
