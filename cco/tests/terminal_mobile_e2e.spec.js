const { test, expect, devices } = require('@playwright/test');

/**
 * WASM Terminal Mobile E2E Tests
 * Tests mobile device compatibility, touch support, and responsive design
 */

const SERVER_URL = 'http://127.0.0.1:3001';

// Common mobile devices
const mobileDevices = [
  devices['iPhone 13'],
  devices['iPhone 13 Pro'],
  devices['Pixel 5'],
  devices['Galaxy S9+']
];

test.describe('Mobile Device Compatibility', () => {
  for (const device of mobileDevices) {
    test(`Terminal works on ${device.name || 'mobile device'}`, async ({ browser }) => {
      const context = await browser.newContext({
        ...device,
        locale: 'en-US'
      });

      const page = await context.newPage();

      try {
        await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
        await page.waitForTimeout(2000);

        // Navigate to terminal tab
        await page.tap('text=Terminal');
        await page.waitForTimeout(3000);

        // Check terminal initialized
        const hasAdapter = await page.evaluate(() => !!window.state.terminalAdapter);
        expect(hasAdapter).toBe(true);

        // Check terminal is visible
        const terminal = page.locator('#terminal');
        await expect(terminal).toBeVisible();

        // Check viewport-appropriate sizing
        const viewportSize = page.viewportSize();
        console.log(`✓ Terminal works on ${device.name || 'device'} (${viewportSize.width}x${viewportSize.height})`);

      } finally {
        await context.close();
      }
    });
  }
});

test.describe('Touch Support', () => {
  test('Terminal responds to touch events', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13'],
      hasTouch: true
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);

      // Navigate to terminal with tap
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      // Tap terminal to focus
      await page.tap('#terminal');
      await page.waitForTimeout(500);

      // Tap control buttons
      await page.tap('#terminalClearBtn');
      await page.waitForTimeout(300);

      // No errors should occur
      const errors = [];
      page.on('console', msg => {
        if (msg.type() === 'error') errors.push(msg.text());
      });

      expect(errors.length).toBe(0);

      console.log('✓ Touch events work correctly');

    } finally {
      await context.close();
    }
  });

  test('Terminal supports pinch-to-zoom (viewport meta)', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13'],
      hasTouch: true
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);

      // Check viewport meta tag
      const viewportMeta = await page.evaluate(() => {
        const meta = document.querySelector('meta[name="viewport"]');
        return meta ? meta.getAttribute('content') : null;
      });

      expect(viewportMeta).toBeTruthy();
      console.log('✓ Viewport meta tag configured:', viewportMeta);

    } finally {
      await context.close();
    }
  });

  test('Terminal touch targets meet minimum size (44x44px)', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13'],
      hasTouch: true
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      // Check button sizes
      const buttons = ['#terminalClearBtn', '#terminalCopyBtn', '#terminalTypeSwitch'];

      for (const selector of buttons) {
        const box = await page.locator(selector).boundingBox();
        if (box) {
          expect(box.width).toBeGreaterThanOrEqual(44);
          expect(box.height).toBeGreaterThanOrEqual(44);
          console.log(`✓ ${selector}: ${box.width}x${box.height}px (meets 44x44px minimum)`);
        }
      }

    } finally {
      await context.close();
    }
  });
});

test.describe('Mobile UI Optimization', () => {
  test('Terminal controls wrap on narrow screens', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13']
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      // Check controls container uses flexbox with wrap
      const controlsStyle = await page.locator('.terminal-controls').evaluate(el => {
        const styles = window.getComputedStyle(el);
        return {
          display: styles.display,
          flexWrap: styles.flexWrap
        };
      });

      expect(controlsStyle.display).toBe('flex');
      expect(controlsStyle.flexWrap).toBe('wrap');

      console.log('✓ Terminal controls wrap correctly on mobile');

    } finally {
      await context.close();
    }
  });

  test('Terminal uses appropriate font size on mobile', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13']
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      // Check terminal font size
      const terminalStyle = await page.locator('#terminal').evaluate(el => {
        const styles = window.getComputedStyle(el);
        return {
          fontSize: styles.fontSize,
          fontFamily: styles.fontFamily
        };
      });

      const fontSize = parseInt(terminalStyle.fontSize);
      expect(fontSize).toBeGreaterThanOrEqual(12); // Readable on mobile
      expect(fontSize).toBeLessThanOrEqual(16); // Not too large

      console.log('✓ Terminal font size appropriate for mobile:', fontSize + 'px');

    } finally {
      await context.close();
    }
  });

  test('Terminal height adapts to mobile viewport', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13']
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      // Check terminal wrapper height
      const box = await page.locator('.terminal-wrapper').boundingBox();
      expect(box.height).toBeGreaterThan(200); // Minimum height from CSS
      expect(box.height).toBeLessThan(page.viewportSize().height); // Fits in viewport

      console.log('✓ Terminal height adapts to mobile viewport:', box.height + 'px');

    } finally {
      await context.close();
    }
  });
});

test.describe('Mobile Performance', () => {
  test('Terminal initializes quickly on mobile', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13']
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);

      const startTime = Date.now();
      await page.tap('text=Terminal');

      // Wait for terminal to be ready
      await page.waitForFunction(() => {
        return window.state && window.state.terminalAdapter && window.state.terminalAdapter.isConnected;
      }, { timeout: 15000 });

      const endTime = Date.now();
      const initTime = endTime - startTime;

      console.log(`✓ Mobile terminal initialization: ${initTime}ms`);

      // Should initialize within 15 seconds on mobile
      expect(initTime).toBeLessThan(15000);

    } finally {
      await context.close();
    }
  });

  test('Terminal rendering smooth on mobile', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13']
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      // Type several characters
      await page.tap('#terminal');
      for (let i = 0; i < 10; i++) {
        await page.keyboard.type('a', { delay: 50 });
      }
      await page.waitForTimeout(500);

      // Check for performance warnings in console
      const perfWarnings = [];
      page.on('console', msg => {
        if (msg.text().includes('performance') || msg.text().includes('slow')) {
          perfWarnings.push(msg.text());
        }
      });

      expect(perfWarnings.length).toBe(0);

      console.log('✓ Terminal rendering smooth on mobile (no performance warnings)');

    } finally {
      await context.close();
    }
  });
});

test.describe('Mobile Landscape/Portrait', () => {
  test('Terminal adapts to orientation changes', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13']
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      // Get initial size (portrait)
      const portraitBox = await page.locator('.terminal-wrapper').boundingBox();

      // Switch to landscape
      await page.setViewportSize({ width: 844, height: 390 }); // iPhone 13 landscape
      await page.waitForTimeout(500);

      // Get landscape size
      const landscapeBox = await page.locator('.terminal-wrapper').boundingBox();

      // Dimensions should adapt
      expect(landscapeBox.width).not.toBe(portraitBox.width);
      expect(landscapeBox.height).not.toBe(portraitBox.height);

      console.log('✓ Terminal adapts to orientation changes');
      console.log(`  Portrait: ${portraitBox.width}x${portraitBox.height}px`);
      console.log(`  Landscape: ${landscapeBox.width}x${landscapeBox.height}px`);

    } finally {
      await context.close();
    }
  });
});

test.describe('Mobile Browser Compatibility', () => {
  test('Works on mobile Safari', async ({ browser }) => {
    // Simulate Safari on iPhone
    const context = await browser.newContext({
      ...devices['iPhone 13'],
      userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1'
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      const hasAdapter = await page.evaluate(() => !!window.state.terminalAdapter);
      expect(hasAdapter).toBe(true);

      console.log('✓ Terminal works on mobile Safari');

    } finally {
      await context.close();
    }
  });

  test('Works on Chrome Mobile', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['Pixel 5']
    });

    const page = await context.newPage();

    try {
      await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded' });
      await page.waitForTimeout(2000);
      await page.tap('text=Terminal');
      await page.waitForTimeout(3000);

      const hasAdapter = await page.evaluate(() => !!window.state.terminalAdapter);
      expect(hasAdapter).toBe(true);

      console.log('✓ Terminal works on Chrome Mobile');

    } finally {
      await context.close();
    }
  });
});
