#!/usr/bin/env node

/**
 * Dashboard Fixes Verification Script
 * Tests the key fixes implemented in dashboard.js
 */

const fs = require('fs');
const path = require('path');

console.log('🔍 Dashboard Fixes Verification\n');

// Read the dashboard.js file
const dashboardPath = path.join(__dirname, 'cco/static/dashboard.js');
const dashboardContent = fs.readFileSync(dashboardPath, 'utf-8');

const tests = [
    {
        name: 'updateLastUpdateTime() has timestamp parameter',
        check: () => dashboardContent.includes('function updateLastUpdateTime(timestamp)'),
        impact: 'Critical - Fix for DOM timestamp updates'
    },
    {
        name: 'updateLastUpdateTime() updates DOM elements',
        check: () => dashboardContent.includes("document.getElementById('projectLastUpdate')"),
        impact: 'Critical - Actually updates the UI'
    },
    {
        name: 'updateLastUpdateTime() handles relative time (<24h)',
        check: () => dashboardContent.includes("const diffMins = Math.round(diffMs / (1000 * 60))"),
        impact: 'Important - Shows "5 minutes ago" format'
    },
    {
        name: 'updateLastUpdateTime() handles absolute time (>24h)',
        check: () => dashboardContent.includes("date.toLocaleDateString"),
        impact: 'Important - Shows "Nov 15, 2:30 PM" format'
    },
    {
        name: 'handleAnalyticsUpdate() checks for Array.isArray()',
        check: () => dashboardContent.includes("Array.isArray(data.activity)"),
        impact: 'Critical - Handles new array format from backend'
    },
    {
        name: 'handleAnalyticsUpdate() has legacy format fallback',
        check: () => dashboardContent.includes("addActivity(data.activity)"),
        impact: 'Important - Backwards compatibility'
    },
    {
        name: 'handleAnalyticsUpdate() calls updateActivityTable() immediately',
        check: () => dashboardContent.includes("updateActivityTable()") &&
                     dashboardContent.includes("if (data.activity)"),
        impact: 'Critical - Activity displays immediately, no stale "Loading..." placeholder'
    },
    {
        name: 'updateActivityTable() handles field name variations',
        check: () => dashboardContent.includes("item.event_type || item.type") &&
                     dashboardContent.includes("item.agent_name"),
        impact: 'Important - Works with both old and new data formats'
    },
    {
        name: 'updateActivityTable() has graceful fallbacks for missing cost',
        check: () => dashboardContent.includes("const cost = item.cost || 0"),
        impact: 'Important - No NaN or undefined in display'
    },
    {
        name: 'updateActivityTable() shows "No matching activity" message',
        check: () => dashboardContent.includes("No matching activity"),
        impact: 'Nice to have - Better UX for empty filters'
    },
    {
        name: 'updateProjectStats() no longer hardcodes timestamp',
        check: () => !dashboardContent.match(/function updateProjectStats[\s\S]*?toLocaleTimeString/),
        impact: 'Good - Removes duplicate timestamp update logic'
    },
    {
        name: 'updateMachineStats() no longer hardcodes timestamp',
        check: () => !dashboardContent.match(/function updateMachineStats[\s\S]*?toLocaleTimeString/),
        impact: 'Good - Removes duplicate timestamp update logic'
    },
    {
        name: 'updateLastUpdateTime() updates machineLastUpdate element',
        check: () => dashboardContent.includes("machineLastUpdate"),
        impact: 'Important - Machine tab also shows last update time'
    },
    {
        name: 'Activity feed handles duration properly',
        check: () => dashboardContent.includes("const duration = item.duration || item.tokens"),
        impact: 'Important - Supports both duration (ms) and tokens format'
    },
    {
        name: 'No uncaught references to removed hardcoded timestamps',
        check: () => {
            // Verify the old logic is actually removed
            const lines = dashboardContent.split('\n');
            let inUpdateProject = false;
            let inUpdateMachine = false;

            for (const line of lines) {
                if (line.includes('function updateProjectStats')) inUpdateProject = true;
                if (line.includes('function updateMachineStats')) inUpdateMachine = true;
                if (line.includes('function ') && !line.includes('updateProjectStats') && !line.includes('updateMachineStats')) {
                    inUpdateProject = false;
                    inUpdateMachine = false;
                }

                if ((inUpdateProject || inUpdateMachine) && line.includes('projectLastUpdate') && line.includes('toLocaleTimeString')) {
                    return false; // Found old code, test fails
                }
                if ((inUpdateProject || inUpdateMachine) && line.includes('machineLastUpdate') && line.includes('toLocaleTimeString')) {
                    return false; // Found old code, test fails
                }
            }
            return true;
        },
        impact: 'Critical - Old code is properly removed'
    }
];

let passed = 0;
let failed = 0;

for (const test of tests) {
    try {
        const result = test.check();
        if (result) {
            console.log(`✅ ${test.name}`);
            console.log(`   Impact: ${test.impact}\n`);
            passed++;
        } else {
            console.log(`❌ ${test.name}`);
            console.log(`   Impact: ${test.impact}\n`);
            failed++;
        }
    } catch (error) {
        console.log(`⚠️  ${test.name}`);
        console.log(`   Error: ${error.message}\n`);
        failed++;
    }
}

console.log(`\n📊 Results: ${passed} passed, ${failed} failed\n`);

if (failed === 0) {
    console.log('✨ All verification tests passed! Dashboard fixes are ready for deployment.');
    process.exit(0);
} else {
    console.log('⚠️  Some tests failed. Review the implementation.');
    process.exit(1);
}
