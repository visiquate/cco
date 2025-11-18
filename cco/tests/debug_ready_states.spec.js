const { test, expect } = require('@playwright/test');
const { spawn } = require('child_process');

let server;
const SERVER_PORT = 3001;
const SERVER_URL = `http://127.0.0.1:${SERVER_PORT}`;

test.describe('Debug Ready States', () => {
  test.beforeAll(async () => {
    console.log('Starting CCO server on port', SERVER_PORT);
    server = spawn('/Users/brent/.cargo/bin/cco', ['run', '--debug', '--port', SERVER_PORT.toString()], {
      env: { ...process.env, NO_BROWSER: '1' },
      stdio: 'pipe'
    });

    // Wait longer for server to start
    await new Promise(resolve => setTimeout(resolve, 3000));

    let serverReady = false;
    for (let i = 0; i < 15; i++) {
      try {
        const response = await fetch(`http://127.0.0.1:${SERVER_PORT}/health`);
        if (response.ok) {
          serverReady = true;
          console.log('✓ Server health check passed');
          break;
        }
      } catch (e) {
        console.log(`Health check attempt ${i+1} failed, retrying...`);
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }

    if (!serverReady) {
      console.warn('⚠ Server did not pass health check, but continuing...');
    }
  });

  test.afterAll(async () => {
    if (server) {
      server.kill('SIGINT');
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  });

  test('Debug Ready States', async ({ page }) => {
    page.on('console', msg => console.log('[PAGE]', msg.text()));

    await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded', timeout: 15000 });
    
    console.log('=== Page loaded, checking ready states ===');

    // First check what's in window
    const windowKeys = await page.evaluate(() => Object.keys(window).slice(0, 20));
    console.log('Window keys sample:', windowKeys);

    const hasState = await page.evaluate(() => typeof window.state !== 'undefined');
    console.log('window.state exists:', hasState);

    for (let i = 0; i < 20; i++) {
      const globalState = await page.evaluate(() => {
        console.log('Checking state, typeof window.state:', typeof window.state);
        return {
          stateExists: typeof window.state !== 'undefined',
          stateValue: window.state,
          globalState: typeof globalState !== 'undefined' ? true : false,
          stateType: typeof window.state
        };
      });

      console.log(`[${i}] Global state check:`, globalState);

      if (globalState.stateExists && globalState.stateValue?.readyStates) {
        const states = globalState.stateValue.readyStates;
        console.log(`[${i}] Ready states:`, states);

        if (states?.fully) {
          console.log('✓ App is fully ready!');
          break;
        }
      }

      await page.waitForTimeout(1000);
    }

    const finalState = await page.evaluate(() => window.state);
    console.log('Final window.state:', finalState);
  });
});
