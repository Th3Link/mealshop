import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture, confirmPlan } from './fixtures'

test('typed plan sentences open the reviewed planner and never change the plan by themselves', async ({
  page,
}) => {
  test.setTimeout(180000)
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'guided-meal-plan.spec.ts')
  for (const title of [
    'Synthetic plan chili',
    'Synthetic plan pasta bake',
    'Synthetic plan pasta salad',
  ]) {
    await page.getByRole('link', { name: 'Recipes', exact: true }).click()
    await page.getByRole('link', { name: 'Add recipe', exact: true }).click()
    await page.getByRole('link', { name: 'Create recipe', exact: true }).click()
    await page.getByLabel('Recipe title', { exact: true }).fill(title)
    await page.getByLabel('Servings', { exact: true }).fill('2')
    await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
    await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill('Oats')
    await page.getByRole('option', { name: 'Oats', exact: true }).click()
    await page.getByLabel('Grams', { exact: true }).fill('100')
    if (title !== 'Synthetic plan chili') {
      await page.getByRole('button', { name: 'Add step', exact: true }).click()
      await page.getByLabel('1. Instruction', { exact: true }).fill('Mix the synthetic fixture')
      await page.getByLabel('Active work', { exact: true }).fill('10')
    }
    await page.getByRole('button', { name: 'Save changes', exact: true }).click()
    await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible()
  }
  const day = (offset: number) =>
    page.evaluate((days) => {
      const date = new Date()
      date.setDate(date.getDate() + days)
      return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
    }, offset)
  const tomorrow = await day(1)
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  const command = page.getByLabel('What should go on the plan?', { exact: true })
  const answer = page.locator('.plan-command-answer')
  const go = page.getByRole('button', { name: 'Go', exact: true })
  // Without people a meal cannot be planned; the answer says what to do instead of failing.
  await command.fill('synthetic chili tomorrow evening')
  await go.click()
  await expect(answer).toContainText('Add at least one person under Household first')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).last().fill('Plan attendee')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  const slot = page.locator(`.meal-slot[data-date="${tomorrow}"][data-meal="dinner"]`)

  // Nonsense and unknown recipes are said plainly; nothing opens or changes.
  await command.fill('hm')
  await go.click()
  await expect(answer).toContainText('No recipe matches “hm”')
  await command.fill('Lasagne tomorrow evening')
  await go.click()
  await expect(answer).toContainText('No recipe matches “lasagne”')
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()

  // Two titles fit "pasta": both are offered and nothing is reviewed before one is picked.
  await command.fill('synthetic pasta tomorrow evening for 3 people')
  await go.click()
  await expect(answer).toContainText('Which recipe did you mean?')
  await expect(page.locator('.mealplan-details')).toHaveCount(0)
  await answer.getByRole('button', { name: 'Synthetic plan pasta bake', exact: true }).click()
  const review = page.getByRole('heading', { name: 'Review meal', exact: true }).last()
  await expect(review).toBeVisible()
  await expect(page.locator('.mealplan-sidebar')).toContainText(
    'Synthetic plan pasta bake · 3 servings',
  )
  await expect(page.locator('.plan-timing')).toBeVisible()
  // The understood sentence alone planned nothing.
  await expect(slot).not.toContainText('Synthetic plan pasta bake')
  expect(
    (
      await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  for (const width of [390, 1440]) {
    await page.setViewportSize({ width, height: 900 })
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    ).toBe(true)
    await page.screenshot({
      path: `test-results/guided-plan-review-${width}.png`,
      animations: 'disabled',
    })
  }
  // Nothing concerns the only attendee: one button applies, no acknowledgement is asked.
  await expect(page.locator('.mealplan-sidebar')).toContainText(
    'Nothing to flag for the people eating',
  )
  await expect(page.getByRole('button', { name: 'Add to plan', exact: true })).toBeVisible()
  expect(await confirmPlan(page)).toBe('direct')
  await expect(slot).toContainText('Synthetic plan pasta bake')
  await expect(slot).toContainText('3 Servings')

  // German sentences work whatever the display language; removal is reviewed too.
  await command.fill('Lösche morgen das Abendessen')
  await go.click()
  await expect(answer).toContainText('Understood: Remove')
  await expect(slot).toContainText('Synthetic plan pasta bake')
  expect(await confirmPlan(page, true)).toBe('direct')
  await expect(slot).not.toContainText('Synthetic plan pasta bake')
  await command.fill('Lösche morgen das Abendessen')
  await go.click()
  await expect(answer).toContainText('Nothing is planned then')

  // A recipe without steps can be planned; its unknown duration is flagged and must be
  // acknowledged, so the one-button path is not offered.
  await command.fill('synthetic chili tomorrow evening')
  await go.click()
  await expect(page.locator('.mealplan-sidebar')).toContainText('preparation time is unknown')
  await expect(page.locator('.plan-timing')).toHaveCount(0)
  await expect(page.locator('.plan-apply-direct')).toHaveCount(0)
  expect(await confirmPlan(page)).toBe('reviewed')
  await expect(slot).toContainText('Synthetic plan chili')

  await command.fill('suggest something for tomorrow lunch')
  await go.click()
  await expect(
    page.getByRole('heading', { name: 'Find suitable recipes', exact: true }),
  ).toBeVisible()
  await expect(page.locator('.suggestion-item')).toHaveCount(3)
  await page.getByRole('button', { name: 'Cancel', exact: true }).click()

  // "Show" only moves the calendar.
  const far = await day(21)
  await command.fill(`zeig ${far}`)
  await go.click()
  await expect(page.locator(`.meal-slot[data-date="${far}"][data-meal="dinner"]`)).toBeVisible()
  await page.locator('#mealshop-language').selectOption('de')
  await expect(page.getByLabel('Was soll auf den Plan?', { exact: true })).toBeVisible()
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.screenshot({
    path: 'test-results/guided-plan-week-de-1440.png',
    animations: 'disabled',
  })
  expect(errors).toEqual([])
})
