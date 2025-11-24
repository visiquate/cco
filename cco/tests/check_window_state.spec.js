const { test, expect } = require('@playwright/test');
const { spawn } = require('child_process');

let server;
const SERVER_PORT = 3001;
const SERVER_URL = `http://127.0.0.1:${SERVER_PORT}`;

test.describe('Check Window State', () => {
  test.beforeAll(async () => {
    console.log('Starting CCO server...');
    server = spawn('/Users/brent/.cargo/bin/cco', ['run', '--debug', '--port', SERVER_PORT.toString()], {
      env: { ...process.env, NO_BROWSER: '1' },
      stdio: 'pipe'
    });

    await new Promise(resolve => setTimeout(resolve, 5000));
  });

  test.afterAll(async () => {
    if (server) {
      server.kill('SIGINT');
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  });

  test('Check window.state access', async ({ page }) => {
    page.on('console', msg => console.log('[CONSOLE]', msg.type(), msg.text()));
    
    const response = await page.goto(SERVER_URL, { waitUntil: 'domcontentloaded', timeout: 20000 });
    console.log('Navigation status:', response?.status());
    
    // Wait a bit for scripts to initialize
    await page.waitForTimeout(2000);
    
    // Try to access window.state
    try {
      const stateCheck = await page.evaluate(() => {
        // Log it in the browser console too
        console.log('In evaluate: typeof window.state =', typeof window.state);
        console.log('In evaluate: window.state =', window.state);
        console.log('In evaluate: window.state?.readyStates =', window.state?.readyStates);
        
        return {
          exists: typeof window.state !== 'undefined',
          value: window.state,
          readyStates: window.state?.readyStates || null
        };
      });
      
      console.log('State check result:', stateCheck);
      console.log('window.state exists:', stateCheck.exists);
      console.log('readyStates:', stateCheck.readyStates);
      
    } catch (error) {
      console.error('Error accessing window.state:', error);
    }
  });
});
