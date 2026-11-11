import { expect, type Page } from '@playwright/test'
import { appendFileSync, readFileSync, mkdirSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname } from 'node:path'
const ledger = fileURLToPath(
  new URL('../../../var/nextcloud/browser-fixtures.jsonl', import.meta.url),
)
export async function recordFixture(page: Page, source: string) {
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
  const id = new URL(page.url()).pathname.match(/\/h\/(home-\d+)\//)?.[1]
  expect(id).toMatch(/^home-\d+$/)
  expect(id).not.toBe('home-1')
  mkdirSync(dirname(ledger), { recursive: true, mode: 0o700 })
  appendFileSync(ledger, JSON.stringify({ id, source }) + '\n', { mode: 0o600 })
}
export async function openFixture(page: Page, source: string) {
  await expect(page.locator('.record-list')).toBeVisible({ timeout: 30000 })
  const entries: unknown[] = readFileSync(ledger, 'utf8')
    .trim()
    .split('\n')
    .filter(Boolean)
    .map((line) => JSON.parse(line) as unknown)
  for (const entry of entries.reverse()) {
    if (
      typeof entry !== 'object' ||
      entry === null ||
      !('source' in entry) ||
      entry.source !== source ||
      !('id' in entry) ||
      typeof entry.id !== 'string' ||
      !/^home-\d+$/.test(entry.id) ||
      entry.id === 'home-1'
    )
      continue
    const link = page.locator(`.record-list a[href$="/h/${entry.id}/recipes"]`)
    if (await link.count()) {
      await link.click()
      return
    }
  }
  throw new Error(
    `No recorded ${source} household exists. Run its setup test first; customer households are never test fixtures.`,
  )
}
