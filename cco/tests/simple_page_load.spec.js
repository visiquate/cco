const { test, expect } = require('@playwright/test');

let server;
const SERVER_PORT = 3001;
const SERVER_URL = `http://127.0.0.1:${SERVER_PORT}`;

test.describe('Simple Page Load Test', () => {
  test.beforeAll(async () => {
    const { spawn } = require('child_process');
    
    console.log('Starting CCO server...');
    server = spawn('/Users/brent/.cargo/bin/cco', ['run', '--debug', '--port', SERVER_PORT.toString()], {
      env: { ...process.env, NO_BROWSER: '1' },
      stdio: 'inherit'  // Show output
    });

    await new Promise(resolve => setTimeout(resolve, 5000));
  });

  test.afterAll(async () => {
    if (server) {
      server.kill('SIGINT');
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  });

  test('Simple page load', async ({ page }) => {
    page.on('console', msg => console.log('[CONSOLE]', msg.type(), msg.text()));
    page.on('error', err => console.log('[ERROR]', err));

    try {
      const response = await page.goto(SERVER_URL, { timeout: 20000 });
      console.log('Navigation response status:', response?.status());
      
      const title = await page.title();
      console.log('Page title:', title);
      
      // Just check that we can see the page
      const hasContent = await page.evaluate(() => {
        return document.body.innerHTML.length > 0;
      });
      
      console.log('Page has content:', hasContent);
      expect(hasContent).toBe(true);
      
    } catch (error) {
      console.error('Navigation failed:', error.message);
      throw error;
    }
  });
});
