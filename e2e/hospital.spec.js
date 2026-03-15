// @ts-check
import { test, expect } from '@playwright/test';

// ─── Helpers ──────────────────────────────────────────────────────────────────

/**
 * Navigate to the app, wait for DOM, then click the hospital mode button
 * and wait for the hospital layout to become visible.
 */
async function switchToHospital(page) {
  await page.goto('/');
  await page.waitForSelector('#mode-btn-hospital');
  await page.locator('#mode-btn-hospital').click();

  // Wait for hospital main row to become visible
  await page.waitForSelector('#hospital-main', { state: 'visible', timeout: 5_000 });
}

/**
 * Wait for the hospital engine to initialise (census element becomes non-empty)
 * and optionally fire one step.
 */
async function waitForHospitalEngine(page, timeoutMs = 20_000) {
  await page.waitForFunction(() => {
    const el = document.getElementById('hval-tick');
    return el && el.textContent !== '' && el.textContent !== '0';
  }, { timeout: timeoutMs });
}

// ─── Mode switching ──────────────────────────────────────────────────────────

test.describe('Hospital mode — mode toggle', () => {
  test('clicking hospital button activates hospital mode', async ({ page }) => {
    await switchToHospital(page);
    await expect(page.locator('#mode-btn-hospital')).toHaveClass(/active/);
    await expect(page.locator('#mode-btn-abm')).not.toHaveClass(/active/);
  });

  test('ABM main row is hidden in hospital mode', async ({ page }) => {
    await switchToHospital(page);
    await expect(page.locator('#abm-main')).toBeHidden();
  });

  test('hospital main row is visible', async ({ page }) => {
    await switchToHospital(page);
    await expect(page.locator('#hospital-main')).toBeVisible();
  });

  test('switching back to ABM mode restores ABM layout', async ({ page }) => {
    await switchToHospital(page);
    await page.locator('#mode-btn-abm').click();
    await expect(page.locator('#abm-main')).toBeVisible();
    await expect(page.locator('#hospital-main')).toBeHidden();
  });
});

// ─── Hospital stats bar ──────────────────────────────────────────────────────

test.describe('Hospital mode — stats bar', () => {
  test.beforeEach(async ({ page }) => {
    await switchToHospital(page);
  });

  test('hospital stats bar is visible', async ({ page }) => {
    await expect(page.locator('#stats-bar-hospital')).toBeVisible();
  });

  test('ABM stats bar is hidden', async ({ page }) => {
    await expect(page.locator('#stats-bar-abm')).toBeHidden();
  });

  test('hospital tick counter element exists', async ({ page }) => {
    await expect(page.locator('#hval-tick')).toBeAttached();
  });

  test('census element exists', async ({ page }) => {
    await expect(page.locator('#hval-census')).toBeAttached();
  });

  test('HAI cumulative counter element exists', async ({ page }) => {
    await expect(page.locator('#hval-hai')).toBeAttached();
  });

  test('SPRT element exists', async ({ page }) => {
    await expect(page.locator('#hval-sprt')).toBeAttached();
  });
});

// ─── Hospital canvas ─────────────────────────────────────────────────────────

test.describe('Hospital mode — canvas rendering', () => {
  test.beforeEach(async ({ page }) => {
    await switchToHospital(page);
  });

  test('hospital canvas is visible', async ({ page }) => {
    await expect(page.locator('#hospital-canvas')).toBeVisible();
  });

  test('ABM canvas is hidden', async ({ page }) => {
    await expect(page.locator('#sim-canvas')).toBeHidden();
  });

  test('hospital canvas has non-zero dimensions', async ({ page }) => {
    const box = await page.locator('#hospital-canvas').boundingBox();
    expect(box).not.toBeNull();
    expect(box.width).toBeGreaterThan(100);
    expect(box.height).toBeGreaterThan(100);
  });

  test('hospital canvas has been painted (not all black zeros)', async ({ page }) => {
    // Wait a moment for WASM to init and at least one render to fire
    await page.waitForTimeout(1_500);

    const hasContent = await page.evaluate(() => {
      const canvas = document.getElementById('hospital-canvas');
      if (!canvas) return false;
      const ctx = canvas.getContext('2d');
      if (!ctx) return false;
      // Sample pixels in a 10×10 grid; expect at least one non-zero pixel
      const { width, height } = canvas;
      const step = Math.floor(Math.min(width, height) / 10);
      for (let x = step; x < width; x += step) {
        for (let y = step; y < height; y += step) {
          const d = ctx.getImageData(x, y, 1, 1).data;
          // Any non-transparent pixel counts as "painted"
          if (d[3] > 0) return true;
        }
      }
      return false;
    });

    expect(hasContent).toBe(true);
  });
});

// ─── Hospital playback controls ───────────────────────────────────────────────

test.describe('Hospital mode — playback controls', () => {
  test.beforeEach(async ({ page }) => {
    await switchToHospital(page);
    await page.waitForSelector('#hbtn-run');
  });

  test('Run, Step, Reset buttons are visible', async ({ page }) => {
    await expect(page.locator('#hbtn-run')).toBeVisible();
    await expect(page.locator('#hbtn-step')).toBeVisible();
    await expect(page.locator('#hbtn-reset')).toBeVisible();
  });

  test('Run button toggles to Pause when clicked', async ({ page }) => {
    await page.locator('#hbtn-run').click();
    await expect(page.locator('#hbtn-run')).toContainText('Pause');
    await page.locator('#hbtn-run').click(); // stop
  });

  test('Step button advances hospital tick counter', async ({ page }) => {
    // Wait for engine ready
    await page.waitForFunction(() => {
      const el = document.getElementById('hval-tick');
      return el !== null;
    }, { timeout: 20_000 });

    const before = await page.locator('#hval-tick').textContent();
    await page.locator('#hbtn-step').click();

    // Wait up to 10 s for the tick to change
    await page.waitForFunction(
      (prev) => {
        const el = document.getElementById('hval-tick');
        return el && el.textContent !== prev;
      },
      before,
      { timeout: 10_000 },
    );

    const after = await page.locator('#hval-tick').textContent();
    expect(Number(after)).toBeGreaterThanOrEqual(Number(before));
  });
});

// ─── Hospital charts ─────────────────────────────────────────────────────────

test.describe('Hospital mode — D3 charts', () => {
  test.beforeEach(async ({ page }) => {
    await switchToHospital(page);
    // Give the engine and charts time to initialise
    await page.waitForTimeout(1_000);
  });

  test('hospital chart panel is visible', async ({ page }) => {
    await expect(page.locator('#chart-panel-hospital')).toBeVisible();
  });

  test('four sub-chart divs are present', async ({ page }) => {
    const subCharts = page.locator('.hospital-sub-chart');
    await expect(subCharts).toHaveCount(4);
  });

  test('each sub-chart contains an SVG element', async ({ page }) => {
    const svgs = page.locator('.hospital-sub-chart svg');
    await expect(svgs).toHaveCount(4);
  });
});

// ─── Hospital control panel params ───────────────────────────────────────────

test.describe('Hospital mode — control panel', () => {
  test.beforeEach(async ({ page }) => {
    await switchToHospital(page);
    await page.waitForSelector('#sidebar-hospital .param-row');
  });

  test('hospital sidebar has parameter sliders', async ({ page }) => {
    const sliders = page.locator('#sidebar-hospital input[type=range]');
    await expect(sliders).toHaveCount({ min: 3 });
  });

  test('HH Audit button is visible', async ({ page }) => {
    await expect(page.locator('#hbtn-audit')).toBeVisible();
  });

  test('HH Audit button click does not throw an error', async ({ page }) => {
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    await page.locator('#hbtn-audit').click();
    await page.waitForTimeout(200);
    expect(errors).toHaveLength(0);
  });
});
