const { test, expect } = require('@playwright/test');
const { spawn } = require('child_process');

let server;
const SERVER_PORT = 3002;  // Use different port to avoid conflicts
const SERVER_URL = `http://127.0.0.1:${SERVER_PORT}`;

test.describe('Investigate Window Object', () => {
  test.beforeAll(async () => {
    console.log('\n===== SERVER STARTUP BEGIN =====');
    server = spawn('/Users/brent/.cargo/bin/cco', ['run', '--debug', '--port', SERVER_PORT.toString()], {
      env: { ...process.env, NO_BROWSER: '1' },
      stdio: 'pipe'
    });

    await new Promise(resolve => setTimeout(resolve, 5000));

    let serverReady = false;
    for (let i = 0; i < 10; i++) {
      try {
        const response = await fetch(`http://127.0.0.1:${SERVER_PORT}/health`);
        if (response.ok) {
          serverReady = true;
          console.log('✓ Server health check passed');
          break;
        }
      } catch (e) {
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    console.log('===== SERVER STARTUP END =====\n');
  });

  test.afterAll(async () => {
    if (server) {
      server.kill('SIGINT');
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  });

  test('Investigate window access in Playwright', async ({ page }) => {
    // Capture console messages
    const consoleLogs = [];
    page.on('console', msg => {
      const text = msg.text();
      consoleLogs.push(text);
      console.log('[BROWSER-CONSOLE]', text);
    });

    console.log('\n===== NAVIGATION BEGIN =====');
    const response = await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded', timeout: 20000 });
    console.log('Response status:', response?.status());
    console.log('===== NAVIGATION END =====\n');

    // Wait for scripts to load
    await page.waitForTimeout(3000);

    console.log('\n===== WINDOW INVESTIGATION BEGIN =====');

    // Method 1: Direct window access
    console.log('\nMethod 1: Direct window evaluation');
    const result1 = await page.evaluate(() => {
      console.log('[EVAL] Checking window...');
      console.log('[EVAL] typeof window:', typeof window);
      console.log('[EVAL] typeof window.state:', typeof window.state);
      return { state: window.state, keys: Object.keys(window).length };
    });
    console.log('Result 1:', result1);

    // Method 2: Check for specific properties
    console.log('\nMethod 2: Checking specific window properties');
    const result2 = await page.evaluate(() => {
      return {
        hasState: 'state' in window,
        stateValue: window.state,
        stateType: typeof window.state,
        stateKeys: window.state ? Object.keys(window.state) : []
      };
    });
    console.log('Result 2:', result2);

    // Method 3: Get HTML content to see if script loaded
    console.log('\nMethod 3: Checking if dashboard.js script exists in DOM');
    const scripts = await page.evaluate(() => {
      const scriptTags = document.querySelectorAll('script');
      const dashboardScript = Array.from(scriptTags).find(s => s.src && s.src.includes('dashboard.js'));
      return {
        totalScripts: scriptTags.length,
        hasDashboardScript: !!dashboardScript,
        scripts: Array.from(scriptTags).map(s => s.src || s.textContent.substring(0, 50))
      };
    });
    console.log('Scripts:', scripts);

    console.log('\n===== WINDOW INVESTIGATION END =====\n');
    
    // Report console logs
    console.log('Console logs captured:');
    consoleLogs.forEach(log => console.log('  -', log));
  });
});
