import { test, expect, type Page } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'

const embedded = '/index.php/apps/app_api/embedded/mealshop/main'
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
async function createHome(page: Page) {
  const name = `Browser verification ${Date.now()}`
  await page.getByLabel('Household name', { exact: true }).fill(name)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'sharing.spec.ts')
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  if (!id) throw new Error('Owned sharing fixture was not recorded')
  return { id, name }
}
async function createRecipe(page: Page, title: string, imported = false) {
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill(title)
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('Oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.500000')
  if (imported) {
    const source = page.getByRole('checkbox', { name: 'Recipe from another source', exact: true })
    await source.focus()
    await source.press('Space')
    await expect(source).toBeChecked()
    await page
      .getByLabel('Original recipe URL', { exact: true })
      .fill('https://example.org/original-sharing-recipe')
  }
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible()
  const id = new URL(page.url()).pathname.match(/\/(recipe-\d+)$/)?.[1]
  if (!id) throw new Error('Synthetic recipe identity missing')
  return id
}
async function grant(page: Page, home: string, household = false) {
  await page.getByRole('button', { name: 'Share', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Sharing', exact: true })
  if (household)
    await dialog
      .getByRole('combobox', { name: 'Share with', exact: true })
      .selectOption('household_invitation')
  const picker = dialog.getByRole('combobox', {
    name: household ? 'Choose household owner' : 'Choose person',
    exact: true,
  })
  await expect(picker).toBeEnabled()
  await picker.fill(process.env.MEALSHOP_READER_USER ?? '')
  await page
    .getByRole('option')
    .filter({ hasText: process.env.MEALSHOP_READER_USER ?? '' })
    .click()
  const response = page.waitForResponse(
    (r) => r.request().method() === 'POST' && r.url().includes(`/api/households/${home}/shares`),
  )
  await dialog.getByRole('button', { name: 'Grant read access', exact: true }).click()
  const result: unknown = await (await response).json()
  await expect(dialog.getByText('Read access granted', { exact: true })).toBeVisible()
  for (const width of [1440, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    await expect
      .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
      .toBeLessThanOrEqual(width)
    await page.screenshot({
      path: `test-results/sharing-dialog-${household ? 'household' : 'person'}-${width}.png`,
      animations: 'disabled',
    })
  }
  expect(
    (
      await new AxeBuilder({ page })
        .include('.sharing-dialog')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.setViewportSize({ width: 1440, height: 1000 })
  if (!result || typeof result !== 'object' || !('id' in result) || typeof result.id !== 'string')
    throw new Error('Synthetic grant identity missing')
  await dialog.getByRole('button', { name: 'Close', exact: true }).last().click()
  await expect(dialog).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Share', exact: true })).toBeFocused()
  return result.id
}
async function audit(page: Page, name: string) {
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    for (const scheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
      for (const width of [1440, 390, 320]) {
        await page.setViewportSize({ width, height: 1000 })
        await expect
          .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(width)
        await page.screenshot({
          path: `test-results/${name}-${language}-${scheme}-${width}.png`,
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

test('personal recipe sharing pins publication, scales and clears revoked content', async ({
  page,
  browser,
}) => {
  test.setTimeout(180000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await login(page)
  const home = await createHome(page)
  const recipe = await createRecipe(page, 'Sharing acceptance recipe')
  const shared = await grant(page, home.id)
  const context = await browser.newContext()
  const reader = await context.newPage()
  reader.on('pageerror', (e) => errors.push(e.message))
  await login(reader, true)
  await reader.goto(`${embedded}/shared/${home.id}/${shared}`)
  await expect(
    reader.getByRole('heading', { name: 'Sharing acceptance recipe', exact: true }),
  ).toBeVisible()
  await expect(reader.getByRole('button', { name: 'Edit recipe', exact: true })).toHaveCount(0)
  await reader.getByLabel('Preview servings', { exact: true }).fill('3')
  await reader.getByRole('button', { name: 'Preview', exact: true }).click()
  await expect(reader.locator('.recipe-part-content')).toContainText('150.75')
  await page.getByRole('link', { name: 'Edit recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Revised sharing acceptance recipe')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Revised sharing acceptance recipe', exact: true }),
  ).toBeVisible()
  await reader.getByRole('button', { name: 'Refresh access', exact: true }).click()
  await expect(
    reader.getByRole('heading', { name: 'Sharing acceptance recipe', exact: true }),
  ).toBeVisible()
  await page.getByRole('button', { name: 'Share', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Sharing', exact: true })
  page.once('dialog', (d) => {
    void d.accept()
  })
  await dialog.getByRole('button', { name: 'Share this revision', exact: true }).click()
  await expect(dialog.getByText('Shared revision updated', { exact: true })).toBeVisible()
  await dialog.getByRole('button', { name: 'Close', exact: true }).last().click()
  await reader.getByRole('button', { name: 'Refresh access', exact: true }).click()
  await expect(
    reader.getByRole('heading', { name: 'Revised sharing acceptance recipe', exact: true }),
  ).toBeVisible()
  await audit(reader, 'shared-recipe')
  await page.getByRole('button', { name: 'Share', exact: true }).click()
  page.once('dialog', (d) => {
    void d.accept()
  })
  await page
    .getByRole('dialog')
    .getByRole('button', { name: new RegExp('^Revoke access') })
    .click()
  await expect(page.getByRole('dialog').getByText('Access revoked', { exact: true })).toBeVisible()
  await reader.getByRole('button', { name: 'Refresh access', exact: true }).click()
  await expect(reader.getByText('Share unavailable', { exact: true })).toBeVisible()
  await expect(
    reader.getByRole('heading', { name: 'Revised sharing acceptance recipe', exact: true }),
  ).toHaveCount(0)
  await page.getByRole('dialog').getByRole('button', { name: 'Close', exact: true }).last().click()
  await expect(page).toHaveURL(new RegExp(`/recipes/${recipe}$`))
  expect(errors).toEqual([])
  await context.close()
})

test('collections pin historical reading and household invitations preserve imported source-only access', async ({
  page,
  browser,
}) => {
  test.setTimeout(240000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  await login(page)
  const source = await createHome(page)
  const owned = await createRecipe(page, 'Sharing acceptance recipe')
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await createRecipe(page, 'Imported sharing acceptance recipe', true)
  await page.getByRole('link', { name: 'Collections', exact: true }).click()
  await page.getByRole('link', { name: 'Create collection', exact: true }).click()
  await page.getByLabel('Collection name', { exact: true }).fill('Sharing acceptance collection')
  for (const name of ['Sharing acceptance recipe', 'Imported sharing acceptance recipe']) {
    await page.getByRole('combobox', { name: 'Choose a recipe', exact: true }).fill(name)
    await page.getByRole('option', { name, exact: true }).click()
    await page.getByRole('button', { name: 'Add recipe', exact: true }).click()
  }
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Sharing acceptance collection', exact: true }),
  ).toBeVisible()
  const collectionPath = new URL(page.url()).pathname
  await page.getByRole('link', { name: 'Edit collection', exact: true }).click()
  await page
    .getByRole('textbox', { name: 'Description', exact: true })
    .fill('Synthetic collection notes')
  await page
    .getByRole('button', {
      name: 'Move recipe up · Imported sharing acceptance recipe',
      exact: true,
    })
    .click()
  await page.setViewportSize({ width: 320, height: 1000 })
  await expect
    .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
    .toBeLessThanOrEqual(320)
  await page.screenshot({ path: 'test-results/collection-editor-320.png', animations: 'disabled' })
  expect(
    (
      await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByText('Synthetic collection notes', { exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Back', exact: true }).click()
  const search = page.getByRole('searchbox', { name: 'Search collections', exact: true })
  await search.pressSequentially('Sharing', { delay: 80 })
  await expect(page).toHaveURL(/q=Sharing/)
  await expect(search).toBeFocused()
  await page.getByRole('link').filter({ hasText: 'Sharing acceptance collection' }).click()
  await page.goto(`${embedded}/h/${source.id}/recipes/${owned}?edit=1`)
  await page.getByLabel('Recipe title', { exact: true }).fill('Revised sharing acceptance recipe')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Revised sharing acceptance recipe', exact: true }),
  ).toBeVisible()
  await page.goto(collectionPath)
  await page
    .getByRole('link')
    .filter({ hasText: 'Sharing acceptance recipe' })
    .filter({ hasNotText: 'Imported' })
    .click()
  await expect(
    page.getByRole('heading', { name: 'Sharing acceptance recipe', exact: true }),
  ).toBeVisible()
  await expect(page.getByRole('link', { name: 'Open current revision', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Back', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Sharing acceptance collection', exact: true }),
  ).toBeVisible()
  const shared = await grant(page, source.id, true)
  const context = await browser.newContext()
  const receiver = await context.newPage()
  receiver.on('pageerror', (e) => errors.push(e.message))
  await login(receiver, true)
  const destination = await createHome(receiver)
  await receiver.getByRole('link', { name: 'Shared with me', exact: true }).click()
  await expect(
    receiver.getByRole('heading', { name: 'Household invitation', exact: true }),
  ).toBeVisible()
  await receiver
    .getByRole('combobox', { name: 'Destination household', exact: true })
    .selectOption({ label: destination.name })
  await receiver.getByRole('button', { name: 'Accept for household', exact: true }).click()
  await receiver.getByRole('link').filter({ hasText: 'Sharing acceptance collection' }).click()
  await expect(
    receiver.getByRole('heading', { name: 'Sharing acceptance collection', exact: true }),
  ).toBeVisible()
  await expect(
    receiver.getByText('Imported sharing acceptance recipe', { exact: true }),
  ).toHaveCount(0)
  await audit(receiver, 'shared-collection')
  await receiver.getByRole('link').filter({ hasText: 'Original recipe' }).click()
  await expect(
    receiver.getByRole('heading', { name: 'Original recipe', exact: true }),
  ).toBeVisible()
  await expect(
    receiver.locator('a[href="https://example.org/original-sharing-recipe"]'),
  ).toBeVisible()
  await expect(receiver.getByRole('heading', { name: 'Nutrient values', exact: true })).toHaveCount(
    0,
  )
  await expect(
    receiver.getByText('Imported sharing acceptance recipe', { exact: true }),
  ).toHaveCount(0)
  receiver.once('dialog', (d) => {
    void d.accept()
  })
  await receiver.getByRole('button', { name: 'Remove shared access', exact: true }).click()
  await expect(receiver.getByRole('heading', { name: 'Shared with me', exact: true })).toBeVisible()
  await receiver.goto(`${embedded}/shared/${source.id}/${shared}`)
  await expect(receiver.getByText('Share unavailable', { exact: true })).toBeVisible()
  expect(errors).toEqual([])
  await context.close()
})
