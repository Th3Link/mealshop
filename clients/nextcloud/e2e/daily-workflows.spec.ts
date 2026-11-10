import { openFixture } from './fixtures'
import { test, expect, type Dialog } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { crc32 } from 'node:zlib'

test('household defaults, focused search, inline composites, private images, favourites and calendar skips', async ({
  page,
}) => {
  test.setTimeout(180000)
  const errors: string[] = []
  page.on('pageerror', (error) => errors.push(error.message))
  const acceptDialog = (dialog: Dialog) => {
    void dialog.accept()
  }
  page.on('dialog', acceptDialog)
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await openFixture(page, 'authoring-experience.spec.ts')
  await expect(page.getByRole('heading', { name: 'Recipes', exact: true })).toBeVisible()
  const homePath = new URL(page.url()).pathname.replace(/\/recipes$/, '')
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/')
  await expect(page).toHaveURL(new RegExp(`${homePath}/recipes$`))
  const switcher = page.locator('.household-switcher').getByRole('combobox')
  await switcher.click()
  await page.getByRole('listbox').getByRole('option').first().click()
  await expect(page).not.toHaveURL(new RegExp(`${homePath}/recipes$`))
  const switched = new URL(page.url()).pathname
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/')
  await expect(page).toHaveURL(new RegExp(`${switched}$`))
  await page.goto(homePath + '/ingredients')
  const search = page.getByLabel('Search ingredients', { exact: true })
  let query = ''
  for (const letter of 'oats') {
    query += letter
    await search.pressSequentially(letter)
    await expect(page).toHaveURL(new RegExp(`q=${query}$`))
    await expect(search).toBeFocused()
  }
  await expect(page.locator('.record-list')).toContainText('Oats')
  const stamp = Date.now()
  const title = `Workflow recipe ${stamp}`
  const ingredientName = `Workflow mix ${stamp}`
  await page.goto(homePath + '/recipes/new')
  await page.getByLabel('Recipe title', { exact: true }).fill(title)
  await page.getByRole('button', { name: 'Add ingredient', exact: true }).click()
  await page.getByRole('combobox', { name: 'Choose ingredient', exact: true }).fill(ingredientName)
  await page.getByRole('option').filter({ hasText: 'Create ingredient' }).click()
  const dialog = page.getByRole('dialog')
  await dialog.getByLabel('Ingredient name', { exact: true }).fill(ingredientName)
  await dialog.getByLabel('Form or product description', { exact: true }).fill('Dry mixture')
  await dialog.getByText('Tags and composition', { exact: true }).click()
  await dialog.getByLabel('Tags', { exact: true }).fill('test-mixture')
  await dialog.getByRole('combobox', { name: 'Contains ingredients', exact: true }).fill('oats')
  await page.getByRole('option', { name: 'Oats', exact: true }).click()
  await dialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(dialog).toHaveCount(0)
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(title)
  await page.getByLabel('Grams', { exact: true }).fill('200.000000')
  const pixel = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aL1sAAAAASUVORK5CYII=',
    'base64',
  )
  const metadata = Buffer.concat([Buffer.from('fixture\0'), Buffer.alloc(150 * 1024, 97)])
  const chunkType = Buffer.from('tEXt')
  const length = Buffer.alloc(4)
  length.writeUInt32BE(metadata.length)
  const crc = Buffer.alloc(4)
  crc.writeUInt32BE(crc32(Buffer.concat([chunkType, metadata])))
  const png = Buffer.concat([
    pixel.subarray(0, -12),
    length,
    chunkType,
    metadata,
    crc,
    pixel.subarray(-12),
  ])
  const uploaded = page.waitForResponse(
    (response) =>
      response.url().endsWith('/api/household/images') && response.request().method() === 'POST',
  )
  await page.getByText('Optional details', { exact: true }).click()
  await page
    .locator('input[type=file]')
    .setInputFiles({ name: 'test-image.png', mimeType: 'image/png', buffer: png })
  expect((await uploaded).status()).toBe(201)
  await page.getByRole('button', { name: 'Add step', exact: true }).click()
  await page.getByLabel('1. Instruction', { exact: true }).fill('Mix and serve.')
  await page.getByRole('combobox', { name: 'Active work · Unit', exact: true }).selectOption('1')
  await page.getByLabel('Active work', { exact: true }).fill('60')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible()
  const recipePath = new URL(page.url()).pathname
  await expect(
    page.locator('.plain-list').getByText(new RegExp(`^200 g · ${ingredientName}`)),
  ).toBeVisible()
  await expect
    .poll(() =>
      page
        .locator('img.recipe-image')
        .evaluate((image) => image instanceof HTMLImageElement && image.naturalWidth > 0),
    )
    .toBe(true)
  await page.getByLabel('Preview servings', { exact: true }).fill('2')
  await page.getByRole('button', { name: 'Preview', exact: true }).click()
  await expect(
    page.locator('.plain-list').getByText(new RegExp(`^400 g · ${ingredientName}`)),
  ).toBeVisible()
  await page.reload()
  await expect
    .poll(() =>
      page
        .locator('img.recipe-image')
        .evaluate((image) => image instanceof HTMLImageElement && image.naturalWidth > 0),
    )
    .toBe(true)
  await page.getByText('Edit recipe', { exact: true }).click()
  await page.getByRole('button', { name: 'Remove image', exact: true }).click()
  page.off('dialog', acceptDialog)
  page.once('dialog', (dialog) => {
    void dialog.dismiss()
  })
  await page.getByRole('link', { name: 'Recipes', exact: true }).click()
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(title)
  await page.locator('.household-switcher').getByRole('combobox').click()
  page.once('dialog', (dialog) => {
    void dialog.dismiss()
  })
  await page.getByRole('listbox').getByRole('option').first().click()
  await expect(page.getByLabel('Recipe title', { exact: true })).toHaveValue(title)
  const defaultCheck = await page.context().newPage()
  await defaultCheck.goto('/index.php/apps/app_api/embedded/mealshop/main/')
  await expect(defaultCheck).toHaveURL(new RegExp(`${homePath}/recipes$`))
  await defaultCheck.close()
  page.on('dialog', acceptDialog)
  await page.getByRole('link', { name: 'Cancel', exact: true }).first().click()
  await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible()
  await page.goto(homePath + '/recipes')
  const thumbnail = page
    .locator('.record-list li')
    .filter({ has: page.getByText(title, { exact: true }) })
    .locator('img.recipe-thumbnail')
  await expect
    .poll(() =>
      thumbnail.evaluate((image) => image instanceof HTMLImageElement && image.naturalWidth > 0),
    )
    .toBe(true)
  await page.goto(recipePath)
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  await page.getByLabel('Search ingredients', { exact: true }).fill(ingredientName)
  const favorite = page.getByRole('button', {
    name: `Household favourite ${ingredientName}`,
    exact: true,
  })
  await favorite.click()
  await expect(favorite).toHaveAttribute('aria-pressed', 'true')
  await page.getByText('Favourites only', { exact: true }).click()
  await expect(page.getByRole('checkbox', { name: 'Favourites only', exact: true })).toBeChecked()
  await page.locator('.record-list').getByRole('link').first().click()
  await expect(page.getByRole('link', { name: title, exact: true })).toBeVisible()
  await expect(
    page.getByRole('heading', { name: 'Contains ingredients', exact: true }),
  ).toBeVisible()
  await page.getByRole('button', { name: 'Archive ingredient', exact: true }).click()
  await expect(page.getByText('Archived', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Restore ingredient', exact: true }).click()
  await expect(page.getByText('Archived', { exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Archive ingredient', exact: true })).toBeEnabled()
  await page.goto(recipePath)
  await expect(page.getByRole('heading', { name: title, exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Household', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Dietary profiles', exact: true })).toBeVisible()
  if ((await page.getByLabel('Name', { exact: true }).count()) === 0) {
    await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
    await page.getByLabel('Name', { exact: true }).fill('Test attendee')
    await page.getByRole('button', { name: 'Save changes', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  }
  await page.getByRole('button', { name: 'Add dietary profile', exact: true }).click()
  await page.getByLabel('Name', { exact: true }).last().fill(`Test skipper ${stamp}`)
  await page.getByText('Notes and excluded tags', { exact: true }).last().click()
  await page
    .getByLabel(/^Hard excluded ingredient tags/)
    .last()
    .fill('test-mixture')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Save changes', exact: true })).toBeDisabled()
  await page.getByRole('link', { name: 'Meal plan', exact: true }).click()
  const today = await page.evaluate(() => {
    const date = new Date()
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
  })
  const slot = page.locator(`.meal-slot[data-date="${today}"][data-meal=lunch]`)
  await slot.getByRole('button').click()
  await page
    .getByRole('combobox', { name: 'Recipes', exact: true })
    .selectOption({ label: `${title} · Revision 1` })
  const attendance = page
    .getByRole('group', { name: 'Attendees', exact: true })
    .getByRole('checkbox')
  for (let index = 0; index < (await attendance.count()); index++) {
    const checkbox = attendance.nth(index)
    if ((await checkbox.isChecked()) !== (index === 0)) await checkbox.press('Space')
  }
  await page.getByLabel('Servings', { exact: true }).fill('1')
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  await page
    .getByText('I reviewed the unknown dietary data and serving/time uncertainty.', { exact: true })
    .click()
  await page.getByRole('button', { name: 'Apply reviewed plan', exact: true }).click()
  await expect(slot).toContainText(title)
  await expect(slot).toContainText('Skipping:')
  await expect(slot).toContainText(`Test skipper ${stamp}`)
  const axe = await new AxeBuilder({ page })
    .include('.mealshop')
    .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
    .analyze()
  expect(axe.violations).toEqual([])
  for (const width of [1440, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 1000 })
    await expect
      .poll(() => page.evaluate(() => document.documentElement.scrollWidth))
      .toBeLessThanOrEqual(width)
    await page.screenshot({
      path: `test-results/meal-calendar-${width}.png`,
      animations: 'disabled',
    })
  }
  await page.setViewportSize({ width: 1440, height: 1000 })
  await slot.getByRole('button').click()
  const skipper=page.getByRole('checkbox',{name:`Test skipper ${stamp}`,exact:true})
  await expect(skipper).not.toBeChecked()
  await skipper.focus()
  await page.keyboard.press('Space')
  await expect(skipper).toBeChecked()
  await page.getByRole('button', { name: 'Review meal', exact: true }).click()
  await expect(
    page.getByRole('button', { name: 'Apply reviewed plan', exact: true }),
  ).toBeDisabled()
  await expect(
    page.getByText('An attendee has a hard dietary exclusion for this recipe.', { exact: true }),
  ).toBeVisible()
  expect(errors).toEqual([])
})

test('ingredient conflicts retain the draft and require explicit version review', async ({
  page,
  context,
}) => {
  page.on('dialog', (dialog) => {
    void dialog.accept()
  })
  await page.goto('/index.php/apps/app_api/embedded/mealshop/main/households')
  await page.locator('#user').fill(process.env.MEALSHOP_TEST_USER ?? '')
  await page.locator('#password').fill(process.env.MEALSHOP_TEST_PASSWORD ?? '')
  await page.getByRole('button', { name: 'Log in', exact: true }).click()
  await openFixture(page, 'authoring-experience.spec.ts')
  await page.getByRole('link', { name: 'Ingredients', exact: true }).click()
  const fixtureName = `Conflict fixture ${Date.now()}`
  await page.getByRole('button', { name: 'Create ingredient', exact: true }).click()
  const createDialog = page.getByRole('dialog')
  await createDialog.getByLabel('Ingredient name', { exact: true }).fill(fixtureName)
  await createDialog.getByLabel('Form or product description', { exact: true }).fill('As declared')
  await createDialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(createDialog).toHaveCount(0)
  await expect(page.getByRole('heading', { name: fixtureName, exact: true })).toBeVisible()
  const ingredientPath = new URL(page.url()).pathname
  await page.getByRole('button', { name: 'Edit ingredient', exact: true }).click()
  await page.getByRole('dialog').getByText('Tags and composition', { exact: true }).click()
  await page.getByRole('dialog').getByLabel('Tags', { exact: true }).fill('unsaved-declaration')
  const other = await context.newPage()
  await other.goto(ingredientPath)
  await other.getByRole('button', { name: 'Edit ingredient', exact: true }).click()
  await other.getByRole('dialog').getByText('Tags and composition', { exact: true }).click()
  await other.getByRole('dialog').getByLabel('Tags', { exact: true }).fill('concurrent-declaration')
  await other.getByRole('dialog').getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(other.getByRole('dialog')).toHaveCount(0)
  const dialog = page.getByRole('dialog')
  await dialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(dialog.getByRole('alert')).toBeVisible()
  await expect(dialog.getByLabel('Tags', { exact: true })).toHaveValue('unsaved-declaration')
  await dialog.getByRole('button', { name: 'Reload saved version', exact: true }).click()
  await expect(dialog.getByLabel('Tags', { exact: true })).toHaveValue('concurrent-declaration')
  await dialog.getByLabel('Tags', { exact: true }).fill('reviewed-declaration')
  await dialog.getByRole('button', { name: 'Save changes', exact: true }).click()
  await expect(dialog).toHaveCount(0)
  await expect(page.getByText('Tags: reviewed-declaration', { exact: true })).toBeVisible()
  await other.close()
})
