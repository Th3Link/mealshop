import { test, expect, type Page } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'
const embedded = '/index.php/apps/app_api/embedded/mealshop/main'
const fixture =
  process.env.MEALSHOP_IMPORT_FIXTURE_URL ??
  'https://raw.githubusercontent.com/Th3Link/mealshop/main/tests/fixtures/import-recipe.html'
async function login(page: Page, reader = false) {
  await page.goto(`${embedded}/households`)
  await page
    .locator('#user')
    .fill(process.env[reader ? 'MEALSHOP_READER_USER' : 'MEALSHOP_TEST_USER'] ?? '')
  await page
    .locator('#password')
    .fill(process.env[reader ? 'MEALSHOP_READER_PASSWORD' : 'MEALSHOP_TEST_PASSWORD'] ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('#mealshop-language').selectOption('en')
}
async function setup(page: Page) {
  await login(page)
  const name = `Browser verification ${Date.now()}`
  await page.getByLabel('Household name', { exact: true }).fill(name)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'recipe-import.spec.ts')
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  if (!id) throw new Error('Owned import fixture identity missing')
  await page.getByRole('link', { name: 'Import from link', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Import from link', exact: true })).toBeVisible()
  return { id, name }
}
async function check(page: Page, name: string) {
  const box = page.getByRole('checkbox', { name, exact: true })
  await box.focus()
  await page.keyboard.press('Space')
  await expect(box).toBeChecked()
}
const consent = 'Allow this server to contact the public source website and validated redirects.'
const reviewed =
  'I reviewed all ingredient mappings, quantities, servings, language, steps and time estimates, including any zero values.'
async function audit(page: Page, name: string) {
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    for (const colorScheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme, reducedMotion: 'reduce' })
      for (const width of [1440, 390, 320]) {
        await page.setViewportSize({ width, height: 1000 })
        await expect
          .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(width)
        await page.locator('.mealshop h1').scrollIntoViewIfNeeded()
        await page.screenshot({
          path: `test-results/${name}-${language}-${colorScheme}-${width}.png`,
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
  }
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.locator('#mealshop-language').selectOption('en')
}

test('public structured import retains source text, exact review and source-only sharing', async ({
  page,
}) => {
  test.setTimeout(240000)
  const errors: string[] = [],
    unsafeRequests: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('request', (request) => {
    if (request.url().includes('must-not-load')) unsafeRequests.push(request.url())
  })
  await setup(page)
  await page.getByLabel('Recipe URL', { exact: true }).fill(fixture)
  await expect(
    page.getByRole('button', { name: 'Fetch recipe preview', exact: true }),
  ).toBeDisabled()
  await check(page, consent)
  await page.getByRole('button', { name: 'Fetch recipe preview', exact: true }).click()
  await expect(
    page.getByText('Preview only · nothing has been saved.', { exact: true }),
  ).toBeVisible({ timeout: 25000 })
  await page.getByRole('button', { name: 'Review this recipe', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Review imported recipe', exact: true }),
  ).toBeFocused()
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(
    'Import acceptance recipe',
  )
  await expect(page.getByLabel('Servings', { exact: true })).toHaveValue('2')
  await expect(page.locator('.import-source-line')).toHaveText([
    'Source ingredient: 100.500000 g Oats',
    'Source ingredient: 25 g Drinking water',
  ])
  const pickers = page.getByRole('combobox', { name: 'Choose ingredient', exact: true })
  await pickers.nth(0).fill('Oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await pickers.nth(1).fill('Drinking water')
  await page.getByRole('option', { name: 'Drinking water', exact: true }).click()
  await expect(page.getByLabel('Grams', { exact: true }).nth(0)).toHaveValue('100.5')
  await expect(page.getByLabel('1. Instruction', { exact: true })).toHaveValue(
    'Preparation: Mix the synthetic ingredients.',
  )
  await expect(page.locator('.import-evidence')).toContainText('PT3M')
  await expect(
    page.getByRole('button', { name: 'Save reviewed import', exact: true }),
  ).toBeDisabled()
  await audit(page, 'import-review')
  await check(page, reviewed)
  await page.getByRole('button', { name: 'Save reviewed import', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Import acceptance recipe', exact: true }),
  ).toBeVisible()
  await expect(page.locator('.import-evidence')).toContainText('Synthetic fixture author')
  await expect(page.locator('.mealshop')).not.toContainText('99999')
  await expect(page.locator('.mealshop')).not.toContainText('GlutenFreeDiet')
  await page.getByRole('link', { name: 'Edit recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Revised import acceptance recipe')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Revised import acceptance recipe', exact: true }),
  ).toBeVisible()
  await expect(page.locator('.import-evidence')).toContainText(
    'Source title Import acceptance recipe',
  )
  await page.getByRole('button', { name: 'Share', exact: true }).click()
  await expect(page.getByRole('dialog', { name: 'Sharing', exact: true })).toContainText('source')
  expect(errors).toEqual([])
  expect(unsafeRequests).toEqual([])
})

test('blocked URL falls back to manual review and a lost save response retries without duplication', async ({
  page,
}) => {
  test.setTimeout(180000)
  const household = await setup(page)
  await page.getByLabel('Recipe URL', { exact: true }).fill('https://127.0.0.1/private')
  await check(page, consent)
  await page.getByRole('button', { name: 'Fetch recipe preview', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('public HTTPS')
  await page.getByLabel('Recipe URL', { exact: true }).fill('https://example.org/import-original')
  await page.getByRole('button', { name: 'Enter privately without fetching', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Import acceptance recipe')
  await expect(page.getByLabel('Servings', { exact: true })).toHaveValue('0')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('Oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.500000')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Prepare synthetic manual import')
  await expect(page.locator('.import-evidence')).toContainText('Not supplied by the source')
  await check(page, reviewed)
  let first: { id: string } | undefined
  await page.route('**/api/household/imports', async (route) => {
    if (first) return route.continue()
    const response = await route.fetch()
    expect(response.status()).toBe(201)
    const body: unknown = await response.json()
    if (!body || typeof body !== 'object' || !('id' in body) || typeof body.id !== 'string')
      throw new Error('Synthetic import identity missing')
    first = { id: body.id }
    await route.abort('failed')
  })
  await page.getByRole('button', { name: 'Save reviewed import', exact: true }).click()
  await expect(page.getByText(/^This confirmed request is frozen/)).toBeVisible()
  await expect(page.getByLabel('Recipe title', { exact: true })).toBeDisabled()
  await page.getByRole('button', { name: 'Retry confirmed import', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Import acceptance recipe', exact: true }),
  ).toBeVisible()
  await expect(page).toHaveURL(new RegExp(`/recipes/${first?.id}$`))
  await page.unroute('**/api/household/imports')
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await expect(page.locator('.record-list a')).toHaveCount(1)
  await page.goto(`${embedded}/h/${household.id}/recipes/import`)
  await page.getByLabel('Recipe URL', { exact: true }).fill('https://example.org/unsaved')
  page.once('dialog', (dialog) => void dialog.dismiss())
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await expect(page).toHaveURL(/\/recipes\/import$/)
  page.once('dialog', (dialog) => void dialog.accept())
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
})

test('reader account sees no import action and cannot submit a source request', async ({
  page,
  browser,
}) => {
  const household = await setup(page)
  page.once('dialog', (dialog) => void dialog.accept())
  await page.getByRole('link', { name: 'Household', exact: true }).click()
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
  const context = await browser.newContext()
  try {
    const reader = await context.newPage()
    await login(reader, true)
    await reader
      .locator('.inline-row')
      .filter({ hasText: household.name })
      .getByRole('button', { name: 'Accept invitation', exact: true })
      .click()
    const joined = reader.locator(`.record-list a[href$="/h/${household.id}/recipes"]`)
    await expect(joined).toBeVisible()
    await joined.click()
    await expect(reader.getByRole('link', { name: 'Import from link', exact: true })).toHaveCount(0)
    await reader.goto(`${embedded}/h/${household.id}/recipes/import`)
    await expect(
      reader.getByRole('button', { name: 'Enter privately without fetching', exact: true }),
    ).toBeDisabled()
    const response = await reader.request.post(
      `/apps/app_api/proxy/mealshop/api/households/${household.id}/domain/api/household/imports/preview`,
      {
        headers: { 'X-Mealshop-Request': '1' },
        data: { mode: 'manual', url: 'https://example.org/import-original' },
      },
    )
    expect(response.status()).toBe(403)
  } finally {
    await context.close()
  }
})
