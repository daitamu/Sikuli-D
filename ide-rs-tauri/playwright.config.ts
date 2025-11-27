import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for SikuliX IDE E2E tests
 * SikuliX IDE E2Eテスト用Playwright設定
 */
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false, // Tauri apps need sequential tests
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker for Tauri
  reporter: [
    ['html', { open: 'never' }],
    ['list']
  ],

  use: {
    // Base URL for the Tauri app
    baseURL: 'tauri://localhost',

    // Collect trace on failure
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video recording
    video: 'on-first-retry',
  },

  // Global timeout
  timeout: 60000,

  // Expect timeout
  expect: {
    timeout: 10000,
  },

  projects: [
    {
      name: 'tauri',
      use: {
        // Use WebDriver for Tauri
        browserName: 'chromium',
      },
    },
  ],

  // Web server configuration for development
  webServer: {
    command: 'cargo tauri dev',
    url: 'tauri://localhost',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
