/**
 * Playwright Debug Script for CCO Dashboard
 * Investigates xterm errors and Claude metrics display issues
 */

const { chromium } = require('playwright');

async function debugDashboard() {
    console.log('🔍 Starting CCO Dashboard Debug Investigation...\n');

    const browser = await chromium.launch({ headless: false });
    const context = await browser.newContext();
    const page = await context.newPage();

    // Capture all console messages
    const consoleMessages = [];
    page.on('console', msg => {
        const type = msg.type();
        const text = msg.text();
        consoleMessages.push({ type, text, timestamp: new Date().toISOString() });
        console.log(`[${type.toUpperCase()}] ${text}`);
    });

    // Capture all network errors
    const networkErrors = [];
    page.on('requestfailed', request => {
        networkErrors.push({
            url: request.url(),
            failure: request.failure().errorText,
            timestamp: new Date().toISOString()
        });
        console.log(`[NETWORK ERROR] ${request.url()} - ${request.failure().errorText}`);
    });

    // Capture all network responses
    const networkRequests = [];
    page.on('response', response => {
        networkRequests.push({
            url: response.url(),
            status: response.status(),
            contentType: response.headers()['content-type'],
            timestamp: new Date().toISOString()
        });
    });

    try {
        console.log('📡 Navigating to http://127.0.0.1:3000...');
        // Don't wait for networkidle since xterm error prevents it
        await page.goto('http://127.0.0.1:3000', { waitUntil: 'domcontentloaded', timeout: 30000 });

        console.log('✅ Page loaded successfully\n');

        // Wait a bit for SSE data
        console.log('⏳ Waiting 5 seconds for SSE data...');
        await page.waitForTimeout(5000);

        // ========================================
        // CHECK 1: xterm Script URLs
        // ========================================
        console.log('\n🔍 CHECK 1: xterm Script URLs');
        console.log('='.repeat(60));

        const scriptTags = await page.evaluate(() => {
            const scripts = Array.from(document.querySelectorAll('script'));
            return scripts
                .filter(s => s.src.includes('xterm'))
                .map(s => ({
                    src: s.src,
                    loaded: s.readyState || 'unknown'
                }));
        });

        console.log('Found xterm scripts:');
        scriptTags.forEach(script => {
            console.log(`  - ${script.src} [${script.loaded}]`);
        });

        const xtermAddonFitScript = scriptTags.find(s => s.src.includes('xterm-addon-fit'));
        const xtermFitCheck = xtermAddonFitScript
            ? '✅ xterm-addon-fit@0.8.0 script tag found'
            : '❌ xterm-addon-fit@0.8.0 script tag NOT found';
        console.log(xtermFitCheck);

        // ========================================
        // CHECK 2: Claude Metrics DOM Elements
        // ========================================
        console.log('\n🔍 CHECK 2: Claude Metrics DOM Elements');
        console.log('='.repeat(60));

        const metricsElements = await page.evaluate(() => {
            return {
                projectCost: {
                    exists: !!document.getElementById('projectCost'),
                    value: document.getElementById('projectCost')?.textContent || 'N/A',
                    innerHTML: document.getElementById('projectCost')?.innerHTML || 'N/A'
                },
                projectTokens: {
                    exists: !!document.getElementById('projectTokens'),
                    value: document.getElementById('projectTokens')?.textContent || 'N/A'
                },
                projectCalls: {
                    exists: !!document.getElementById('projectCalls'),
                    value: document.getElementById('projectCalls')?.textContent || 'N/A'
                },
                modelBreakdown: {
                    exists: !!document.querySelector('[data-section="model-breakdown"]'),
                    html: document.querySelector('[data-section="model-breakdown"]')?.innerHTML || 'N/A'
                }
            };
        });

        console.log('Metrics DOM Elements:');
        console.log(`  projectCost: ${metricsElements.projectCost.exists ? '✅' : '❌'} exists, value="${metricsElements.projectCost.value}"`);
        console.log(`  projectTokens: ${metricsElements.projectTokens.exists ? '✅' : '❌'} exists, value="${metricsElements.projectTokens.value}"`);
        console.log(`  projectCalls: ${metricsElements.projectCalls.exists ? '✅' : '❌'} exists, value="${metricsElements.projectCalls.value}"`);
        console.log(`  modelBreakdown: ${metricsElements.modelBreakdown.exists ? '✅' : '❌'} exists`);

        // ========================================
        // CHECK 3: SSE Stream Events
        // ========================================
        console.log('\n🔍 CHECK 3: SSE Stream Events');
        console.log('='.repeat(60));

        const sseRequests = networkRequests.filter(r => r.url.includes('/api/stream'));
        console.log(`SSE requests: ${sseRequests.length > 0 ? '✅' : '❌'} found (${sseRequests.length})`);
        sseRequests.forEach(req => {
            console.log(`  - ${req.url} [${req.status}] ${req.contentType}`);
        });

        // Check if EventSource is connected
        const eventSourceState = await page.evaluate(() => {
            return {
                exists: typeof window.state !== 'undefined' && window.state && window.state.eventSource !== null,
                readyState: window.state?.eventSource?.readyState,
                isConnected: window.state?.isConnected
            };
        });

        console.log(`EventSource state: ${eventSourceState.exists ? '✅' : '❌'} exists`);
        if (eventSourceState.exists) {
            console.log(`  readyState: ${eventSourceState.readyState} (0=CONNECTING, 1=OPEN, 2=CLOSED)`);
            console.log(`  isConnected: ${eventSourceState.isConnected}`);
        }

        // ========================================
        // CHECK 4: Global State (claude_metrics)
        // ========================================
        console.log('\n🔍 CHECK 4: Global State (claude_metrics)');
        console.log('='.repeat(60));

        const globalState = await page.evaluate(() => {
            return {
                claudeMetrics: window.state?.claudeMetrics || null,
                projectStats: window.state?.projectStats || null,
                machineStats: window.state?.machineStats || null
            };
        });

        console.log('Global state.claudeMetrics:');
        if (globalState.claudeMetrics) {
            console.log('✅ claudeMetrics exists in state');
            console.log(`  total_cost: $${globalState.claudeMetrics.total_cost || 0}`);
            console.log(`  messages_count: ${globalState.claudeMetrics.messages_count || 0}`);
            console.log(`  total_input_tokens: ${globalState.claudeMetrics.total_input_tokens || 0}`);
            console.log(`  total_output_tokens: ${globalState.claudeMetrics.total_output_tokens || 0}`);
            console.log(`  model_breakdown keys: ${Object.keys(globalState.claudeMetrics.model_breakdown || {}).join(', ')}`);
        } else {
            console.log('❌ claudeMetrics is NULL in state');
        }

        // ========================================
        // CHECK 5: Function Availability
        // ========================================
        console.log('\n🔍 CHECK 5: Function Availability');
        console.log('='.repeat(60));

        const functionChecks = await page.evaluate(() => {
            return {
                updateClaudeMetrics: typeof window.updateClaudeMetrics === 'function',
                handleAnalyticsUpdate: typeof window.handleAnalyticsUpdate === 'function',
                updateModelBreakdown: typeof window.updateModelBreakdown === 'function',
                FitAddon: typeof FitAddon !== 'undefined'
            };
        });

        console.log(`updateClaudeMetrics(): ${functionChecks.updateClaudeMetrics ? '✅' : '❌'} available`);
        console.log(`handleAnalyticsUpdate(): ${functionChecks.handleAnalyticsUpdate ? '✅' : '❌'} available`);
        console.log(`updateModelBreakdown(): ${functionChecks.updateModelBreakdown ? '✅' : '❌'} available`);
        console.log(`FitAddon: ${functionChecks.FitAddon ? '✅' : '❌'} available (xterm addon)`);

        // ========================================
        // CHECK 6: Network Errors (xterm-addon-fit)
        // ========================================
        console.log('\n🔍 CHECK 6: Network Errors');
        console.log('='.repeat(60));

        const xtermErrors = networkErrors.filter(e => e.url.includes('xterm'));
        if (xtermErrors.length > 0) {
            console.log('❌ xterm-related network errors found:');
            xtermErrors.forEach(err => {
                console.log(`  - ${err.url}`);
                console.log(`    Error: ${err.failure}`);
            });
        } else {
            console.log('✅ No xterm-related network errors');
        }

        // ========================================
        // CHECK 7: Take Screenshot
        // ========================================
        console.log('\n📸 Taking screenshot...');
        await page.screenshot({ path: '/tmp/cco-dashboard-debug.png', fullPage: true });
        console.log('✅ Screenshot saved to /tmp/cco-dashboard-debug.png');

        // ========================================
        // SUMMARY REPORT
        // ========================================
        console.log('\n' + '='.repeat(60));
        console.log('📊 SUMMARY REPORT');
        console.log('='.repeat(60));

        const report = {
            xtermScriptURL: xtermAddonFitScript ? '✅ Correct' : '❌ Wrong/Missing',
            claudeMetricsData: globalState.claudeMetrics ? '✅ Data exists' : '❌ No data',
            updateFunctionCalled: functionChecks.updateClaudeMetrics ? '✅ Function exists' : '❌ Function missing',
            metricsDisplaying: metricsElements.projectCost.value !== '$0.00' ? '✅ Displaying' : '❌ Not displaying',
            modelBreakdown: metricsElements.modelBreakdown.exists ? '✅ Exists' : '❌ Missing',
            networkErrors: xtermErrors.length === 0 ? '✅ No errors' : `❌ ${xtermErrors.length} errors`
        };

        console.log('\nChecklist Results:');
        Object.entries(report).forEach(([key, status]) => {
            console.log(`  ${key}: ${status}`);
        });

        // Root cause analysis
        console.log('\n🔬 ROOT CAUSE ANALYSIS:');

        if (!functionChecks.FitAddon) {
            console.log('❌ ISSUE: FitAddon not loaded - xterm-addon-fit.js failed to load');
            console.log('   Likely cause: Script URL is correct but file not served or MIME type issue');
        }

        if (!globalState.claudeMetrics) {
            console.log('❌ ISSUE: No claude_metrics in state - SSE not sending data');
            console.log('   Likely cause: Backend not sending claude_metrics in analytics events');
        } else if (metricsElements.projectCost.value === '$0.00') {
            console.log('⚠️  ISSUE: claude_metrics exists but not displaying');
            console.log('   Likely cause: updateClaudeMetrics() not being called from handleAnalyticsUpdate()');
        }

        console.log('\n✅ Debug investigation complete!');

    } catch (error) {
        console.error('❌ Error during investigation:', error.message);
        console.error(error.stack);
    } finally {
        console.log('\n⏳ Keeping browser open for 30 seconds for manual inspection...');
        await page.waitForTimeout(30000);
        await browser.close();
    }
}

// Run the debug script
debugDashboard().catch(console.error);
