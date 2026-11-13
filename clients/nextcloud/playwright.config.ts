import { defineConfig } from '@playwright/test'
export default defineConfig({
  testDir: 'e2e',
  testMatch: '*.spec.ts',
  workers: 1,
  timeout: 90000,
  use: {
    baseURL: process.env.MEALSHOP_TEST_URL,
    headless: true,
    actionTimeout: 10000,
    viewport: { width: 1440, height: 1000 },
    launchOptions: {
      executablePath: process.env.MEALSHOP_CHROMIUM ?? '/usr/bin/chromium',
      args: ['--no-sandbox'],
    },
    screenshot: 'only-on-failure',
    trace: 'off',
  },
})
