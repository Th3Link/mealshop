import { test, expect, type Page } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'
const embedded = '/index.php/apps/app_api/embedded/mealshop/main'
async function setup(page: Page) {
  await page.goto(`${embedded}/households`)
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('#mealshop-language').selectOption('en')
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'inventory.spec.ts')
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  if (!id) throw new Error('Owned inventory fixture identity missing')
  page.on('dialog', (d) => void d.accept())
  return `${embedded}/h/${id}`
}
async function review(page: Page) {
  await page.getByRole('button', { name: 'Review changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Review stock changes', exact: true }),
  ).toBeFocused()
}
async function confirm(page: Page) {
  await review(page)
  await page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }).click()
  await expect(page.getByText('Stock changes recorded.', { exact: false })).toBeVisible()
}
async function stock(page: Page, amount: string) {
  await page.getByRole('button', { name: 'Add stock', exact: true }).click()
  await page.getByRole('combobox', { name: 'Ingredient', exact: true }).fill('Oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Quantity', { exact: true }).fill(amount)
  await page.getByLabel('Location', { exact: true }).fill('Test pantry')
  await confirm(page)
}
async function recipe(page: Page) {
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Inventory acceptance recipe')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('Oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.000001')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page
    .getByLabel('1. Instruction', { exact: true })
    .fill('Prepare synthetic inventory recipe')
  await page.getByLabel('Active work', { exact: true }).fill('1')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Inventory acceptance recipe', exact: true }),
  ).toBeVisible()
}
async function audit(page: Page, name: string) {
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    for (const colorScheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme, reducedMotion: 'reduce' })
      for (const width of [1440, 390, 320]) {
        await page.setViewportSize({ width, height: 1000 })
        await page.evaluate(async () => {
          await document.fonts.ready
          await new Promise<void>((resolve) =>
            requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
          )
          await Promise.all(
            document
              .getAnimations()
              .filter((a) => a.effect?.getTiming().iterations !== Infinity)
              .map((a) => a.finished.catch(() => {})),
          )
        })
        expect(
          await page.locator('.mealshop').evaluate((e) => e.scrollWidth <= e.clientWidth),
        ).toBe(true)
        expect(
          (
            await new AxeBuilder({ page })
              .include('.mealshop')
              .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
              .analyze()
          ).violations,
        ).toEqual([])
        await page
          .locator(
            name.includes('review') || name.includes('reservation')
              ? '.stock-review'
              : '.stock-page h1',
          )
          .evaluate((e) => e.scrollIntoView({ block: 'start' }))
        await page.screenshot({
          path: `test-results/${name}-${language}-${colorScheme}-${width}.png`,
          animations: 'disabled',
        })
      }
    }
  }
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.locator('#mealshop-language').selectOption('en')
}

test('reviewed stock, shopping reservation, completed cooking and reversal preserve exact coverage', async ({
  page,
}) => {
  test.setTimeout(240000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  const base = await setup(page)
  await recipe(page)
  const recipeUrl = page.url()
  await page.getByRole('link', { name: 'Inventory', exact: true }).click()
  await stock(page, '100.000001')
  await expect(page.locator('.stock-lots')).toContainText('100.000001')
  await audit(page, 'inventory-stock')
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'Manage list', exact: true }).click()
  await page.getByRole('button', { name: 'Add recipe quantities', exact: true }).click()
  const form = page
    .locator('form')
    .filter({ has: page.getByRole('heading', { name: 'Add recipe quantities', exact: true }) })
  await form.getByRole('button', { name: 'Save changes', exact: true }).click()
  await page.getByRole('link', { name: 'Use stock', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Reserve stock for shopping', exact: true }),
  ).toBeVisible()
  const source = page.getByRole('checkbox', { name: /Inventory acceptance recipe/ })
  await source.focus()
  await page.keyboard.press('Space')
  await review(page)
  await audit(page, 'inventory-reservation')
  await page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }).click()
  await expect(page.getByText('Stock changes recorded.', { exact: false })).toBeVisible()
  await page.goto(`${base}/shopping`)
  await expect(page.getByText('Covered by stock', { exact: true })).toBeVisible()
  await expect(page.getByRole('checkbox', { name: /Oats/ })).toHaveCount(0)
  await page.goto(recipeUrl)
  await page.getByRole('link', { name: 'Start cooking or baking', exact: true }).click()
  await page.getByRole('button', { name: 'Review schedule', exact: true }).click()
  await page
    .getByText('I reviewed the times, dependencies and uncertainties.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save this schedule', exact: true }).click()
  await page.getByRole('button', { name: /Start step:/ }).click()
  await page.getByRole('button', { name: 'Mark step complete', exact: true }).click()
  await page.getByRole('link', { name: 'Record cooking consumption', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Inventory acceptance recipe/ })).toBeChecked()
  await review(page)
  await expect(page.locator('.stock-review')).toContainText('100.000001')
  await page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }).click()
  await expect(page.getByText('Stock changes recorded.', { exact: false })).toBeVisible()
  await page.getByRole('link', { name: 'View booking history', exact: true }).click()
  const consumption = page
    .locator('.stock-history > li')
    .filter({ has: page.getByRole('heading', { name: 'Cooking consumption', exact: true }) })
  await expect(consumption).toContainText('− 100.000001 g')
  await consumption.getByRole('button', { name: 'Reverse booking', exact: true }).click()
  await page.getByLabel('Reason', { exact: true }).fill('Synthetic reversal check')
  await confirm(page)
  await expect(page.locator('.stock-history')).toContainText('Synthetic reversal check')
  await audit(page, 'inventory-history')
  await page.goto(`${base}/inventory`)
  await expect(page.locator('.stock-lots')).toContainText('100.000001')
  expect(errors).toEqual([])
})

test('lost confirmed response freezes values and retries once, then ignores late response after navigation', async ({
  page,
}) => {
  test.setTimeout(120000)
  const base = await setup(page)
  await page.getByRole('link', { name: 'Inventory', exact: true }).click()
  await page.getByRole('button', { name: 'Add stock', exact: true }).click()
  await page.getByLabel('Match to', { exact: true }).selectOption('unmatched')
  await page.getByLabel('Item name', { exact: true }).fill('Synthetic unlinked stock')
  await page.getByLabel('Quantity', { exact: true }).fill('0.000001')
  await review(page)
  let first = true,
    release = () => {}
  const held = new Promise<void>((resolve) => {
    release = resolve
  })
  await page.route('**/api/household/inventory', async (route) => {
    if (route.request().method() !== 'POST') return route.continue()
    const response = await route.fetch()
    expect(response.ok()).toBe(true)
    if (first) {
      first = false
      return route.abort('failed')
    }
    await held
    return route.fulfill({ response })
  })
  await page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }).click()
  await expect(page.getByText(/^The confirmed values are locked/)).toBeVisible()
  await expect(page.getByLabel('Item name', { exact: true })).toBeDisabled()
  await page.getByRole('button', { name: 'Retry confirmed booking', exact: true }).click()
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  const response = page.waitForResponse(
    (r) => r.url().endsWith('/api/household/inventory') && r.request().method() === 'POST',
  )
  release()
  await (await response).finished()
  await page.unroute('**/api/household/inventory')
  await expect(page).toHaveURL(/\/recipes$/)
  await page.goto(`${base}/inventory/history`)
  await expect(page.locator('.stock-history > li')).toHaveCount(1)
  await expect(page.locator('.stock-history')).toContainText('+ 0.000001 g')
})

test('manual offer keeps localized exact price and package provenance through purchase and reverse', async ({
  page,
}) => {
  test.setTimeout(180000)
  const base = await setup(page)
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'Manage list', exact: true }).click()
  const settings = page.locator('.shopping-settings')
  await settings.getByText('Shops and shopping order', { exact: true }).click()
  await settings.getByLabel('Shop name', { exact: true }).fill('Test market')
  await settings.getByRole('button', { name: 'Add shop', exact: true }).click()
  await expect(settings.locator('strong').filter({ hasText: 'Test market' })).toBeVisible()
  await page.goto(`${base}/inventory/offers`)
  await page.getByRole('button', { name: 'Record offer', exact: true }).click()
  await page.getByRole('combobox', { name: 'Ingredient', exact: true }).fill('Oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Package quantity', { exact: true }).fill('500.000001')
  await page.locator('#mealshop-language').selectOption('de')
  const price = page.getByLabel('Preis je Packung', { exact: true })
  await price.pressSequentially('1,29')
  await expect(price).toHaveValue('1,29')
  await page.locator('#mealshop-language').selectOption('en')
  await expect(page.getByLabel('Price per package', { exact: true })).toHaveValue('1.29')
  await page.getByLabel('Source', { exact: true }).fill('Synthetic shop leaflet')
  await page
    .getByLabel('Source URL (optional, HTTPS)', { exact: true })
    .fill('https://example.org/synthetic-offer')
  await review(page)
  await audit(page, 'inventory-offer-review')
  await page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }).click()
  await page.getByRole('button', { name: 'Add offer to shopping', exact: true }).click()
  await page.getByLabel('Packages', { exact: true }).fill('2')
  await review(page)
  await expect(
    page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }),
  ).toBeDisabled()
  const consent = page.getByRole('checkbox', { name: /^I reviewed the quantities/ })
  await consent.focus()
  await page.keyboard.press('Space')
  await page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }).click()
  await expect(page.getByText('Stock changes recorded.', { exact: false })).toBeVisible()
  await page.goto(`${base}/shopping`)
  const oats = page.getByRole('checkbox', { name: /Oats/ })
  await expect(oats).toHaveCount(1)
  await expect(page.locator('.shopping-items')).toContainText('1000.000002 g')
  await oats.focus()
  await page.keyboard.press('Space')
  await expect(oats).toBeChecked()
  await page.goto(`${base}/inventory?task=receive`)
  const receipt = page.getByRole('checkbox', { name: /Oats.*1000.000002|Oats.*1000.000002/ })
  await receipt.focus()
  await page.keyboard.press('Space')
  await confirm(page)
  await expect(page.locator('.stock-lots')).toContainText('1000.000002')
  await page.getByRole('link', { name: 'View booking history', exact: true }).click()
  await expect(page.locator('.stock-history')).toContainText('Synthetic shop leaflet')
  const purchase = page
    .locator('.stock-history > li')
    .filter({ has: page.getByRole('heading', { name: 'Purchase receipt', exact: true }) })
  await purchase.getByRole('button', { name: 'Reverse booking', exact: true }).click()
  await page.getByLabel('Reason', { exact: true }).fill('Synthetic return')
  await confirm(page)
  await page.goto(`${base}/inventory`)
  await expect(page.locator('.stock-lots')).toContainText('0 g')
})

test('current reader can inspect native stock and offers but cannot preview or confirm changes', async ({
  page,
  browser,
}) => {
  const base = await setup(page)
  const id = base.split('/').at(-1)
  await page.getByRole('link', { name: 'Inventory', exact: true }).click()
  await stock(page, '7.000001')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  const name = await page.locator('.workspace > .context').innerText()
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
    await reader.goto(`${embedded}/households`)
    await reader.locator('#user').fill(process.env.MEALSHOP_READER_USER ?? '')
    await reader.locator('#password').fill(process.env.MEALSHOP_READER_PASSWORD ?? '')
    await reader.getByRole('button', { name: 'Log in', exact: true }).click()
    await reader.locator('#mealshop-language').selectOption('en')
    await reader
      .locator('.inline-row')
      .filter({ hasText: name.split(' · ')[0] })
      .getByRole('button', { name: 'Accept invitation', exact: true })
      .click()
    await expect(reader.locator(`.record-list a[href$="/h/${id}/recipes"]`)).toBeVisible()
    await reader.goto(`${base}/inventory`)
    await expect(reader.locator('.stock-lots')).toContainText('7.000001')
    await expect(reader.getByRole('button', { name: 'Add stock', exact: true })).toHaveCount(0)
    await expect(reader.getByRole('button', { name: /^Correct quantity:/ })).toHaveCount(0)
    await reader.goto(`${base}/inventory/offers`)
    await expect(reader.getByRole('button', { name: 'Record offer', exact: true })).toHaveCount(0)
    for (const suffix of ['', '/preview']) {
      const response = await reader.request.post(
        `/apps/app_api/proxy/mealshop/api/households/${id}/domain/api/household/inventory${suffix}`,
        { headers: { 'X-Mealshop-Request': '1' }, data: {} },
      )
      expect(response.status()).toBe(403)
    }
  } finally {
    await context.close()
  }
})

test('real concurrent stock change retains draft on conflict and supports reviewed zero correction and archive', async ({
  page,
  context,
}) => {
  const base = await setup(page)
  await page.getByRole('link', { name: 'Inventory', exact: true }).click()
  await page.getByRole('button', { name: 'Add stock', exact: true }).click()
  await page.getByRole('combobox', { name: 'Ingredient', exact: true }).fill('Oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Quantity', { exact: true }).fill('3.000001')
  await review(page)
  const other = await context.newPage()
  try {
    await other.goto(`${base}/inventory`)
    await stock(other, '2')
  } finally {
    await other.close()
  }
  await page.getByRole('button', { name: 'Confirm reviewed changes', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('changed')
  await expect(page.getByLabel('Quantity', { exact: true })).toHaveValue('3.000001')
  await page.getByRole('button', { name: 'Load latest state; keep entries', exact: true }).click()
  await confirm(page)
  await expect(page.locator('.stock-lots > li')).toHaveCount(2)
  await page.getByRole('button', { name: 'Correct quantity: Oats', exact: true }).first().click()
  await page.getByLabel('Observed remaining quantity', { exact: true }).fill('0')
  await page.getByLabel('Reason', { exact: true }).fill('Synthetic count')
  await confirm(page)
  await page.getByRole('button', { name: 'Archive empty lot: Oats', exact: true }).click()
  await confirm(page)
  await expect(page.locator('.stock-lots > li')).toHaveCount(1)
  await page.getByRole('checkbox', { name: 'Show archived lots', exact: true }).focus()
  await page.keyboard.press('Space')
  await expect(page.locator('.stock-lots > li')).toHaveCount(2)
})
