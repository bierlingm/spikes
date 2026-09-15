import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for the Spikes widget test suite.
 * Chromium-only to keep the <60s time budget.
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'list',
  use: {
    baseURL: 'http://localhost:4717',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        // SPIKES_CHROMIUM=/usr/bin/chromium runs the suite on a system browser
        // when the Playwright download is unavailable (offline / slow CI).
        ...(process.env.SPIKES_CHROMIUM ? { launchOptions: { executablePath: process.env.SPIKES_CHROMIUM } } : {}),
      },
    },
  ],
  webServer: {
    command: 'node server/index.js',
    url: 'http://localhost:4717',
    reuseExistingServer: !process.env.CI,
    timeout: 10000,
  },
});
