/**
 * Terminal State Debug Test
 * Purpose: Debug why window.state is undefined when initTerminal is defined
 */

const { test, expect } = require('@playwright/test');
const { spawn } = require('child_process');

let server;
const SERVER_PORT = 3001;
const SERVER_URL = `http://127.0.0.1:${SERVER_PORT}`;

test.describe('Window State Debug', () => {
  test.beforeAll(async () => {
    server = spawn('/Users/brent/.cargo/bin/cco', ['run', '--debug', '--port', SERVER_PORT.toString()], {
      env: { ...process.env, NO_BROWSER: '1' },
      stdio: 'pipe'
    });

    let serverReady = false;
    for (let i = 0; i < 10; i++) {
      try {
        const response = await fetch(`http://127.0.0.1:${SERVER_PORT}/health`);
        if (response.ok) {
          serverReady = true;
          break;
        }
      } catch (e) {
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }
  });

  test.afterAll(async () => {
    if (server) {
      server.kill('SIGINT');
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  });

  test('Immediate State Check on Page Load', async ({ page }) => {
    console.log('\n=== IMMEDIATE STATE CHECK ===\n');

    const allErrors = [];
    const allLogs = [];

    page.on('console', msg => {
      if (msg.type() === 'error') {
        allErrors.push(msg.text());
      }
      allLogs.push(`[${msg.type()}] ${msg.text()}`);
    });

    page.on('pageerror', error => {
      console.log('PAGE ERROR:', error);
      allErrors.push(error.message);
    });

    await page.goto(SERVER_URL, { waitUntil: 'networkidle', timeout: 30000 }).catch(() => {
      // Ignore timeout errors - let's check state anyway
    });

    // Small wait
    await page.waitForTimeout(1000);

    // Check for errors
    console.log('Console errors captured:', allErrors.length);
    if (allErrors.length > 0) {
      console.log('ERRORS:');
      allErrors.forEach(err => console.log(`  - ${err}`));
    }

    // Now check state
    const stateInfo = await page.evaluate(() => {
      return {
        window_has_state: 'state' in window,
        window_state_value: typeof window.state,
        window_state_is_null: window.state === null,
        window_state_is_undefined: window.state === undefined,
        initTerminal_exists: typeof initTerminal,
        initTerminalWebSocket_exists: typeof initTerminalWebSocket,
        document_ready: document.readyState,
        first_10_script_tags: Array.from(document.querySelectorAll('script')).slice(0, 5).map(s => ({
          src: s.src || '(inline)',
          async: s.async,
          defer: s.defer
        }))
      };
    });

    console.log('\nState Information:');
    console.log('  window has "state" property:', stateInfo.window_has_state);
    console.log('  window.state type:', stateInfo.window_state_value);
    console.log('  window.state === null:', stateInfo.window_state_is_null);
    console.log('  window.state === undefined:', stateInfo.window_state_is_undefined);
    console.log('  initTerminal exists:', stateInfo.initTerminal_exists);
    console.log('  initTerminalWebSocket exists:', stateInfo.initTerminalWebSocket_exists);
    console.log('  document.readyState:', stateInfo.document_ready);

    console.log('\nFirst 5 script tags:');
    stateInfo.first_10_script_tags.forEach((tag, i) => {
      console.log(`  ${i + 1}. ${tag.src} (async: ${tag.async}, defer: ${tag.defer})`);
    });

    console.log('\nAll console logs:');
    allLogs.forEach(log => console.log(`  ${log}`));

    expect(stateInfo.window_has_state).toBe(true);
  });

  test('State Availability After DOMContentLoaded', async ({ page }) => {
    console.log('\n=== STATE AFTER DOM CONTENT LOADED ===\n');

    // Wait for DOMContentLoaded
    await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded', timeout: 30000 });

    // Check state right after DOM loaded
    const afterDOM = await page.evaluate(() => {
      return {
        state_type: typeof window.state,
        has_state: !!window.state,
        state_keys: window.state ? Object.keys(window.state) : []
      };
    });

    console.log('After DOMContentLoaded:');
    console.log('  state type:', afterDOM.state_type);
    console.log('  has state:', afterDOM.has_state);
    console.log('  state keys:', afterDOM.state_keys.join(', '));

    expect(afterDOM.state_type).toBe('object');
    expect(afterDOM.has_state).toBe(true);
  });

  test('Dashboard.js Loading Verification', async ({ page }) => {
    console.log('\n=== DASHBOARD.JS LOADING ===\n');

    const allLogs = [];
    page.on('console', msg => {
      allLogs.push(`[${msg.type()}] ${msg.text()}`);
    });

    await page.goto(SERVER_URL, { waitUntil: 'load', timeout: 30000 });

    // Check if dashboard.js is loaded
    const dashboardCheck = await page.evaluate(() => {
      const scripts = Array.from(document.querySelectorAll('script'));
      const dashboardScript = scripts.find(s => s.src && s.src.includes('dashboard.js'));

      return {
        dashboardFound: !!dashboardScript,
        dashboardSrc: dashboardScript?.src,
        totalScripts: scripts.length,
        state_accessible: !!window.state,
        state_type: typeof window.state
      };
    });

    console.log('Dashboard.js Check:');
    console.log('  Found in DOM:', dashboardCheck.dashboardFound);
    console.log('  Source:', dashboardCheck.dashboardSrc);
    console.log('  Total scripts:', dashboardCheck.totalScripts);
    console.log('  window.state accessible:', dashboardCheck.state_accessible);
    console.log('  window.state type:', dashboardCheck.state_type);

    console.log('\nAll console logs:');
    allLogs.forEach(log => {
      if (log.includes('[log]') || log.includes('Initializing')) {
        console.log(`  ${log}`);
      }
    });

    expect(dashboardCheck.dashboardFound).toBe(true);
    expect(dashboardCheck.state_accessible).toBe(true);
  });
});
