import { test, expect, type Page } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { Client } from '@modelcontextprotocol/client'
import { StdioClientTransport } from '@modelcontextprotocol/client/stdio'
import { resolve } from 'node:path'
import { z } from 'zod'
import { recordFixture } from './fixtures'

const embedded = '/index.php/apps/app_api/embedded/mealshop/main'
async function login(page: Page, reader = false) {
  await page.goto(`${embedded}/households`)
  await page
    .locator('#user')
    .fill(process.env[reader ? 'MEALSHOP_READER_USER' : 'MEALSHOP_TEST_USER'] ?? '')
  await page
    .locator('#password')
    .fill(process.env[reader ? 'MEALSHOP_READER_PASSWORD' : 'MEALSHOP_TEST_PASSWORD'] ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('#mealshop-language').selectOption('en')
}
async function setup(page: Page) {
  await login(page)
  const name = `Browser verification ${Date.now()}`
  await page.getByLabel('Household name', { exact: true }).fill(name)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'mcp.spec.ts')
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  if (!id) throw new Error('Recorded MCP fixture identity missing')
  await page.getByRole('link', { name: 'Assistant tools', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Assistant tools', exact: true })).toBeVisible()
  return { id, name }
}
function access(page: Page, user = process.env.MEALSHOP_TEST_USER ?? '') {
  return page.getByRole('combobox', {
    name: `Assistant access for · ${user} · ${user === process.env.MEALSHOP_TEST_USER ? 'Owner' : 'Reader'}`,
    exact: true,
  })
}
async function save(page: Page) {
  await page.getByRole('button', { name: 'Save assistant permissions', exact: true }).click()
  await expect(page.getByText('Assistant permissions saved.', { exact: true })).toBeVisible()
}
async function connect(url: string, reader = false) {
  const client = new Client(
    { name: 'Mealshop Nextcloud acceptance', version: '1' },
    { versionNegotiation: { mode: 'auto' } },
  )
  await client.connect(
    new StdioClientTransport({
      command: 'python3',
      args: [resolve('scripts/mcp_stdio.py')],
      env: {
        MEALSHOP_MCP_URL: url,
        MEALSHOP_MCP_USER:
          process.env[reader ? 'MEALSHOP_READER_USER' : 'MEALSHOP_TEST_USER'] ?? '',
        MEALSHOP_MCP_APP_PASSWORD:
          process.env[reader ? 'MEALSHOP_READER_PASSWORD' : 'MEALSHOP_TEST_PASSWORD'] ?? '',
      },
      stderr: 'pipe',
    }),
  )
  return client
}
async function audit(page: Page, name: string) {
  for (const language of ['en', 'de']) {
    await page.setViewportSize({ width: 1440, height: 1000 })
    await page.locator('#mealshop-language').selectOption(language)
    for (const scheme of ['light', 'dark'] as const) {
      await page.emulateMedia({ colorScheme: scheme, reducedMotion: 'reduce' })
      for (const width of [1440, 390, 320]) {
        await page.setViewportSize({ width, height: 1000 })
        await expect
          .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
          .toBeLessThanOrEqual(width)
        await page.screenshot({
          path: `test-results/${name}-${language}-${scheme}-${width}.png`,
          animations: 'disabled',
          fullPage: true,
        })
        expect(
          (
            await new AxeBuilder({ page })
              .include('.mealshop')
              .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
              .analyze()
          ).violations,
        ).toEqual([])
      }
    }
  }
  await page.setViewportSize({ width: 1440, height: 1000 })
  await page.locator('#mealshop-language').selectOption('en')
}

test('native MCP grants, official client creation, source restriction and reader scope', async ({
  page,
  browser,
}) => {
  test.setTimeout(240000)
  const errors: string[] = []
  page.on('pageerror', (e) => errors.push(e.message))
  const home = await setup(page)
  await expect(access(page)).toHaveValue('disabled')
  await expect(page.getByText('Your assistant access: Disabled', { exact: true })).toBeVisible()
  await access(page).selectOption('create')
  await save(page)
  const url = await page.getByLabel('Household MCP endpoint', { exact: true }).inputValue()
  expect(new URL(url).origin).toBe(process.env.MEALSHOP_TEST_URL)
  expect(new URL(url).pathname).toMatch(
    new RegExp(
      `^/(?:index.php/)?apps/app_api/proxy/mealshop/api/households/${home.id}/domain/api/household/mcp$`,
    ),
  )
  let client = await connect(url)
  try {
    expect((await client.listTools()).tools.map((tool) => tool.name)).toContain('recipe_create')
    const found = await client.callTool({
      name: 'ingredient_search',
      arguments: { query: 'Haferkorn' },
    })
    expect(found.isError).toBe(false)
    expect(found.structuredContent).toMatchObject({
      items: expect.arrayContaining([expect.objectContaining({ id: 'oats' })]),
    })
    const recipe = {
      title: 'MCP acceptance recipe',
      language: 'en',
      servings: 2,
      preparation: 'as_recorded',
      lines: [{ ingredient_id: 'oats', quantity: { amount: '0.100500', unit: 'kg' } }],
      steps: [],
    }
    const preview = await client.callTool({ name: 'recipe_preview_create', arguments: { recipe } })
    expect(preview.isError).toBe(false)
    expect(preview.structuredContent).toMatchObject({
      saved: false,
      confirmation_required: true,
      content: { recipe: { lines: [{ grams: '100.500000' }] } },
    })
    const previewId = z
      .object({ preview_id: z.string() })
      .parse(preview.structuredContent).preview_id
    const input = {
      preview_id: previewId,
      idempotency_key: `nextcloud-mcp-${Date.now()}`,
      confirmed: true,
    }
    const created = await client.callTool({ name: 'recipe_create', arguments: input })
    expect(created.isError).toBe(false)
    const id = z.object({ record: z.object({ id: z.string() }) }).parse(created.structuredContent)
      .record.id
    await client.close()
    client = await connect(url)
    const retry = await client.callTool({ name: 'recipe_create', arguments: input })
    expect(retry.structuredContent).toMatchObject({ replayed: true, record: { id } })
    await page.getByRole('button', { name: 'Reload saved version', exact: true }).click()
    await expect(page.locator('.record-list a')).toHaveCount(1)
    await expect(page.locator('.record-list')).toContainText('Created by admin')
    await expect(page.locator('.mealshop')).not.toContainText(input.idempotency_key)
    await audit(page, 'mcp-owner')
    await page.getByRole('link', { name: /MCP acceptance recipe/ }).click()
    await expect(page).toHaveURL(new RegExp(`/recipes/${id}\\?revision=1&assistant=1$`))
    await expect(
      page.getByRole('heading', { name: 'MCP acceptance recipe', exact: true }),
    ).toBeVisible()
    await page.getByRole('link', { name: 'Assistant tools', exact: true }).click()
    await page.getByRole('link', { name: 'Recipes', exact: true }).click()
    await page.getByRole('link', { name: /MCP acceptance recipe/ }).click()
    await page.getByRole('link', { name: 'Edit recipe', exact: true }).click()
    await page.getByRole('checkbox', { name: 'Recipe from another source', exact: true }).focus()
    await page.keyboard.press('Space')
    await page
      .getByLabel('Original recipe URL', { exact: true })
      .fill('https://example.org/mcp-original')
    await page.getByRole('button', { name: 'Save changes', exact: true }).click()
    await expect(
      page.getByRole('heading', { name: 'MCP acceptance recipe', exact: true }),
    ).toBeVisible()
    const restricted = await client.callTool({ name: 'recipe_get', arguments: { id, revision: 1 } })
    expect(restricted.structuredContent).toEqual({
      id,
      revision: 1,
      content: { kind: 'source_link', source_url: 'https://example.org/mcp-original' },
    })
    await page.getByRole('link', { name: 'Household', exact: true }).click()
    await page
      .getByRole('combobox', { name: 'Person to invite', exact: true })
      .fill(process.env.MEALSHOP_READER_USER ?? '')
    await page
      .getByRole('option')
      .filter({ hasText: process.env.MEALSHOP_READER_USER ?? '' })
      .click()
    await page.getByRole('combobox', { name: 'Role', exact: true }).selectOption('reader')
    await page.getByRole('button', { name: 'Invite a person', exact: true }).click()
    await expect(
      page.getByRole('heading', { name: 'Pending invitations', exact: true }),
    ).toBeVisible()
    const readerContext = await browser.newContext()
    try {
      const reader = await readerContext.newPage()
      reader.on('pageerror', (e) => errors.push(e.message))
      await login(reader, true)
      await reader
        .locator('.inline-row')
        .filter({ hasText: home.name })
        .getByRole('button', { name: 'Accept invitation', exact: true })
        .click()
      const joined = reader.locator(`.record-list a[href$="/h/${home.id}/recipes"]`)
      await expect(joined).toBeVisible()
      await joined.click()
      await reader.getByRole('link', { name: 'Assistant tools', exact: true }).click()
      await expect(
        reader.getByRole('button', { name: 'Save assistant permissions', exact: true }),
      ).toHaveCount(0)
      await page.goto(`${embedded}/h/${home.id}/assistants`)
      await access(page, process.env.MEALSHOP_READER_USER ?? '').selectOption('read')
      await save(page)
      await reader.getByRole('button', { name: 'Reload saved version', exact: true }).click()
      await expect(
        reader.getByText('Your assistant access: Read and preview', { exact: true }),
      ).toBeVisible()
      const readClient = await connect(url, true)
      try {
        expect((await readClient.listTools()).tools.map((tool) => tool.name)).not.toContain(
          'recipe_create',
        )
        expect(
          (await readClient.callTool({ name: 'recipe_create', arguments: input })).isError,
        ).toBe(true)
        expect(
          (await readClient.callTool({ name: 'recipe_get', arguments: { id } })).structuredContent,
        ).toMatchObject({ content: { kind: 'source_link' } })
      } finally {
        await readClient.close()
      }
      await audit(reader, 'mcp-reader')
    } finally {
      await readerContext.close()
    }
    await access(page).selectOption('read')
    await save(page)
    expect((await client.callTool({ name: 'recipe_create', arguments: input })).isError).toBe(true)
    await access(page).selectOption('disabled')
    await save(page)
    await expect(client.listTools()).rejects.toThrow()
    expect(errors).toEqual([])
  } finally {
    await client.close()
  }
})

test('assistant permission conflict preserves drafts and navigation requires explicit discard', async ({
  page,
  context,
}) => {
  await setup(page)
  await access(page).selectOption('read')
  const concurrent = await context.newPage()
  await concurrent.goto(page.url())
  await access(concurrent).selectOption('create')
  await save(concurrent)
  await concurrent.close()
  await page.getByRole('button', { name: 'Save assistant permissions', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Your draft is retained')
  await expect(access(page)).toHaveValue('read')
  page.once('dialog', (dialog) => void dialog.dismiss())
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await expect(page).toHaveURL(/\/assistants$/)
  page.once('dialog', (dialog) => void dialog.accept())
  await page.getByRole('button', { name: 'Reload saved version', exact: true }).click()
  await expect(access(page)).toHaveValue('create')
  await expect(page.getByRole('alert')).toHaveCount(0)
  await access(page).selectOption('disabled')
  page.once('dialog', (dialog) => void dialog.accept())
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
})
