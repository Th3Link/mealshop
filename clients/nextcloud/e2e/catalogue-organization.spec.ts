import { openFixture } from './fixtures'
import { test, expect } from '@playwright/test'
test('all reference ingredients have browsable groups and exact subtype search', async ({
  page,
}) => {
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await openFixture(page, 'authoring-experience.spec.ts')
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await expect.poll(() => page.locator('.ingredient-group-row').count()).toBeGreaterThanOrEqual(73)
  const wheat = page
    .locator('.ingredient-group-row')
    .filter({ has: page.getByRole('link', { name: 'Wheat flour', exact: true }) })
  await wheat.getByRole('button', { name: 'Wheat flour: Subtypes', exact: true }).click()
  await expect(
    page.locator('.record-list .subtype').filter({ hasText: 'Wheat flour, Type 405' }),
  ).toBeVisible()
  await expect(wheat).not.toContainText('custom-group-')
  await page.screenshot({ path: 'test-results/organized-catalogue.png', animations: 'disabled' })
  await page.getByLabel('Search ingredients', { exact: true }).fill('haferflocken')
  await expect(page.locator('.record-list')).toContainText('Oat flakes')
  await expect(page.locator('.record-list')).not.toContainText('Oats and oat products')
  await page.getByRole('link').filter({ hasText: 'Oat flakes' }).click()
  await expect(page.getByRole('heading', { name: 'Oat flakes', exact: true })).toBeVisible()
  await page.setViewportSize({ width: 390, height: 844 })
  await expect
    .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
    .toBeLessThanOrEqual(390)
})
