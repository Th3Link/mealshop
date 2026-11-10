import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'

test.beforeEach(async ({ page }) => {
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
  await recordFixture(page, 'shopping.spec.ts')
})

test('shopping quantities, purchased provenance, recipe rescaling and selected plan sync work together', async ({
  page,
}) => {
  test.setTimeout(180000)
  page.on('dialog', (dialog) => void dialog.accept())
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Shopping acceptance recipe')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.000001')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Prepare test recipe.')
  await page.getByLabel('Active time (seconds)', { exact: true }).fill('60')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Shopping acceptance recipe', exact: true }),
  ).toBeVisible()
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await expect(
    page.getByRole('note', { name: 'Your shopping list is empty', exact: true }),
  ).toBeVisible()
  await page.getByRole('button', { name: 'Add recipe quantities', exact: true }).click()
  const recipeForm = page
    .locator('form')
    .filter({ has: page.getByRole('heading', { name: 'Add recipe quantities', exact: true }) })
  await recipeForm.getByLabel('Servings', { exact: true }).fill('4')
  await recipeForm.getByRole('button', { name: 'Save changes', exact: true }).click()
  const oats = page
    .locator('.shopping-items > li')
    .filter({ has: page.getByRole('checkbox', { name: /Oats/ }) })
  await expect(oats).toContainText('200.000002 g')
  await oats.getByText('Oats · 200.000002 g', { exact: true }).click()
  await page
    .locator('summary')
    .filter({ hasText: /^Purchased/ })
    .click()
  await expect(oats).toContainText('200.000002 g')
  const manual = page.locator('form').filter({ has: page.getByLabel('Name', { exact: true }) })
  await manual.getByLabel('Name', { exact: true }).fill('Test kitchen roll')
  await page.getByLabel('Language', { exact: true }).selectOption('de')
  await manual.getByLabel('Menge', { exact: true }).fill('1.5')
  await expect(
    manual.getByRole('button', { name: 'Artikel hinzufügen', exact: true }),
  ).toBeDisabled()
  await page.getByLabel('Sprache', { exact: true }).selectOption('en')
  await expect(manual.getByRole('button', { name: 'Add item', exact: true })).toBeDisabled()
  await manual.getByLabel('Quantity', { exact: true }).fill('2')
  await manual.getByRole('button', { name: 'Add item', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Test kitchen roll/ })).toBeVisible()
  const sources = page
    .locator('section')
    .filter({ has: page.getByRole('heading', { name: 'List sources', exact: true }) })
  const recipeSource = sources.locator('li').filter({ hasText: 'Shopping acceptance recipe' })
  await recipeSource.getByRole('button', { name: 'Edit recipe', exact: true }).click()
  await recipeForm.getByLabel('Servings', { exact: true }).fill('6')
  await recipeForm.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(oats).toContainText('100.000001 g')
  await expect(oats).toContainText('Purchased: 200.000002 g')
  await oats.getByText('Recipes and contributions', { exact: true }).click()
  await expect(oats).toContainText('Shopping acceptance recipe')
  await recipeSource.getByRole('button', { name: 'Remove', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Test kitchen roll/ })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('checkbox', { name: /Test kitchen roll/ })).toBeVisible()
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).fill('Test attendee')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Plan a meal', exact: true }).click()
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  await page
    .getByText('I reviewed the unknown dietary data and serving/time uncertainty.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Apply reviewed plan', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Plan a meal', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'From meal plan', exact: true }).click()
  await page.getByRole('button', { name: 'Update plan contributions', exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Oats/ })).toHaveCount(1)
  await page.getByRole('button', { name: 'Shopping mode', exact: true }).click()
  await expect(page.getByLabel('Name', { exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Manage list', exact: true })).toBeVisible()
  for (const width of [1440, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    expect(
      await page.locator('.workspace').evaluate((node) => node.scrollWidth <= node.clientWidth + 1),
    ).toBe(true)
  }
  expect(
    (
      await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.screenshot({ path: 'test-results/shopping-320.png', animations: 'disabled' })
  expect(errors).toEqual([])
})

test('shop preferences, fallback, manual ordering and consented history work in the embedded app', async ({
  page,
}) => {
  page.on('dialog', (dialog) => void dialog.accept())
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  const settings = page.locator('.shopping-settings')
  await settings.getByText('Shops and shopping order', { exact: true }).click()
  for (const shop of ['Test market', 'Test backup']) {
    await settings.getByLabel('Shop name', { exact: true }).fill(shop)
    await settings.getByRole('button', { name: 'Add shop', exact: true }).click()
    await expect(settings.locator('strong').filter({ hasText: shop })).toBeVisible()
  }
  await settings.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await settings.getByLabel('First choice', { exact: true }).selectOption({ label: 'Test market' })
  await settings.getByLabel('Second choice', { exact: true }).selectOption({ label: 'Test backup' })
  await settings.getByRole('button', { name: 'Save changes', exact: true }).click()
  const manual = page.locator('form').filter({ has: page.getByLabel('Name', { exact: true }) })
  await manual.getByLabel('Name', { exact: true }).fill('Test oats')
  await manual.getByText('Link to an ingredient (optional)', { exact: true }).click()
  await manual.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await manual.getByRole('button', { name: 'Add item', exact: true }).click()
  await manual.getByLabel('Name', { exact: true }).fill('Test kitchen cloth')
  await manual.getByRole('button', { name: 'Add item', exact: true }).click()
  const oats = page
    .locator('.shopping-items > li')
    .filter({ has: page.getByRole('checkbox', { name: /Oats/ }) })
  await oats.locator('summary').filter({ hasText: 'Test market' }).click()
  await oats
    .getByRole('button', { name: 'Move to second choice: Test backup', exact: true })
    .click()
  await expect(oats.getByLabel('Shop for this item', { exact: true })).toHaveValue(/shop-/)
  await oats
    .getByLabel('Shop for this item', { exact: true })
    .selectOption({ label: 'Test market' })
  const cloth = page
    .locator('.shopping-items > li')
    .filter({ has: page.getByRole('checkbox', { name: /Test kitchen cloth/ }) })
  await cloth.locator('summary').filter({ hasText: 'Unassigned' }).click()
  await cloth
    .getByLabel('Shop for this item', { exact: true })
    .selectOption({ label: 'Test market' })
  await page.getByLabel('Shopping at', { exact: true }).selectOption({ label: 'Test market' })
  await cloth.getByRole('button', { name: 'Move up', exact: true }).click()
  await expect(page.locator('.shopping-items > li').first()).toContainText('Test kitchen cloth')
  await page.getByRole('button', { name: 'Use learned/default order', exact: true }).click()
  await settings
    .getByText('Learn check-off order locally in this household', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Start shopping session', exact: true }).click()
  await expect(oats.getByRole('checkbox')).toBeEnabled()
  await oats.locator('.checkbox-radio-switch__text').click()
  await expect(cloth.getByRole('checkbox')).toBeEnabled()
  await cloth.locator('.checkbox-radio-switch__text').click()
  await expect(cloth).toHaveCount(0)
  await page.getByRole('button', { name: 'Complete session', exact: true }).click()
  await page
    .locator('summary')
    .filter({ hasText: /^Purchased/ })
    .click()
  await expect(oats.getByRole('checkbox')).toBeEnabled()
  await oats.locator('.checkbox-radio-switch__text').click()
  await settings.getByText(/Session history · 1/).click()
  await expect(settings).toContainText('Corrected session')
  await settings.getByRole('button', { name: 'Reset learning history', exact: true }).click()
  await expect(settings).toContainText('No sessions recorded.')
  await expect(page.getByRole('checkbox', { name: /Oats/ })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('checkbox', { name: /Oats/ })).toBeVisible()
})

test('barcode entry completes manually after denied camera and failed lookup, retaining exact package provenance', async ({
  page,
}) => {
  page.on('dialog', (dialog) => void dialog.accept())
  await page.addInitScript(() => {
    class Detector {
      static async getSupportedFormats() {
        return ['ean_13']
      }
      async detect() {
        return []
      }
    }
    Object.defineProperty(window, 'BarcodeDetector', { value: Detector, configurable: true })
    Object.defineProperty(navigator.mediaDevices, 'getUserMedia', {
      value: async () => {
        throw new DOMException('Denied', 'NotAllowedError')
      },
      configurable: true,
    })
  })
  await page.reload()
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'Add packaged product', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Add packaged product', exact: true })
  await dialog.getByRole('button', { name: 'Scan with camera', exact: true }).click()
  await expect(dialog).toContainText('Camera scanning is unavailable')
  await dialog.getByLabel('Barcode (EAN / UPC / GTIN)', { exact: true }).fill('034000470694')
  await expect(
    dialog.getByRole('button', { name: 'Find in household', exact: true }),
  ).toBeDisabled()
  await dialog.getByLabel('Barcode (EAN / UPC / GTIN)', { exact: true }).fill('034000470693')
  await dialog.getByRole('button', { name: 'Find in household', exact: true }).click()
  await expect(dialog).toContainText('No saved product found')
  await dialog.getByText('Optional Open Food Facts lookup', { exact: true }).click()
  await expect(
    dialog.getByRole('button', { name: 'Look up this barcode', exact: true }),
  ).toBeDisabled()
  await dialog
    .getByText('Send this barcode to Open Food Facts for this request', { exact: true })
    .click()
  await page.route('**/shopping/lookup', (route) =>
    route.fulfill({
      status: 422,
      contentType: 'application/json',
      body: JSON.stringify({ code: 'shopping_lookup_unavailable' }),
    }),
  )
  await dialog.getByRole('button', { name: 'Look up this barcode', exact: true }).click()
  await expect(dialog.getByRole('alert')).toContainText('Enter the product manually')
  await dialog.getByLabel('Product name', { exact: true }).fill('Test oats package')
  await dialog.getByLabel('Original package label (optional)', { exact: true }).fill('0.100001 kg')
  await dialog.getByLabel('Quantity', { exact: true }).fill('0.100001')
  await dialog.getByLabel('Unit', { exact: true }).selectOption('kg')
  await dialog.getByLabel('Number of packages', { exact: true }).fill('3')
  await dialog.getByText('Link to an ingredient (optional)', { exact: true }).click()
  await dialog.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await expect(
    dialog.getByRole('button', { name: 'Save and add packages', exact: true }),
  ).toBeDisabled()
  await dialog
    .getByText('I confirm this ingredient identity for shopping aggregation', { exact: true })
    .click()
  for (const width of [390, 320, 1440]) {
    await page.setViewportSize({ width, height: 1000 })
    expect(await dialog.evaluate((node) => node.scrollWidth <= node.clientWidth + 1)).toBe(true)
  }
  expect(
    (
      await new AxeBuilder({ page })
        .include('[role="dialog"]')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.screenshot({
    path: 'test-results/shopping-product-dialog.png',
    animations: 'disabled',
  })
  await dialog.getByRole('button', { name: 'Save and add packages', exact: true }).click()
  await expect(dialog).toHaveCount(0)
  const oats = page
    .locator('.shopping-items > li')
    .filter({ has: page.getByRole('checkbox', { name: /Oats/ }) })
  await expect(oats).toContainText('300.003 g')
  await oats.getByText('Recipes and contributions', { exact: true }).click()
  await expect(oats).toContainText('034000470693')
  await expect(oats).toContainText('3 packages × 0.100001 kg')
  await page.reload()
  await expect(page.getByRole('checkbox', { name: /Oats/ })).toBeVisible()
})

test('supported scan adapter resolves a barcode and stops its local media track', async ({
  page,
}) => {
  await page.addInitScript(() => {
    const counter = { stopped: 0 }
    Object.defineProperty(window, 'mealshopCameraTest', { value: counter })
    class Detector {
      static async getSupportedFormats() {
        return ['ean_8']
      }
      async detect() {
        return [{ rawValue: '12345670' }]
      }
    }
    Object.defineProperty(window, 'BarcodeDetector', { value: Detector, configurable: true })
    Object.defineProperty(navigator.mediaDevices, 'getUserMedia', {
      value: async () => {
        const canvas = document.createElement('canvas')
        canvas.width = 16
        canvas.height = 16
        const stream = canvas.captureStream(1)
        canvas.getContext('2d')?.fillRect(0, 0, 16, 16)
        for (const track of stream.getTracks()) {
          const stop = track.stop.bind(track)
          track.stop = () => {
            counter.stopped++
            stop()
          }
        }
        return stream
      },
      configurable: true,
    })
  })
  await page.reload()
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  await page.getByRole('button', { name: 'Add packaged product', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Add packaged product', exact: true })
  await dialog.getByRole('button', { name: 'Scan with camera', exact: true }).click()
  await expect(dialog.getByLabel('Barcode (EAN / UPC / GTIN)', { exact: true })).toHaveValue(
    '12345670',
  )
  await expect(dialog.getByLabel('Product name', { exact: true })).toBeVisible()
  expect(
    await page.evaluate(
      () =>
        (window as Window & { mealshopCameraTest?: { stopped: number } }).mealshopCameraTest
          ?.stopped,
    ),
  ).toBe(1)
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click()
  await expect(dialog).toHaveCount(0)
})
