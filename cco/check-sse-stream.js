#!/usr/bin/env node
/**
 * Quick SSE Stream Checker
 * Connects to the SSE stream and logs what data is actually being sent
 */

const { EventSource } = require('eventsource');

console.log('🔍 Connecting to SSE stream at http://127.0.0.1:3000/api/stream...\n');

const eventSource = new EventSource('http://127.0.0.1:3000/api/stream');

let eventCount = 0;
const maxEvents = 3; // Capture first 3 events then exit

eventSource.addEventListener('analytics', (event) => {
    eventCount++;
    console.log(`📊 Event ${eventCount} received at ${new Date().toISOString()}`);
    console.log('='.repeat(80));

    try {
        const data = JSON.parse(event.data);

        // Check for claude_metrics field
        if (data.claude_metrics) {
            console.log('✅ claude_metrics EXISTS in SSE payload');
            console.log('   total_cost:', data.claude_metrics.total_cost);
            console.log('   messages_count:', data.claude_metrics.messages_count);
            console.log('   conversations_count:', data.claude_metrics.conversations_count);
            console.log('   total_input_tokens:', data.claude_metrics.total_input_tokens);
            console.log('   total_output_tokens:', data.claude_metrics.total_output_tokens);
            console.log('   model_breakdown:', Object.keys(data.claude_metrics.model_breakdown || {}));
        } else {
            console.log('❌ claude_metrics is MISSING from SSE payload');
        }

        // Show project data
        if (data.project) {
            console.log('\n📁 Project Data:');
            console.log('   name:', data.project.name);
            console.log('   cost:', data.project.cost);
            console.log('   tokens:', data.project.tokens);
            console.log('   calls:', data.project.calls);
        }

        // Show machine data
        if (data.machine) {
            console.log('\n🖥️  Machine Data:');
            console.log('   uptime:', data.machine.uptime, 'seconds');
            console.log('   process_count:', data.machine.process_count);
        }

        // Show activity count
        if (data.activity) {
            console.log('\n📝 Activity:', Array.isArray(data.activity) ? data.activity.length : 1, 'events');
        }

        console.log('\n' + '='.repeat(80));

        // Full JSON dump (commented out to avoid spam)
        // console.log('\nFull JSON payload:');
        // console.log(JSON.stringify(data, null, 2));

    } catch (error) {
        console.error('❌ Error parsing event data:', error.message);
        console.log('Raw event data:', event.data);
    }

    if (eventCount >= maxEvents) {
        console.log(`\n✅ Captured ${maxEvents} events. Closing connection.\n`);
        eventSource.close();
        process.exit(0);
    }
});

eventSource.addEventListener('error', (error) => {
    console.error('❌ SSE Error:', error);
    eventSource.close();
    process.exit(1);
});

eventSource.addEventListener('open', () => {
    console.log('✅ SSE connection established\n');
});

// Timeout after 30 seconds
setTimeout(() => {
    console.log('\n⏱️  Timeout - closing connection');
    eventSource.close();
    process.exit(0);
}, 30000);
