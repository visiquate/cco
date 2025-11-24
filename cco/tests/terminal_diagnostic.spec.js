const { test, expect } = require('@playwright/test');
const { spawn } = require('child_process');

let server;
const SERVER_PORT = 3003;
const SERVER_URL = 'http://127.0.0.1:' + SERVER_PORT;

test.describe.serial('Terminal Diagnostic Tests', () => {
  test.beforeAll(async () => {
    console.log('\nSTARTING SERVER ON PORT', SERVER_PORT, '\n');

    server = spawn('/Users/brent/.cargo/bin/cco', ['run', '--debug', '--port', SERVER_PORT.toString()], {
      env: { ...process.env, NO_BROWSER: '1' },
      stdio: ['ignore', 'pipe', 'pipe']
    });

    server.stdout.on('data', (data) => {
      console.log('[SERVER]', data.toString().trim());
    });

    server.stderr.on('data', (data) => {
      console.error('[SERVER ERR]', data.toString().trim());
    });

    // Wait for server with health checks
    let serverReady = false;
    for (let i = 0; i < 20; i++) {
      try {
        const response = await fetch(SERVER_URL + '/health');
        if (response.ok) {
          const data = await response.json();
          console.log('Server health check PASSED:', data);
          serverReady = true;
          break;
        }
      } catch (e) {
        console.log('Waiting for server... attempt', (i + 1), '/ 20');
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }

    if (!serverReady) {
      throw new Error('Server failed to start after 10 seconds');
    }

    await new Promise(resolve => setTimeout(resolve, 1000));
  });

  test.afterAll(async () => {
    console.log('\nSHUTTING DOWN SERVER\n');
    if (server) {
      server.kill('SIGTERM');
      await new Promise(resolve => setTimeout(resolve, 3000));
    }
  });

  test('Diagnostic: Check page loads', async ({ page }) => {
    console.log('\n--- Test 1: Page Load ---');

    const response = await page.goto(SERVER_URL, {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    console.log('Page load status:', response.status());
    expect(response.ok()).toBe(true);

    const title = await page.title();
    console.log('Page title:', title);

    await page.waitForTimeout(2000);
  });

  test('Diagnostic: Check window.state exists', async ({ page }) => {
    console.log('\n--- Test 2: window.state ---');

    await page.goto(SERVER_URL, {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    await page.waitForTimeout(2000);

    const stateExists = await page.evaluate(() => {
      return typeof window.state !== 'undefined';
    });

    console.log('window.state exists:', stateExists);
    expect(stateExists).toBe(true);
  });

  test('Diagnostic: Check terminal initialization', async ({ page }) => {
    console.log('\n--- Test 3: Terminal Initialization ---');

    page.on('console', msg => {
      const type = msg.type();
      const text = msg.text();
      if (type === 'error') {
        console.error('[BROWSER ERROR]', text);
      } else if (text.includes('[Terminal]') || text.includes('[WebSocket]')) {
        console.log('[BROWSER]', text);
      }
    });

    await page.goto(SERVER_URL, {
      waitUntil: 'networkidle',
      timeout: 30000
    });

    await page.waitForTimeout(2000);

    const terminalTab = await page.locator('text=Terminal').first();
    if (await terminalTab.isVisible()) {
      await terminalTab.click();
      console.log('Clicked terminal tab');
      await page.waitForTimeout(2000);
    }

    const terminalState = await page.evaluate(() => {
      return {
        terminalExists: window.state && window.state.terminal !== null,
        terminalType: window.state && window.state.terminal ? typeof window.state.terminal : 'undefined',
        wsExists: window.state && window.state.ws !== null,
        wsReadyState: window.state && window.state.ws ? window.state.ws.readyState : 'undefined',
        hasWriteMethod: window.state && window.state.terminal ? typeof window.state.terminal.write === 'function' : false
      };
    });

    console.log('Terminal state:', JSON.stringify(terminalState, null, 2));

    expect(terminalState.terminalExists).toBe(true);
  });
});
