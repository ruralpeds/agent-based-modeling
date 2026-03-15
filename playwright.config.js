// @ts-check
import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E configuration.
 *
 * Prerequisites (local):
 *   1. pnpm build:wasm      — build WASM into public/pkg/
 *   2. pnpm build           — build Vite bundle into dist/
 *   3. pnpm test:e2e        — start preview server and run tests
 *
 * In CI the `test-e2e` workflow job runs build steps first, then
 * starts `pnpm preview` via the webServer option below.
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: '**/*.spec.js',

  // Maximum time one test can run (WASM init can be slow on CI).
  timeout: 30_000,

  // Retry flaky tests once on CI; zero retries locally.
  retries: process.env.CI ? 1 : 0,

  // Run tests in parallel by default; sequential within a file.
  fullyParallel: true,
  workers: process.env.CI ? 2 : undefined,

  reporter: process.env.CI
    ? [['github'], ['html', { outputFolder: 'playwright-report', open: 'never' }]]
    : [['list'], ['html', { outputFolder: 'playwright-report', open: 'on-failure' }]],

  use: {
    baseURL: 'http://localhost:4173',

    // Keep all traces on CI so failures are diagnosable.
    trace:      process.env.CI ? 'on-first-retry' : 'off',
    screenshot: process.env.CI ? 'only-on-failure' : 'off',
    video:      'off',

    // Required for SharedArrayBuffer (COOP/COEP are set by Vite preview).
    ignoreHTTPSErrors: true,
  },

  // Auto-start `vite preview` if a server is not already running.
  webServer: {
    command: 'pnpm preview',
    port: 4173,
    // Reuse an already-running server locally; always start fresh on CI.
    reuseExistingServer: !process.env.CI,
    // Allow 30 s for the server to boot (slower on CI runners).
    timeout: 30_000,
  },

  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        // launchOptions.args needed for SharedArrayBuffer in Chromium
        launchOptions: {
          args: ['--enable-features=SharedArrayBuffer'],
        },
      },
    },
  ],
});
