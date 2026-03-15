// @ts-check
import { test, expect } from '@playwright/test';

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Wait up to `ms` for the tick counter to advance past `startTick`. */
async function waitForTickAdvance(page, startTick, ms = 8_000) {
  await page.waitForFunction(
    (start) => {
      const el = document.getElementById('val-tick');
      return el && Number(el.textContent?.replace(/,/g, '')) > start;
    },
    startTick,
    { timeout: ms },
  );
}

// ─── Page load ───────────────────────────────────────────────────────────────

test.describe('ABM mode — page load', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('page title is "ABM Simulator"', async ({ page }) => {
    await expect(page).toHaveTitle(/ABM Simulator/);
  });

  test('#app container is rendered', async ({ page }) => {
    await expect(page.locator('#app')).toBeVisible();
  });

  test('mode toggle bar is visible with both mode buttons', async ({ page }) => {
    await expect(page.locator('.mode-bar')).toBeVisible();
    await expect(page.locator('#mode-btn-abm')).toBeVisible();
    await expect(page.locator('#mode-btn-hospital')).toBeVisible();
  });

  test('ABM mode button is active by default', async ({ page }) => {
    await expect(page.locator('#mode-btn-abm')).toHaveClass(/active/);
    await expect(page.locator('#mode-btn-hospital')).not.toHaveClass(/active/);
  });
});

// ─── ABM canvas ──────────────────────────────────────────────────────────────

test.describe('ABM mode — canvas rendering', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('ABM canvas element is visible', async ({ page }) => {
    await expect(page.locator('#sim-canvas')).toBeVisible();
  });

  test('hospital canvas is hidden in ABM mode', async ({ page }) => {
    await expect(page.locator('#hospital-canvas')).toBeHidden();
  });

  test('ABM canvas has non-zero dimensions', async ({ page }) => {
    const box = await page.locator('#sim-canvas').boundingBox();
    expect(box).not.toBeNull();
    expect(box.width).toBeGreaterThan(100);
    expect(box.height).toBeGreaterThan(100);
  });
});

// ─── Stats bar ───────────────────────────────────────────────────────────────

test.describe('ABM mode — stats bar', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('ABM stats bar is visible', async ({ page }) => {
    await expect(page.locator('#stats-bar-abm')).toBeVisible();
  });

  test('hospital stats bar is hidden in ABM mode', async ({ page }) => {
    await expect(page.locator('#stats-bar-hospital')).toBeHidden();
  });

  test('tick counter element exists', async ({ page }) => {
    await expect(page.locator('#val-tick')).toBeAttached();
  });

  test('prey counter element exists', async ({ page }) => {
    await expect(page.locator('#val-prey')).toBeAttached();
  });
});

// ─── Playback controls ───────────────────────────────────────────────────────

test.describe('ABM mode — playback controls', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM engine to be ready (sidebar visible = DOM built)
    await page.waitForSelector('#btn-run');
  });

  test('Run, Step, Reset buttons are visible', async ({ page }) => {
    await expect(page.locator('#btn-run')).toBeVisible();
    await expect(page.locator('#btn-step')).toBeVisible();
    await expect(page.locator('#btn-reset')).toBeVisible();
  });

  test('Step button advances the tick counter', async ({ page }) => {
    // Wait for engine ready — a initial step fires automatically
    await page.waitForFunction(() => {
      const el = document.getElementById('val-tick');
      return el && el.textContent !== '';
    }, { timeout: 15_000 });

    const before = await page.locator('#val-tick').textContent();
    await page.locator('#btn-step').click();

    await page.waitForFunction(
      (prev) => {
        const el = document.getElementById('val-tick');
        return el && el.textContent !== prev;
      },
      before,
      { timeout: 5_000 },
    );

    const after = await page.locator('#val-tick').textContent();
    expect(Number(after.replace(/,/g, ''))).toBeGreaterThan(Number(before.replace(/,/g, '')));
  });

  test('Run button changes label to Pause when clicked', async ({ page }) => {
    await page.waitForSelector('#btn-run');
    await page.locator('#btn-run').click();
    await expect(page.locator('#btn-run')).toContainText('Pause');
    // Clean up — pause so subsequent tests are unaffected
    await page.locator('#btn-run').click();
  });

  test('Reset button resets tick counter to 0', async ({ page }) => {
    // Step a few times first
    await page.waitForFunction(() => !!document.getElementById('val-tick')?.textContent, { timeout: 15_000 });
    await page.locator('#btn-step').click();
    await page.locator('#btn-reset').click();
    // After reset the tick should read 0 (or very low)
    await page.waitForFunction(() => {
      const el = document.getElementById('val-tick');
      return el && Number(el.textContent?.replace(/,/g, '')) <= 1;
    }, { timeout: 5_000 });
    const tick = await page.locator('#val-tick').textContent();
    expect(Number(tick.replace(/,/g, ''))).toBeLessThanOrEqual(1);
  });
});

// ─── Control panel sliders ───────────────────────────────────────────────────

test.describe('ABM mode — control panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.sidebar');
  });

  test('sidebar contains parameter sliders', async ({ page }) => {
    const sliders = page.locator('#sidebar-abm input[type=range]');
    await expect(sliders).toHaveCount({ min: 3 }); // speed + at least several param sliders
  });

  test('rule selector buttons are visible', async ({ page }) => {
    await expect(page.locator('.rule-btn').first()).toBeVisible();
  });
});
