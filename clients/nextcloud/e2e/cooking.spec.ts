import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'

test('native cooking timers, offline recovery and reviewed multiday schedules', async ({
  page,
  context,
}) => {
  test.setTimeout(180000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  page.on('dialog', (d) => void d.accept())
  await context.grantPermissions(['notifications'])
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'cooking.spec.ts')
  await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
  await page.getByLabel('Recipe title', { exact: true }).fill('Cooking acceptance recipe')
  await page.getByLabel('Servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await page.getByLabel('Grams', { exact: true }).fill('100.000001')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Prepare test dough')
  await page.getByLabel('Active time (seconds)', { exact: true }).fill('1')
  await page.getByLabel('Waiting time (seconds)', { exact: true }).fill('10')
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('2. Instruction', { exact: true }).fill('Bake test dough')
  await page.getByLabel('Active time (seconds)', { exact: true }).nth(1).fill('2')
  await page.getByLabel('Waiting time (seconds)', { exact: true }).nth(1).fill('20')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Cooking acceptance recipe', exact: true }),
  ).toBeVisible()
  await page.getByRole('link', { name: 'Start cooking or baking', exact: true }).click()
  await expect(page.getByLabel('Name', { exact: true })).toHaveValue('Cooking acceptance recipe')
  await page.getByLabel('Servings', { exact: true }).fill('3')
  // Remove the second dependency to exercise two independent timers.
  const editors = page.locator('.cooking-stage-editor')
  await editors.nth(1).locator('summary').first().click()
  await editors.nth(1).locator('.vs__deselect').click()
  await page.getByRole('button', { name: 'Review schedule', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Review your schedule', exact: true }),
  ).toBeFocused()
  await expect(page.getByRole('button', { name: 'Save this schedule', exact: true })).toBeDisabled()
  await page
    .getByText('I reviewed the times, dependencies and uncertainties.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save this schedule', exact: true }).click()
  await expect(page.locator('.cooking-timeline > li')).toHaveCount(2)
  await expect(page).toHaveURL(/\/cooking\/cook-\d+$/)
  const detailUrl = page.url()
  const first = page.locator('[data-cooking-step="s1-1"]'),
    second = page.locator('[data-cooking-step="s2-1"]')
  await page.getByText('Ingredients · 3 servings', { exact: true }).click()
  await expect(page.locator('.cooking-ingredients')).toContainText('150.000002 g')
  await first.getByRole('button', { name: 'Start step: Prepare test dough', exact: true }).click()
  await expect(first.getByRole('button', { name: 'Pause', exact: true })).toBeEnabled()
  await second.getByRole('button', { name: 'Start step: Bake test dough', exact: true }).click()
  await first.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(first).toContainText('Paused')
  await expect(second.getByRole('button', { name: 'Pause', exact: true })).toBeVisible()
  await page.reload()
  await expect(first).toContainText('Paused')
  await first.getByRole('button', { name: 'Resume', exact: true }).click()
  await page.getByText('Reminders and delivery', { exact: true }).click()
  await page.getByText('Send Nextcloud reminders to me for this session', { exact: true }).click()
  await expect(
    page.getByText('Accepted by Nextcloud · device delivery unconfirmed', { exact: false }),
  ).toBeVisible({ timeout: 20000 })
  await context.setOffline(true)
  await expect(page.getByText(/Showing the last saved state/)).toBeVisible({ timeout: 12000 })
  await expect(first.getByRole('button', { name: 'Pause', exact: true })).toBeDisabled()
  await context.setOffline(false)
  await expect(first.getByRole('button', { name: 'Pause', exact: true })).toBeEnabled({
    timeout: 12000,
  })
  await expect(first).toContainText('Time elapsed')
  await first.getByRole('button', { name: 'Mark step complete', exact: true }).click()
  await second.getByRole('button', { name: 'Mark step complete', exact: true }).click()
  await expect(page.locator('.cooking-view > .context')).toContainText('Completed')
  await page.getByRole('link', { name: 'Start cooking or baking', exact: true }).click()
  await page.getByRole('combobox', { name: 'Recipes', exact: true }).click()
  await page.getByRole('option', { name: 'Cooking acceptance recipe', exact: true }).click()
  await page.getByRole('button', { name: 'Continue with recipe steps', exact: true }).click()
  await page.getByLabel('Preparation', { exact: true }).selectOption('bread')
  await page.getByLabel('Local date and time', { exact: true }).fill('2027-10-31T02:30')
  await page.getByLabel('IANA time zone', { exact: true }).fill('Europe/Berlin')
  await page.getByRole('button', { name: 'Review schedule', exact: true }).click()
  await expect(
    page.getByText('This local time occurs twice. Choose its first or second occurrence.', {
      exact: true,
    }),
  ).toBeVisible()
  await page.getByLabel('If the clock repeats this time', { exact: true }).selectOption('later')
  await editors.first().locator('summary').first().click()
  const waiting = editors.first().getByLabel('Rest or waiting time', { exact: true })
  await waiting.fill('172800')
  await page.getByRole('button', { name: 'Review schedule', exact: true }).click()
  await expect(page.locator('.cooking-review')).toContainText('+02:00[Europe/Berlin]')
  await expect(page.locator('.cooking-review')).toContainText('+01:00[Europe/Berlin]')
  await page
    .getByText('I reviewed the times, dependencies and uncertainties.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save this schedule', exact: true }).click()
  await expect(page.locator('.cooking-timeline > li')).toHaveCount(2)
  await page.getByRole('button', { name: 'Change desired finish', exact: true }).click()
  await page.getByLabel('Local date and time', { exact: true }).fill('2027-11-01T08:00')
  await page.getByRole('button', { name: 'Review schedule', exact: true }).click()
  await expect(page.locator('.cooking-review')).toContainText('Previous start')
  await page
    .getByText('I reviewed the times, dependencies and uncertainties.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save this schedule', exact: true }).click()
  await expect(page.locator('.cooking-view > .muted')).toContainText('2027-11-01T08:00')
  await page.locator('#cooking-title').scrollIntoViewIfNeeded()
  await page.screenshot({ path: 'var/nextcloud/cooking-desktop.png', fullPage: true })
  for (const scheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
    const a11y = await new AxeBuilder({ page })
      .include('.mealshop')
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze()
    expect(a11y.violations).toEqual([])
    for (const width of [1440, 768, 390, 320]) {
      await page.setViewportSize({ width, height: 1000 })
      await expect(page.locator('#app-navigation-vue')).toHaveAttribute(
        'aria-hidden',
        width <= 1024 ? 'true' : 'false',
      )
      await page.locator('#cooking-title').scrollIntoViewIfNeeded()
      await page.screenshot({
        path: `var/nextcloud/cooking-${scheme}-${width}.png`,
        animations: 'disabled',
      })
      expect(
        await page.locator('.cooking-view').evaluate((e) => e.scrollWidth <= e.clientWidth),
      ).toBe(true)
      expect(await page.locator('.mealshop').evaluate((e) => e.scrollWidth <= e.clientWidth)).toBe(
        true,
      )
    }
  }
  await page.goto(detailUrl)
  await expect(page.locator('.cooking-view > .context')).toContainText('Completed')
  expect(errors).toEqual([])
})
