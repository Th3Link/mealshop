import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'
test('visual shopping keeps search, exact entry, check-off undo and failed writes usable', async ({
  page,
}) => {
  test.setTimeout(120000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  page.on('dialog', (d) => void d.accept())
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'visual-shopping.spec.ts')
  await page.getByRole('link', { name: 'Shopping list', exact: true }).click()
  const search = page.getByLabel('Find an item', { exact: true })
  for (const name of [
    'Test kitchen roll',
    'Test kitchen cloth',
    'Test oats',
    'Test apples',
    'Test milk',
  ]) {
    await search.fill(name)
    await search.press('Enter')
    await expect(page.getByLabel('Name', { exact: true })).toHaveValue(name)
    if (name.endsWith('roll')) {
      await page.getByLabel('Quantity', { exact: true }).fill('0.100001')
      await page.getByRole('combobox', { name: 'Unit', exact: true }).selectOption('kg')
    }
    const food =
      name === 'Test oats'
        ? 'Oats'
        : name === 'Test apples'
          ? 'Apple raw'
          : name === 'Test milk'
            ? 'Milk skimmed, fresh, max. 0.1 % fat, pasteurised'
            : null
    if (food) {
      await page.getByText('Link to an ingredient (optional)', { exact: true }).click()
      await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill(food)
      await page.getByRole('option', { name: food, exact: true }).click()
    }
    await page
      .locator('form')
      .filter({ has: page.getByLabel('Name', { exact: true }) })
      .getByRole('button', { name: 'Add item', exact: true })
      .click()
    await expect(search).toBeFocused()
    await expect(search).toHaveValue('')
  }
  const roll = page.locator('.shopping-items>li').filter({
    has: page.getByRole('checkbox', { name: 'Test kitchen roll · 100.001 g', exact: true }),
  })
  await expect(roll).toBeVisible()
  await search.pressSequentially('roll')
  await expect(search).toBeFocused()
  await expect(page.locator('.shopping-items>li')).toHaveCount(1)
  await page.getByRole('button', { name: 'Reset filters', exact: true }).click()
  await roll.getByRole('checkbox').focus()
  await page.keyboard.press('Space')
  await expect(page.getByRole('button', { name: 'Undo', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Undo', exact: true }).click()
  await expect(roll.getByRole('checkbox')).toBeFocused()
  await expect(roll.getByRole('checkbox')).not.toBeChecked()
  await roll.getByRole('button', { name: 'Item details: Test kitchen roll', exact: true }).click()
  await expect(page.getByRole('dialog')).toContainText('0.100001')
  await page.keyboard.press('Escape')
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await page.route('**/api/household/shopping', (route) =>
    route.request().method() === 'POST'
      ? route.fulfill({
          status: 503,
          contentType: 'application/json',
          body: '{"code":"shopping_storage_failed"}',
        })
      : route.continue(),
  )
  await roll.locator('.shopping-tile__body').click()
  await expect(page.getByRole('alert')).toBeVisible()
  await expect(roll.getByRole('checkbox')).not.toBeChecked()
  await page.unroute('**/api/household/shopping')
  await page.getByRole('button', { name: 'Try again', exact: true }).click()
  await page
    .locator('.shopping-items>li')
    .filter({ has: page.getByRole('checkbox', { name: /Test kitchen cloth/ }) })
    .locator('.shopping-tile__body')
    .click()
  await page.reload()
  await expect(page.getByRole('button', { name: 'Undo', exact: true })).toHaveCount(0)
  await page.locator('#mealshop-purchased>summary').click()
  for (const scheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
    for (const width of [1440, 768, 390, 320]) {
      await page.setViewportSize({ width, height: 1000 })
      await expect(page.locator('#app-navigation-vue')).toHaveAttribute(
        'aria-hidden',
        String(width < 1024),
      )
      await page.locator('.mealshop h1').scrollIntoViewIfNeeded()
      await page.screenshot({
        path: `test-results/visual-shopping-${scheme}-${width}.png`,
        animations: 'disabled',
      })
      expect(await page.locator('.workspace').evaluate((e) => e.scrollWidth <= e.clientWidth)).toBe(
        true,
      )
    }
    const axe = await new AxeBuilder({ page })
      .include('.mealshop')
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze()
    expect(axe.violations).toEqual([])
  }
  expect(errors).toEqual([])
})
