import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { recordFixture } from './fixtures'

test('person presets, explicit exceptions and availability survive native-compatible web editing', async ({
  page,
}) => {
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'people.spec.ts')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).fill('P2 synthetic web person')
  await page
    .getByRole('combobox', { name: 'Dietary classification', exact: true })
    .selectOption('vegan')
  await page.getByRole('combobox', { name: 'Explicit exceptions', exact: true }).fill('egg')
  await page.getByRole('option', { name: 'Chicken egg raw', exact: true }).first().click()
  await page.getByText('Chicken egg raw · Only household-produced', { exact: true }).click()
  await expect(page.getByRole('checkbox', { name: /Only household-produced/ })).toBeChecked()
  await page
    .getByLabel('Maximum active preparation time (seconds, optional)', { exact: true })
    .fill('900')
  await page.getByRole('button', { name: 'Add availability window', exact: true }).click()
  await page.getByRole('combobox', { name: 'Meal', exact: true }).selectOption('dinner')
  for (const day of ['Mon', 'Tue', 'Wed', 'Thu', 'Sat', 'Sun'])
    await page.getByText(day, { exact: true }).click()
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByText('Changes saved.', { exact: true })).toBeVisible()
  await page.reload()
  await expect(page.getByLabel('Name', { exact: true })).toHaveValue('P2 synthetic web person')
  await expect(
    page.getByRole('combobox', { name: 'Dietary classification', exact: true }),
  ).toHaveValue('vegan')
  await expect(page.getByRole('checkbox', { name: /Only household-produced/ })).toBeChecked()
  await expect(
    page.getByLabel('Maximum active preparation time (seconds, optional)', { exact: true }),
  ).toHaveValue('900')
  await expect(page.getByRole('checkbox', { name: 'Fri', exact: true })).toBeChecked()
  await expect(page.getByRole('checkbox', { name: 'Mon', exact: true })).not.toBeChecked()
  const results = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(results.violations).toEqual([])
})
