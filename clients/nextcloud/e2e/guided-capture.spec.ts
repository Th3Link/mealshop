import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { openFixture, recordFixture } from './fixtures'

test('pasted recipe text is reviewed, saved with an own unit weight and rated', async ({
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
  await recordFixture(page, 'guided-capture.spec.ts')
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).last().fill('Capture attendee')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()

  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.getByRole('link', { name: 'Add recipe', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Add recipe', exact: true })).toBeVisible()
  const chooser = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(chooser.violations).toEqual([])
  await page.getByRole('button', { name: /^Paste or dictate/ }).click()
  await page
    .getByLabel('Recipe text', { exact: true })
    .fill(
      [
        'Synthetic capture bowl',
        'for 2 people',
        '',
        'Ingredients',
        '1,500 g Oats',
        '1/2 tbsp Oats',
        '1/3 g Oats',
        '2 synthetic unknown food',
        '',
        'Method',
        '1. Mix everything.',
      ].join('\n'),
    )
  const found = page.locator('.capture-found')
  await expect(found).toContainText('Synthetic capture bowl · 2 servings')
  await expect(found).toContainText('0.5 tbsp Oats')
  // Neither a grouped-looking decimal nor a repeating fraction is turned into a number.
  await expect(found).toContainText('1,500 g Oats')
  await expect(found).toContainText('1/3 g Oats')
  await page.getByRole('button', { name: 'Review recipe', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Review and save', exact: true })).toBeVisible()
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(
    'Synthetic capture bowl',
  )
  await expect(page.locator('.import-source-line')).toHaveCount(4)
  const rows = page.locator('.ingredient-entry')
  await expect(rows.nth(0).getByLabel('Grams', { exact: true })).toHaveValue('')
  await rows.nth(0).getByLabel('Grams', { exact: true }).fill('100.5')
  await rows.nth(2).getByLabel('Grams', { exact: true }).fill('0.333333')
  // The unidentified line cannot be saved: it is identified or removed by the cook.
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Review and save', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Remove 4', exact: true }).click()
  // A spoon without a weight is not an obstacle: it saves and nutrition says what is missing.
  const spoonWeight = page.getByLabel('Weight of 1 tbsp in grams', { exact: true })
  await expect(spoonWeight).not.toHaveAttribute('required')
  await expect(rows.nth(1)).toContainText('weight unknown')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Synthetic capture bowl', exact: true }),
  ).toBeVisible()
  await expect(page.getByText(/^0\.5\s+tbsp\s+· Oats/)).toContainText('weight unknown')
  await expect(page.locator('.nutrition-basis')).toContainText('Incomplete — no weight for: Oats.')
  await expect(page.locator('.nutrition-tiles li').first()).toContainText('incomplete')
  expect(
    (
      await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.getByRole('link', { name: 'Edit recipe', exact: true }).click()
  await page.getByLabel('Weight of 1 tbsp in grams', { exact: true }).fill('7.5')
  const review = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(review.violations).toEqual([])
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(
    page.getByRole('heading', { name: 'Synthetic capture bowl', exact: true }),
  ).toBeVisible()
  await expect(page.getByText(/^3\.75 g · Oats/)).toContainText('own weight')
  await expect(page.locator('.nutrition-basis')).toContainText('Uses your own weights for: Oats.')
  await expect(page.locator('.nutrition-basis')).not.toContainText('Incomplete')
  await expect(page.locator('.nutrition-tiles li')).toHaveCount(5)

  await page.getByRole('button', { name: 'Cooked it? Rate it', exact: true }).click()
  await page
    .getByRole('group', { name: 'Household rating', exact: true })
    .getByRole('radio', { name: '4 / 5', exact: true })
    .check({ force: true })
  await page.getByRole('button', { name: 'Save rating', exact: true }).click()
  await expect(page.getByText('Saved to your cooking history.', { exact: true })).toBeVisible()
  await expect(page.getByRole('img', { name: '4 out of 5 stars', exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  const card = page.locator('.recipe-cards li').filter({ hasText: 'Synthetic capture bowl' })
  await expect(card.getByRole('img', { name: '4 out of 5 stars', exact: true })).toBeVisible()
  await expect(card).toContainText('Cooked 1×')
  for (const width of [390, 1440]) {
    await page.setViewportSize({ width, height: 900 })
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    ).toBe(true)
  }
  expect(errors).toEqual([])
})

test('a suggestion handed in by a companion waits in the inbox and is saved as a reviewed import', async ({
  page,
}) => {
  test.setTimeout(180000)
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('#mealshop-language').selectOption('en')
  await openFixture(page, 'guided-capture.spec.ts')
  await page.getByRole('link', { name: 'Assistant tools', exact: true }).click()
  const user = process.env.MEALSHOP_TEST_USER ?? ''
  const access = page.getByRole('combobox', {
    name: `Assistant access for · ${user} · Owner`,
    exact: true,
  })
  if ((await access.inputValue()) !== 'create') {
    await access.selectOption('create')
    await page.getByRole('button', { name: 'Save assistant permissions', exact: true }).click()
    await expect(page.getByText('Assistant permissions saved.', { exact: true })).toBeVisible()
  }
  const url = await page.getByLabel('Household MCP endpoint', { exact: true }).inputValue()
  const stamp = Date.now()
  const sourced = `Synthetic reel oats ${stamp}`,
    plain = `Synthetic dictated note ${stamp}`
  const source = `https://example.org/reel/${stamp}`
  const lines = [
    {
      title: sourced,
      language: 'en',
      yield_text: 'for 2 people',
      ingredients: ['100 g Oats', '2 tbsp Oats'],
      steps: ['Mix everything.'],
      source_url: source,
      note: 'Synthetic transcript for browser verification',
    },
    { title: plain, ingredients: ['1 synthetic unknown food'] },
  ]
  const send = () =>
    execFileSync(
      'python3',
      [fileURLToPath(new URL('../../../scripts/suggest_recipe.py', import.meta.url))],
      {
        input: lines.map((line) => JSON.stringify(line)).join('\n') + '\n',
        env: {
          PATH: process.env.PATH ?? '',
          MEALSHOP_MCP_URL: url,
          MEALSHOP_MCP_USER: user,
          MEALSHOP_MCP_APP_PASSWORD: process.env.MEALSHOP_TEST_PASSWORD ?? '',
        },
        encoding: 'utf8',
      },
    )
  expect(send()).toMatch(/^1\tadded\tsuggestion-[0-9a-f]{32}\n2\tadded\tsuggestion-[0-9a-f]{32}\n$/)
  // Sending the same pipeline output again adds nothing.
  expect(send()).toMatch(/^1\twaiting\t.+\n2\twaiting\t.+\n$/)

  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await page.getByRole('link', { name: /suggestions waiting$/ }).click()
  const inbox = page.getByRole('region', { name: 'Suggestions waiting for you', exact: true })
  await expect(inbox).toContainText(sourced)
  await expect(inbox.locator(`a[href="${source}"]`)).toHaveText('example.org')
  expect(
    (
      await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  await page.setViewportSize({ width: 390, height: 900 })
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  )
  await page.screenshot({ path: 'test-results/suggestion-inbox-390.png', animations: 'disabled' })
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.screenshot({ path: 'test-results/suggestion-inbox-1440.png', animations: 'disabled' })

  // Discarding needs a second click.
  await inbox.getByRole('button', { name: `Discard: ${plain}`, exact: true }).click()
  await expect(inbox).toContainText(plain)
  await inbox.getByRole('button', { name: `Really discard?: ${plain}`, exact: true }).click()
  await expect(inbox).not.toContainText(plain)

  await inbox.getByRole('button', { name: `Review: ${sourced}`, exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Review and save', exact: true })).toBeVisible()
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(sourced)
  await expect(page.getByLabel('Servings', { exact: true })).toHaveValue('2')
  await expect(page.locator('.import-source-line')).toHaveCount(2)
  await page.getByRole('button', { name: 'Use a rough estimate: 15 g', exact: true }).click()
  await expect(page.getByLabel('Weight of 1 tbsp in grams', { exact: true })).toHaveValue('15')
  await page.screenshot({
    path: 'test-results/suggestion-review-1440.png',
    animations: 'disabled',
    fullPage: true,
  })
  await page.getByRole('checkbox', { name: /reviewed/i }).check({ force: true })
  await page.getByRole('button', { name: 'Save reviewed import', exact: true }).click()
  await expect(page.getByRole('heading', { name: sourced, exact: true })).toBeVisible()
  await expect(page.getByText(/^30 g · Oats/)).toContainText('rough estimate')
  await expect(page.locator(`a[href="${source}"]`).first()).toBeVisible()
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await expect(page.locator('.recipe-cards')).toContainText(sourced)
  // The saved suggestion left the inbox.
  await page.getByRole('link', { name: 'Add recipe', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Add recipe', exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: /^Paste or dictate/ })).toBeVisible()
  // The inbox section disappears entirely once nothing waits.
  await expect(page.locator('.capture-inbox').getByText(sourced)).toHaveCount(0)
  expect(errors).toEqual([])
})

test('the Assistant reads a picture into the capture text when the instance offers it', async ({
  page,
}) => {
  test.setTimeout(120000)
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  // The local instance has no Assistant provider: its three answers are simulated here, the
  // rest of the page and the picture upload are real.
  let polls = 0
  const sent: string[] = []
  await page.route('**/proxy/mealshop/api/assist', (route) =>
    route.fulfill({ json: { speech: false, picture: true, handwriting: false } }),
  )
  await page.route('**/proxy/mealshop/api/assist/transcriptions?kind=*', (route) => {
    sent.push(`${route.request().method()} ${new URL(route.request().url()).search}`)
    return route.fulfill({ status: 202, json: { id: 7, kind: 'picture', status: 'waiting' } })
  })
  await page.route('**/proxy/mealshop/api/assist/transcriptions/7', (route) =>
    route.fulfill({
      json:
        polls++ === 0
          ? { id: 7, kind: 'picture', status: 'running', progress: 0.5 }
          : {
              id: 7,
              kind: 'picture',
              status: 'done',
              text: 'Synthetic picture bowl\nfor 2 people\n\nIngredients\n100 g Oats',
            },
    }),
  )
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('#mealshop-language').selectOption('en')
  await openFixture(page, 'guided-capture.spec.ts')
  await page.getByRole('link', { name: 'Add recipe', exact: true }).click()
  await page.getByRole('button', { name: /^Photo or screenshot/ }).click()
  const read = page.getByRole('button', { name: 'Read printed text', exact: true })
  // Nothing can be read before a picture is chosen; handwriting and recording are not offered.
  await expect(read).toBeDisabled()
  await expect(page.getByRole('button', { name: 'Read handwriting', exact: true })).toHaveCount(0)
  await expect(
    page.getByRole('button', { name: 'Record and transcribe', exact: true }),
  ).toHaveCount(0)
  await page.locator('.capture-file input[type="file"]').setInputFiles({
    name: 'synthetic.png',
    mimeType: 'image/png',
    buffer: Buffer.from(
      'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==',
      'base64',
    ),
  })
  await expect(read).toBeEnabled()
  await expect(page.locator('.capture-photo')).toContainText('Assistant provider your Nextcloud')
  await read.click()
  await expect(page.getByLabel('Recipe text', { exact: true })).toHaveValue(
    /Synthetic picture bowl/,
    {
      timeout: 15000,
    },
  )
  expect(sent).toEqual(['POST ?kind=picture'])
  // The words are only text: the deterministic reader shows what it found, nothing is saved.
  const found = page.locator('.capture-found')
  await expect(found).toContainText('Synthetic picture bowl · 2 servings')
  await expect(found).toContainText('100 g Oats')
  expect(
    (
      await new AxeBuilder({ page })
        .include('.mealshop')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations,
  ).toEqual([])
  expect(errors).toEqual([])
})
