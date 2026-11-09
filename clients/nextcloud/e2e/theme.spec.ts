import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
test('native system light/dark themes, reduced motion and keyboard focus', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light', reducedMotion: 'reduce' })
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click({ timeout: 30000 })
  await page.locator('.record-list').getByRole('link').last().click()
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await page.getByLabel('Search ingredients', { exact: true }).fill('oats')
  await page.locator('.record-list').getByRole('link').first().click()
  await expect(page.getByRole('heading', { name: 'Nutrient values', exact: true })).toBeVisible()
  const backgrounds: string[] = []
  for (const scheme of ['light', 'dark'] satisfies ('light' | 'dark')[]) {
    await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
    backgrounds.push(
      await page
        .locator('.mealshop')
        .evaluate((e) => getComputedStyle(e).getPropertyValue('--color-main-background').trim()),
    )
    const axe = await new AxeBuilder({ page })
      .include('.mealshop')
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze()
    expect(axe.violations).toEqual([])
    for (const width of [1440, 390]) {
      await page.setViewportSize({ width, height: 1000 })
      await page.screenshot({ path: `test-results/theme-${scheme}-${width}.png`, fullPage: true })
    }
  }
  expect(backgrounds[0]).not.toEqual(backgrounds[1])
  await page.setViewportSize({ width: 320, height: 1000 })
  const toggle = page.locator('button.app-navigation-toggle')
  const bounds = await toggle.boundingBox()
  expect(bounds?.x ?? -1).toBeGreaterThanOrEqual(0)
  await toggle.click()
  await expect(page.getByRole('link', { name: 'Your households', exact: true })).toBeVisible()
  await toggle.click()
  await page.keyboard.press('Tab')
  await expect(page.locator(':focus')).toBeVisible()
  expect(await page.evaluate(() => matchMedia('(prefers-reduced-motion: reduce)').matches)).toBe(
    true,
  )
})
