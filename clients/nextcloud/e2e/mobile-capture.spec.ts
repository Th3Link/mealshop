import { test, expect } from '@playwright/test'
import { recordFixture } from './fixtures'
const embedded = '/index.php/apps/app_api/embedded/mealshop/main'
test('mobile photo sources remain readable in the native Nextcloud client', async ({ page }) => {
  await page.goto(`${embedded}/households`)
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await page.locator('#mealshop-language').selectOption('en')
  await page
    .getByLabel('Household name', { exact: true })
    .fill(`Browser verification ${Date.now()}`)
  await page.getByRole('button', { name: 'Create household', exact: true }).click()
  await recordFixture(page, 'mobile-capture.spec.ts')
  const home = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  expect(home).toMatch(/^home-\d+$/)
  const proxy = `/index.php/apps/app_api/proxy/mealshop/api/households/${home}/domain/api/household`
  const image = await page.evaluate(() => {
    const canvas = document.createElement('canvas')
    canvas.width = 1200
    canvas.height = 500
    const ctx = canvas.getContext('2d')
    if (!ctx) throw new Error('Synthetic canvas unavailable')
    ctx.fillStyle = 'white'
    ctx.fillRect(0, 0, 1200, 500)
    ctx.fillStyle = 'black'
    ctx.font = '46px sans-serif'
    for (const [i, line] of ['P5 synthetic recipe', '1 cup oats', '0.250001 g water'].entries())
      ctx.fillText(line, 40, 80 + i * 80)
    return canvas.toDataURL('image/png').split(',')[1] ?? ''
  })
  const headers = { 'X-Mealshop-Request': '1' }
  const upload = await page.request.post(`${proxy}/images`, {
    headers: { ...headers, 'content-type': 'image/png' },
    data: Buffer.from(image, 'base64'),
  })
  expect(upload.status()).toBe(201)
  const asset: unknown = await upload.json()
  if (
    typeof asset !== 'object' ||
    asset === null ||
    !('id' in asset) ||
    typeof asset.id !== 'string'
  )
    throw new Error('Synthetic image identity missing')
  const capture = {
    image_id: asset.id,
    rotation: 0,
    text: 'P5 synthetic recipe\n1 cup oats\n0.250001 g water',
    engine: 'manual_transcription',
    score: null,
  }
  const staged = await page.request.post(`${proxy}/imports/preview`, {
    headers,
    data: {
      mode: 'photo',
      capture,
      original: {
        title: 'P5 synthetic photo recipe',
        language: 'en',
        ingredients: ['1 cup oats'],
        steps: [],
      },
    },
  })
  expect(staged.status()).toBe(200)
  const preview: unknown = await staged.json()
  if (
    typeof preview !== 'object' ||
    preview === null ||
    !('preview_id' in preview) ||
    typeof preview.preview_id !== 'string'
  )
    throw new Error('Synthetic preview identity missing')
  const saved = await page.request.post(`${proxy}/imports`, {
    headers,
    data: {
      preview_id: preview.preview_id,
      candidate: 0,
      idempotency_key: `p5-browser-import-${Date.now()}`,
      reviewed: true,
      recipe: {
        title: 'P5 synthetic photo recipe',
        author: '',
        language: 'en',
        servings: 2,
        preparation: 'as_recorded',
        origin: { kind: 'captured', source_image_id: asset.id },
        lines: [{ ingredient_id: 'oats', grams: '10.000001' }],
        steps: [],
      },
    },
  })
  expect(saved.status()).toBe(201)
  const record: unknown = await saved.json()
  if (
    typeof record !== 'object' ||
    record === null ||
    !('id' in record) ||
    typeof record.id !== 'string'
  )
    throw new Error('Synthetic recipe identity missing')
  await page.goto(`${embedded}/h/${home}/recipes/${record.id}`)
  await expect(
    page.getByRole('heading', { name: 'P5 synthetic photo recipe', exact: true }),
  ).toBeVisible()
  await expect(page.locator('.import-evidence')).toContainText('1 cup oats')
  await expect(page.locator('.source-photo')).toBeVisible()
  await expect(page.locator('.source-photo')).toHaveJSProperty('naturalWidth', 1200)
  await expect(
    page
      .getByText('Private photo source. Its content and image cannot be shared.', { exact: true })
      .first(),
  ).toBeVisible()
  await page.setViewportSize({ width: 390, height: 900 })
  await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' })
  await expect
    .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
    .toBeLessThanOrEqual(390)
  await page.screenshot({ path: '../../var/mobile/p5-nextcloud-photo-dark.png', fullPage: true })
})
