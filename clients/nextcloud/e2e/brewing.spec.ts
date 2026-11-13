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
  await recordFixture(page, 'brewing.spec.ts')
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  if (!id) throw new Error('Missing owned brewing fixture')
  page.on('dialog', (d) => void d.accept())
  return {
    base: `${embedded}/h/${id}/brewing`,
    proxy: `/apps/app_api/proxy/mealshop/api/households/${id}/domain/api/household/brewing`,
  }
}
async function review(page: Page) {
  await page.getByRole('button', { name: 'Review brewing change', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Review brewing change', exact: true }),
  ).toBeFocused()
}
async function save(page: Page) {
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await page
    .getByText('I reviewed this change and its quantities, times and equipment.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Review brewing change', exact: true }),
  ).toHaveCount(0)
}
async function equipment(page: Page, base: string, name: string) {
  await page.goto(`${base}/equipment/new`)
  await page.getByLabel('Name', { exact: true }).fill(name)
  await page.getByRole('button', { name: 'Declared usable capacity', exact: true }).click()
  await page.getByLabel('Declared usable capacity', { exact: true }).fill('25.000001')
  await review(page)
  await save(page)
  await expect(page.locator('.plain-list')).toContainText(name)
}
async function recipe(page: Page, base: string) {
  await page.goto(`${base}/recipes/new`)
  const template = page.getByRole('combobox', { name: 'Starting template', exact: true })
  await template.selectOption('grainfather')
  await expect(page.locator('.brew-phase')).toHaveCount(11)
  await template.selectOption('braumeister')
  await expect(page.locator('.brew-phase')).not.toHaveCount(0)
  await template.selectOption('extract')
  await page.getByLabel('Recipe title', { exact: true }).fill('Brewing acceptance recipe')
  await page.getByLabel('Target batch volume', { exact: true }).fill('20.000001')
  while ((await page.locator('.brew-phase').count()) > 2)
    await page
      .getByRole('button', { name: /^Remove phase:/ })
      .last()
      .click()
  await page.getByRole('button', { name: 'Add brewing ingredient', exact: true }).click()
  await page.getByLabel('Ingredient', { exact: true }).fill('Synthetic brewing ingredient')
  await page.getByRole('button', { name: 'Quantity', exact: true }).click()
  await page.getByLabel('Quantity', { exact: true }).fill('100.000001')
  const first = page.locator('.brew-phase').first()
  await first.getByText('Enter a duration', { exact: true }).click()
  await first
    .getByRole('combobox', { name: 'Rest or waiting time · Unit', exact: true })
    .selectOption('1')
  await first.getByLabel('Rest or waiting time', { exact: true }).fill('2')
  await first.getByRole('button', { name: 'Target temperature', exact: true }).click()
  await first.getByLabel('Target temperature', { exact: true }).fill('20.000001')
  await review(page)
  await expect(page.locator('.brew-review')).toContainText('20.000001 l')
  await expect(page.locator('.brew-review')).toContainText('100.000001 g')
  await save(page)
  await expect(
    page.getByRole('heading', { name: 'Brewing acceptance recipe', exact: true }),
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
        await page.locator('.brewing-page h1').scrollIntoViewIfNeeded()
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
test('native brew authoring, equipment, exact readings, explicit phase completion and offline recovery', async ({
  page,
  context,
}) => {
  test.setTimeout(240000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  const { base, proxy } = await setup(page)
  await equipment(page, base, 'Brewing acceptance kettle')
  await recipe(page, base)
  await audit(page, 'brewing-recipe')
  await page.getByRole('link', { name: 'Plan a batch', exact: true }).click()
  await page.getByLabel('Batch name', { exact: true }).fill('Brewing acceptance batch')
  const start = new Date(Date.now() + 15000).toISOString().slice(0, 19)
  // Use a deliberate IANA zone, independently of the browser's device zone.
  await page.getByLabel('IANA time zone', { exact: true }).fill('UTC')
  await page.getByLabel('Local date and time', { exact: true }).fill(start)
  const assignments = page.locator('.brew-assignment')
  await assignments.first().getByRole('combobox').fill('Brewing acceptance kettle')
  await page.getByRole('option', { name: /Brewing acceptance kettle/ }).click()
  await assignments.nth(1).getByRole('combobox').fill('Brewing acceptance kettle')
  await page.getByRole('option', { name: /Brewing acceptance kettle/ }).click()
  await review(page)
  await save(page)
  await expect(page.locator('.cooking-timeline > li')).toHaveCount(2)
  const batchUrl = page.url(),
    first = page.locator('[data-cooking-step="phase-1-1"]'),
    second = page.locator('[data-cooking-step="phase-2-1"]')
  await expect(first.getByRole('button', { name: /Start step:/ })).toBeEnabled({ timeout: 25000 })
  await first.getByRole('button', { name: /Start step:/ }).click()
  await expect(first.getByRole('button', { name: 'Pause', exact: true })).toBeEnabled()
  await expect(second.getByRole('button', { name: /Start step:/ })).toBeDisabled()
  await expect(first).toContainText('Time elapsed', { timeout: 15000 })
  await expect(second.getByRole('button', { name: /Start step:/ })).toBeDisabled()
  await first.getByRole('button', { name: 'Mark step complete', exact: true }).click()
  await second.getByRole('button', { name: /Start step:/ }).click()
  await expect(second).toContainText('In progress · complete manually')
  await expect(second.getByRole('button', { name: 'Pause', exact: true })).toHaveCount(0)
  await page.getByRole('button', { name: 'Record measurement', exact: true }).click()
  await page.getByLabel('Original measured value', { exact: true }).fill('1.050001')
  await page
    .getByRole('combobox', { name: 'Measurement source', exact: true })
    .selectOption('instrument')
  await page.getByLabel('Instrument name', { exact: true }).fill('Synthetic hydrometer')
  await page.getByLabel('Instrument identifier', { exact: true }).fill('synthetic-device')
  await review(page)
  await save(page)
  await expect(page.locator('.brew-readings')).toContainText('1.050001')
  await page.getByRole('button', { name: 'Correct measurement', exact: true }).click()
  await page.getByLabel('Original measured value', { exact: true }).fill('1.050002')
  await page
    .getByLabel('Reason for correction', { exact: true })
    .fill('Corrected synthetic transcription')
  await review(page)
  await save(page)
  await expect(page.locator('.brew-readings .plain-list > li')).toHaveCount(2)
  await audit(page, 'brewing-batch')
  await context.setOffline(true)
  await expect(page.getByText(/Showing the last saved state/)).toBeVisible({ timeout: 20000 })
  await expect(
    second.getByRole('button', { name: 'Mark step complete', exact: true }),
  ).toBeDisabled()
  await context.setOffline(false)
  await page.getByRole('button', { name: 'Reload saved state', exact: true }).click()
  await expect(
    second.getByRole('button', { name: 'Mark step complete', exact: true }),
  ).toBeEnabled()
  await page.reload()
  await expect(page.locator('.brew-readings .plain-list > li')).toHaveCount(2)
  await expect(page).toHaveURL(batchUrl)
  const response = await page.request.get(proxy)
  expect(response.ok()).toBe(true)
  const state = (await response.json()) as {
    batches: { run: { brew_recipe: unknown; servings?: number; recipe?: unknown } }[]
  }
  expect(state.batches[0]?.run.brew_recipe).toBeDefined()
  expect(state.batches[0]?.run.servings).toBeUndefined()
  await page.goto(`${base}/batches/new`)
  await page
    .getByRole('combobox', { name: 'Brew recipes', exact: true })
    .selectOption({ label: 'Brewing acceptance recipe · 1' })
  await page
    .locator('.brew-assignment')
    .first()
    .getByRole('combobox')
    .fill('Brewing acceptance kettle')
  await page.getByRole('option', { name: /Brewing acceptance kettle/ }).click()
  await expect(page.locator('.brew-resource-uses')).toContainText(['Brewing acceptance batch'])
  await page.getByRole('button', { name: 'Review brewing change', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('overlapping reservations')
  await expect(page.getByLabel('Batch name', { exact: true })).toHaveValue(
    'Brewing acceptance recipe',
  )

  expect(errors).toEqual([])
})

test('brewing readers see pinned recipes and readings but cannot mutate or subscribe', async ({
  page,
  browser,
}) => {
  test.setTimeout(240000)
  const { base, proxy } = await setup(page)
  await recipe(page, base)
  const result = await page.request.get(proxy)
  expect(result.ok()).toBe(true)
  const state = (await result.json()) as {
    revision: number
    recipes: { record: { id: string; revision: number } }[]
  }
  const r = state.recipes[0]?.record
  if (!r) throw new Error('Missing owned brew recipe')
  const created = await page.request.post(proxy, {
    headers: { 'X-Mealshop-Request': '1' },
    data: {
      expected_revision: state.revision,
      idempotency_key: crypto.randomUUID(),
      reviewed: true,
      command: {
        operation: 'create_batch',
        draft: {
          name: 'Brewing acceptance reader batch',
          recipe_id: r.id,
          recipe_revision: r.revision,
          brew_day: { local: '2030-01-15T08:00:00', timezone: 'UTC', fold: 'reject' },
          yeast_preparation: { choice: 'skip' },
          equipment: {},
        },
      },
    },
  })
  expect(created.ok()).toBe(true)
  const saved = (await created.json()) as { brewing: { batches: { run: { id: string } }[] } }
  const run = saved.brewing.batches[0]?.run.id
  if (!run) throw new Error('Missing owned batch')
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
  const readerContext = await browser.newContext()
  try {
    const reader = await readerContext.newPage()
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
    const homeId = base.match(/\/h\/(home-\d+)\//)?.[1]
    await expect(reader.locator(`.record-list a[href$="/h/${homeId}/recipes"]`)).toBeVisible()
    await reader.goto(`${base}/batches/${run}`)
    await expect(
      reader.getByText('You have read-only access to this household.', { exact: true }),
    ).toBeVisible()
    await expect(reader.locator('.cooking-timeline')).toBeVisible()
    await expect(reader.locator('.cooking-timeline button').first()).toBeDisabled()
    await expect(
      reader.getByRole('button', { name: 'Record measurement', exact: true }),
    ).toHaveCount(0)
    for (const suffix of ['', '/preview'])
      expect(
        (
          await reader.request.post(proxy + suffix, {
            headers: { 'X-Mealshop-Request': '1' },
            data: {},
          })
        ).status(),
      ).toBe(403)
    await audit(reader, 'brewing-reader')
  } finally {
    await readerContext.close()
  }
})

test('brewing keeps drafts on concurrent changes and retries an uncertain save without duplication', async ({
  page,
}) => {
  test.setTimeout(120000)
  const { base, proxy } = await setup(page)
  await page.goto(`${base}/equipment/new`)
  await page.getByLabel('Name', { exact: true }).fill('Brewing acceptance retained draft')
  await review(page)
  const current = (await (await page.request.get(proxy)).json()) as { revision: number }
  const concurrent = await page.request.post(proxy, {
    headers: { 'X-Mealshop-Request': '1' },
    data: {
      expected_revision: current.revision,
      idempotency_key: crypto.randomUUID(),
      reviewed: true,
      command: {
        operation: 'save_equipment',
        id: null,
        equipment: {
          name: 'Brewing acceptance other vessel',
          kind: 'kettle',
          model: 'Synthetic',
          capacity: null,
          notes: '',
        },
      },
    },
  })
  expect(concurrent.ok()).toBe(true)
  await page
    .getByText('I reviewed this change and its quantities, times and equipment.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByLabel('Name', { exact: true })).toHaveValue(
    'Brewing acceptance retained draft',
  )
  await page.getByRole('button', { name: 'Load latest state; keep entries', exact: true }).click()
  await expect(page.getByLabel('Name', { exact: true })).toHaveValue(
    'Brewing acceptance retained draft',
  )
  await review(page)
  let intercepted = false
  await page.route('**/api/household/brewing', async (route) => {
    if (route.request().method() !== 'POST' || intercepted) {
      await route.continue()
      return
    }
    intercepted = true
    const response = await route.fetch()
    expect(response.ok()).toBe(true)
    await route.abort('failed')
  })
  await page
    .getByText('I reviewed this change and its quantities, times and equipment.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Retry the same save', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Retry the same save', exact: true }).click()
  await expect(page.locator('.plain-list')).toContainText('Brewing acceptance retained draft')
  const state = (await (await page.request.get(proxy)).json()) as {
    equipment: { details: { name: string } }[]
  }
  expect(
    state.equipment.filter((e) => e.details.name === 'Brewing acceptance retained draft'),
  ).toHaveLength(1)
  expect(state.equipment).toHaveLength(2)
})

test('empty brewing views lead directly to the matching recipe or equipment form', async ({
  page,
}) => {
  const { base } = await setup(page)
  await page.goto(`${base}/batches`)
  await page
    .locator('.brew-overview-empty')
    .getByRole('link', { name: 'Create brew recipe', exact: true })
    .click()
  await expect(page.getByRole('heading', { name: 'Create brew recipe', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Equipment', exact: true }).click()
  await page
    .locator('.brew-overview-empty')
    .getByRole('link', { name: 'Add equipment', exact: true })
    .click()
  await expect(page.getByRole('heading', { name: 'Add equipment', exact: true })).toBeVisible()
  await expect(page.getByLabel('Name', { exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Brew recipes', exact: true }).click()
  await page
    .locator('.brew-overview-empty')
    .getByRole('link', { name: 'Create brew recipe', exact: true })
    .click()
  await expect(page.getByRole('combobox', { name: 'Starting template', exact: true })).toBeVisible()
})
