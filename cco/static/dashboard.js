/**
 * CCO Dashboard - Real-Time Analytics Frontend
 * Handles tab navigation, SSE data streaming, D3 charting, and terminal emulation
 */

// ========================================
// Configuration
// ========================================

const CONFIG = {
    API_BASE: '/api',
    UPDATE_INTERVAL: 5000, // 5 seconds
    MAX_ACTIVITY_ROWS: 50,
    CHART_COLORS: {
        primary: '#3b82f6',
        success: '#10b981',
        warning: '#f59e0b',
        error: '#ef4444',
        alt: '#8b5cf6',
    },
};

// ========================================
// Global State
// ========================================

const state = {
    currentTab: 'project',
    projectStats: null,
    machineStats: null,
    activity: [],
    eventSource: null,
    terminal: null,
    ws: null,
    isConnected: false,
};

// ========================================
// Tab Navigation
// ========================================

function initTabNavigation() {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabPanels = document.querySelectorAll('.tab-panel');

    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const tabName = button.dataset.tab;
            switchTab(tabName);
        });
    });

    function switchTab(tabName) {
        // Hide all tabs
        tabPanels.forEach(panel => panel.classList.remove('active'));
        tabButtons.forEach(btn => btn.classList.remove('active'));

        // Show selected tab
        const selectedButton = document.querySelector(`[data-tab="${tabName}"]`);
        const selectedPanel = document.querySelector(`[data-panel="${tabName}"]`);

        if (selectedButton && selectedPanel) {
            selectedButton.classList.add('active');
            selectedPanel.classList.add('active');
            state.currentTab = tabName;

            // Initialize terminal if switching to terminal tab
            if (tabName === 'terminal' && !state.terminal) {
                initTerminal();
            }

            // Trigger chart redraw on resize
            if (tabName === 'machine') {
                setTimeout(() => {
                    window.dispatchEvent(new Event('resize'));
                }, 100);
            }
        }
    }
}

// ========================================
// SSE Stream Handler
// ========================================

function initSSEStream() {
    try {
        state.eventSource = new EventSource(`${CONFIG.API_BASE}/stream`);

        state.eventSource.addEventListener('analytics', event => {
            try {
                const data = JSON.parse(event.data);
                handleAnalyticsUpdate(data);
            } catch (error) {
                console.error('Error parsing analytics data:', error);
            }
        });

        state.eventSource.addEventListener('error', () => {
            console.error('EventSource error');
            updateConnectionStatus(false);
            // Attempt to reconnect
            setTimeout(initSSEStream, 5000);
        });

        updateConnectionStatus(true);
    } catch (error) {
        console.error('Error initializing SSE:', error);
        updateConnectionStatus(false);
    }
}

function handleAnalyticsUpdate(data) {
    // Update project stats
    if (data.project) {
        state.projectStats = data.project;
        updateProjectStats(data.project);
    }

    // Update machine stats
    if (data.machine) {
        state.machineStats = data.machine;
        updateMachineStats(data.machine);
    }

    // Add activity
    if (data.activity) {
        addActivity(data.activity);
    }

    // Update last update timestamp
    updateLastUpdateTime();
}

function updateConnectionStatus(isConnected) {
    state.isConnected = isConnected;
    const statusEl = document.getElementById('connectionStatus');
    const indicator = statusEl.querySelector('.status-indicator');
    const text = statusEl.querySelector('.status-text');

    if (isConnected) {
        statusEl.style.backgroundColor = 'rgba(16, 185, 129, 0.1)';
        statusEl.style.borderColor = 'rgba(16, 185, 129, 0.3)';
        statusEl.style.color = '#10b981';
        text.textContent = 'Connected';
        indicator.style.backgroundColor = '#10b981';
    } else {
        statusEl.style.backgroundColor = 'rgba(239, 68, 68, 0.1)';
        statusEl.style.borderColor = 'rgba(239, 68, 68, 0.3)';
        statusEl.style.color = '#ef4444';
        text.textContent = 'Disconnected';
        indicator.style.backgroundColor = '#ef4444';
    }
}

// ========================================
// Project Stats Update
// ========================================

function updateProjectStats(stats) {
    // Update cost
    const costEl = document.getElementById('projectCost');
    if (costEl && stats.cost !== undefined) {
        costEl.textContent = `$${stats.cost.toFixed(2)}`;
        updateTrend('projectCostTrend', stats.costTrend);
    }

    // Update tokens
    const tokensEl = document.getElementById('projectTokens');
    if (tokensEl && stats.tokens !== undefined) {
        tokensEl.textContent = formatNumber(stats.tokens);
        updateTrend('projectTokensTrend', stats.tokensTrend);
    }

    // Update API calls
    const callsEl = document.getElementById('projectCalls');
    if (callsEl && stats.calls !== undefined) {
        callsEl.textContent = formatNumber(stats.calls);
        updateTrend('projectCallsTrend', stats.callsTrend);
    }

    // Update average response time
    const timeEl = document.getElementById('projectAvgTime');
    if (timeEl && stats.avgTime !== undefined) {
        timeEl.textContent = `${stats.avgTime.toFixed(0)}ms`;
        updateTrend('projectTimeTrend', stats.timeTrend);
    }

    // Update last update time
    document.getElementById('projectLastUpdate').textContent = `Last updated: ${new Date().toLocaleTimeString()}`;
}

// ========================================
// Machine Stats Update
// ========================================

function updateMachineStats(stats) {
    // Update summary cards
    if (stats.totalCost !== undefined) {
        document.getElementById('totalCost').textContent = `$${stats.totalCost.toFixed(2)}`;
    }

    if (stats.activeProjects !== undefined) {
        document.getElementById('activeProjects').textContent = stats.activeProjects;
    }

    if (stats.totalCalls !== undefined) {
        document.getElementById('totalCalls').textContent = formatNumber(stats.totalCalls);
    }

    if (stats.totalTokens !== undefined) {
        document.getElementById('totalTokens').textContent = formatNumber(stats.totalTokens);
    }

    // Update projects table
    if (stats.projects) {
        updateProjectsTable(stats.projects);
    }

    // Update models table
    if (stats.models) {
        updateModelsTable(stats.models);
    }

    // Update discrepancies
    if (stats.discrepancies) {
        updateDiscrepancies(stats.discrepancies);
    }

    // Update charts
    if (stats.chartData) {
        updateCharts(stats.chartData);
    }

    // Update last update time
    document.getElementById('machineLastUpdate').textContent = `Last updated: ${new Date().toLocaleTimeString()}`;
}

// ========================================
// Table Updates
// ========================================

function updateProjectsTable(projects) {
    const tbody = document.getElementById('projectsTableBody');
    if (!tbody) return;

    if (!projects || projects.length === 0) {
        tbody.innerHTML = '<tr class="loading-row"><td colspan="5" class="text-center">No projects</td></tr>';
        return;
    }

    tbody.innerHTML = projects.map(project => `
        <tr>
            <td><strong>${escapeHtml(project.name)}</strong></td>
            <td>${formatNumber(project.calls)}</td>
            <td>${formatNumber(project.inputTokens)} / ${formatNumber(project.outputTokens)}</td>
            <td><strong>$${project.cost.toFixed(2)}</strong></td>
            <td>${formatTime(project.lastActivity)}</td>
        </tr>
    `).join('');
}

function updateModelsTable(models) {
    const tbody = document.getElementById('modelsTableBody');
    if (!tbody) return;

    if (!models || models.length === 0) {
        tbody.innerHTML = '<tr class="loading-row"><td colspan="6" class="text-center">No model data</td></tr>';
        return;
    }

    tbody.innerHTML = models.map(model => `
        <tr>
            <td><strong>${escapeHtml(model.name)}</strong></td>
            <td>${formatNumber(model.calls)}</td>
            <td>${formatNumber(model.inputTokens)}</td>
            <td>${formatNumber(model.outputTokens)}</td>
            <td>$${model.cost.toFixed(2)}</td>
            <td>$${(model.cost / model.calls).toFixed(6)}</td>
        </tr>
    `).join('');
}

function addActivity(activity) {
    state.activity.unshift(activity);
    // Keep only the last MAX_ACTIVITY_ROWS
    state.activity = state.activity.slice(0, CONFIG.MAX_ACTIVITY_ROWS);
    updateActivityTable();
}

function updateActivityTable() {
    const tbody = document.getElementById('activityTableBody');
    if (!tbody) return;

    if (state.activity.length === 0) {
        tbody.innerHTML = '<tr class="loading-row"><td colspan="6" class="text-center">No activity</td></tr>';
        return;
    }

    // Get filter value
    const filterValue = document.getElementById('activityFilter')?.value || 'all';

    const filtered = state.activity.filter(item => {
        if (filterValue === 'all') return true;
        return item.type === filterValue;
    });

    tbody.innerHTML = filtered.slice(0, 20).map(item => {
        const statusClass = `status-${item.status || 'pending'}`;
        return `
            <tr class="${statusClass}">
                <td>${formatTime(item.timestamp)}</td>
                <td>${escapeHtml(item.event)}</td>
                <td><span class="status-badge ${item.type}">${item.type}</span></td>
                <td>${item.duration}ms</td>
                <td>$${(item.cost || 0).toFixed(4)}</td>
                <td><span class="status-badge ${item.status}">${item.status || 'pending'}</span></td>
            </tr>
        `;
    }).join('');
}

function updateDiscrepancies(discrepancies) {
    const container = document.getElementById('discrepanciesContainer');
    if (!container) return;

    if (!discrepancies || discrepancies.length === 0) {
        container.innerHTML = '<p class="text-muted">No discrepancies detected</p>';
        return;
    }

    container.innerHTML = discrepancies.map(disc => `
        <div style="padding: 8px 0; border-bottom: 1px solid rgba(59,130,246,0.1);">
            <strong>${escapeHtml(disc.title)}</strong>
            <p style="margin: 4px 0; color: #94a3b8; font-size: 0.875rem;">${escapeHtml(disc.description)}</p>
        </div>
    `).join('');
}

// ========================================
// Charts with D3
// ========================================

function updateCharts(chartData) {
    if (chartData.costOverTime) {
        drawCostChart(chartData.costOverTime);
    }
    if (chartData.costByProject) {
        drawProjectCostChart(chartData.costByProject);
    }
    if (chartData.modelDistribution) {
        drawModelChart(chartData.modelDistribution);
    }
}

function drawCostChart(data) {
    const container = document.getElementById('costChart');
    if (!container) return;

    container.innerHTML = ''; // Clear previous chart

    const margin = { top: 10, right: 30, bottom: 30, left: 60 };
    const width = container.clientWidth - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const svg = d3.select('#costChart')
        .append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom)
        .append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);

    // Parse data
    data = data.map(d => ({
        date: new Date(d.date),
        cost: d.cost
    }));

    // Scales
    const xScale = d3.scaleTime()
        .domain(d3.extent(data, d => d.date))
        .range([0, width]);

    const yScale = d3.scaleLinear()
        .domain([0, d3.max(data, d => d.cost)])
        .range([height, 0]);

    // Line generator
    const line = d3.line()
        .x(d => xScale(d.date))
        .y(d => yScale(d.cost));

    // Draw path
    svg.append('path')
        .datum(data)
        .attr('class', 'chart-line')
        .attr('d', line);

    // Add X axis
    svg.append('g')
        .attr('transform', `translate(0,${height})`)
        .attr('class', 'chart-axis')
        .call(d3.axisBottom(xScale).tickFormat(d3.timeFormat('%b %d')));

    // Add Y axis
    svg.append('g')
        .attr('class', 'chart-axis')
        .call(d3.axisLeft(yScale).tickFormat(d => `$${d}`));
}

function drawProjectCostChart(data) {
    const container = document.getElementById('projectCostChart');
    if (!container) return;

    container.innerHTML = '';

    const margin = { top: 10, right: 30, bottom: 30, left: 60 };
    const width = container.clientWidth - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const svg = d3.select('#projectCostChart')
        .append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom)
        .append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);

    // Sort data
    data = data.sort((a, b) => b.cost - a.cost).slice(0, 10);

    // Scales
    const xScale = d3.scaleBand()
        .domain(data.map(d => d.project))
        .range([0, width])
        .padding(0.1);

    const yScale = d3.scaleLinear()
        .domain([0, d3.max(data, d => d.cost)])
        .range([height, 0]);

    // Draw bars
    svg.selectAll('.bar')
        .data(data)
        .enter()
        .append('rect')
        .attr('class', 'chart-bar')
        .attr('x', d => xScale(d.project))
        .attr('y', d => yScale(d.cost))
        .attr('width', xScale.bandwidth())
        .attr('height', d => height - yScale(d.cost));

    // Add X axis
    svg.append('g')
        .attr('transform', `translate(0,${height})`)
        .attr('class', 'chart-axis')
        .call(d3.axisBottom(xScale))
        .selectAll('text')
        .attr('transform', 'rotate(-45)')
        .style('text-anchor', 'end');

    // Add Y axis
    svg.append('g')
        .attr('class', 'chart-axis')
        .call(d3.axisLeft(yScale).tickFormat(d => `$${d}`));
}

function drawModelChart(data) {
    const container = document.getElementById('modelChart');
    if (!container) return;

    container.innerHTML = '';

    const width = container.clientWidth;
    const height = 300;
    const radius = Math.min(width, height) / 2;

    const svg = d3.select('#modelChart')
        .append('svg')
        .attr('width', width)
        .attr('height', height)
        .append('g')
        .attr('transform', `translate(${width / 2},${height / 2})`);

    // Create pie
    const pie = d3.pie().value(d => d.count);
    const arc = d3.arc().innerRadius(0).outerRadius(radius - 10);

    // Colors
    const colors = ['#3b82f6', '#8b5cf6', '#10b981', '#f59e0b', '#ef4444'];

    // Draw slices
    svg.selectAll('.arc')
        .data(pie(data))
        .enter()
        .append('g')
        .attr('class', 'arc')
        .append('path')
        .attr('class', 'chart-pie-slice')
        .attr('d', arc)
        .attr('fill', (d, i) => colors[i % colors.length]);

    // Add labels
    svg.selectAll('.label')
        .data(pie(data))
        .enter()
        .append('text')
        .attr('transform', d => `translate(${arc.centroid(d)})`)
        .attr('text-anchor', 'middle')
        .attr('fill', '#f1f5f9')
        .attr('font-size', '12px')
        .text(d => d.data.model);
}

// ========================================
// Terminal Emulation
// ========================================

function initTerminal() {
    const terminalElement = document.getElementById('terminal');
    if (!terminalElement) return;

    state.terminal = new Terminal({
        rows: 25,
        cols: 120,
        theme: {
            background: '#1e293b',
            foreground: '#f1f5f9',
            cursor: '#3b82f6',
        },
        fontSize: 14,
        fontFamily: 'Monaco, Menlo, Ubuntu Mono, monospace',
    });

    state.terminal.open(terminalElement);

    // Load and apply FitAddon if available
    let fitAddon = null;
    if (typeof FitAddon !== 'undefined') {
        fitAddon = new FitAddon();
        state.terminal.loadAddon(fitAddon);
        fitAddon.fit();
    }

    // Handle window resize
    window.addEventListener('resize', () => {
        if (state.terminal && state.currentTab === 'terminal' && fitAddon) {
            fitAddon.fit();
        }
    });

    // Initialize WebSocket
    initTerminalWebSocket();

    // Handle terminal input
    state.terminal.onData(data => {
        if (state.ws && state.ws.readyState === WebSocket.OPEN) {
            state.ws.send(new TextEncoder().encode(data));
        }
    });

    // Setup terminal control buttons
    document.getElementById('terminalClearBtn')?.addEventListener('click', () => {
        state.terminal.clear();
    });

    document.getElementById('terminalCopyBtn')?.addEventListener('click', () => {
        const buffer = state.terminal.getSelectionText() || state.terminal.toString();
        navigator.clipboard.writeText(buffer);
    });
}

function initTerminalWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/terminal`;

    try {
        state.ws = new WebSocket(wsUrl);
        state.ws.binaryType = 'arraybuffer';

        state.ws.onopen = () => {
            console.log('Terminal WebSocket connected');
        };

        state.ws.onmessage = event => {
            if (state.terminal && event.data instanceof ArrayBuffer) {
                const text = new TextDecoder().decode(new Uint8Array(event.data));
                state.terminal.write(text);
            }
        };

        state.ws.onerror = error => {
            console.error('Terminal WebSocket error:', error);
        };

        state.ws.onclose = () => {
            console.log('Terminal WebSocket closed');
            // Attempt to reconnect
            setTimeout(initTerminalWebSocket, 3000);
        };
    } catch (error) {
        console.error('Error creating WebSocket:', error);
    }
}

// ========================================
// Helper Functions
// ========================================

function updateTrend(elementId, trend) {
    const el = document.getElementById(elementId);
    if (!el) return;

    if (!trend) {
        el.textContent = '';
        return;
    }

    const direction = trend.value > 0 ? '↑' : '↓';
    const color = trend.value > 0 ? '#ef4444' : '#10b981';
    el.textContent = `${direction} ${Math.abs(trend.value).toFixed(1)}% from ${trend.period}`;
    el.style.color = color;
}

function updateLastUpdateTime() {
    const now = new Date();
    const timeString = now.toLocaleTimeString();
    // Don't update timestamps too frequently to reduce DOM operations
}

function formatNumber(num) {
    if (num === undefined || num === null) return '0';
    if (num >= 1e9) return (num / 1e9).toFixed(1) + 'B';
    if (num >= 1e6) return (num / 1e6).toFixed(1) + 'M';
    if (num >= 1e3) return (num / 1e3).toFixed(1) + 'K';
    return num.toString();
}

function formatTime(timestamp) {
    if (!timestamp) return '-';
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;

    // Less than 1 minute
    if (diff < 60000) {
        return 'Just now';
    }

    // Less than 1 hour
    if (diff < 3600000) {
        const mins = Math.floor(diff / 60000);
        return `${mins}m ago`;
    }

    // Less than 1 day
    if (diff < 86400000) {
        const hours = Math.floor(diff / 3600000);
        return `${hours}h ago`;
    }

    // Default to time string
    return date.toLocaleTimeString();
}

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ========================================
// Event Listeners
// ========================================

function setupEventListeners() {
    // Refresh button
    document.getElementById('refreshBtn')?.addEventListener('click', () => {
        location.reload();
    });

    // Activity filter
    document.getElementById('activityFilter')?.addEventListener('change', () => {
        updateActivityTable();
    });

    // Export button
    document.getElementById('exportProjectsBtn')?.addEventListener('click', () => {
        exportProjectsToCSV();
    });

    // Shutdown button
    document.getElementById('shutdownBtn')?.addEventListener('click', () => {
        handleShutdown();
    });
}

function exportProjectsToCSV() {
    if (!state.machineStats || !state.machineStats.projects) {
        alert('No project data to export');
        return;
    }

    const projects = state.machineStats.projects;
    const headers = ['Project Name', 'API Calls', 'Input Tokens', 'Output Tokens', 'Cost', 'Last Activity'];

    const rows = [headers];
    projects.forEach(project => {
        rows.push([
            project.name,
            project.calls,
            project.inputTokens,
            project.outputTokens,
            project.cost.toFixed(2),
            new Date(project.lastActivity).toISOString()
        ]);
    });

    const csv = rows.map(row => row.map(cell => `"${cell}"`).join(',')).join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `projects-${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    window.URL.revokeObjectURL(url);
}

// ========================================
// Shutdown Handler
// ========================================

async function handleShutdown() {
    // Show confirmation dialog
    const confirmed = confirm(
        'Are you sure you want to shut down the CCO server?\n\n' +
        'This will:\n' +
        '- Terminate all active sessions\n' +
        '- Close all connections\n' +
        '- Stop the server immediately\n\n' +
        'Click OK to proceed with shutdown.'
    );

    if (!confirmed) {
        return;
    }

    try {
        // Disable the button to prevent multiple clicks
        const shutdownBtn = document.getElementById('shutdownBtn');
        if (shutdownBtn) {
            shutdownBtn.disabled = true;
            shutdownBtn.textContent = 'Shutting down...';
        }

        // Send shutdown request
        const response = await fetch(`${CONFIG.API_BASE}/shutdown`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            }
        });

        if (response.ok) {
            // Show success message
            alert('Server shutdown initiated successfully.\n\nThe dashboard will disconnect shortly.');

            // Update connection status
            updateConnectionStatus(false);

            // Close event source if open
            if (state.eventSource) {
                state.eventSource.close();
            }

            // Close WebSocket if open
            if (state.ws) {
                state.ws.close();
            }
        } else {
            // Handle error response
            const errorText = await response.text();
            console.error('Shutdown failed:', errorText);
            alert(`Failed to shut down server: ${errorText}`);

            // Re-enable button on error
            if (shutdownBtn) {
                shutdownBtn.disabled = false;
                shutdownBtn.textContent = 'Shutdown Server';
            }
        }
    } catch (error) {
        console.error('Error during shutdown:', error);

        // Check if it's a network error (expected during shutdown)
        if (error.name === 'TypeError' && error.message.includes('fetch')) {
            // This is expected - server shut down before response
            alert('Server shutdown initiated. Connection lost as expected.');
            updateConnectionStatus(false);
        } else {
            alert(`Error during shutdown: ${error.message}`);

            // Re-enable button on unexpected error
            const shutdownBtn = document.getElementById('shutdownBtn');
            if (shutdownBtn) {
                shutdownBtn.disabled = false;
                shutdownBtn.textContent = 'Shutdown Server';
            }
        }
    }
}

// ========================================
// Initialization
// ========================================

document.addEventListener('DOMContentLoaded', () => {
    console.log('Initializing CCO Dashboard...');

    // Initialize components
    initTabNavigation();
    setupEventListeners();

    // Load initial data
    loadInitialData();

    // Start SSE stream
    initSSEStream();

    // Periodic refresh
    setInterval(() => {
        if (state.currentTab === 'machine') {
            loadMachineStats();
        }
    }, CONFIG.UPDATE_INTERVAL);
});

async function loadInitialData() {
    try {
        // Fetch version info from health endpoint
        const healthRes = await fetch('/health');
        if (healthRes.ok) {
            const healthData = await healthRes.json();
            const versionInfo = document.getElementById('versionInfo');
            const settingsVersion = document.getElementById('settingsVersion');

            if (healthData.version) {
                const versionText = `v${healthData.version}`;
                if (versionInfo) {
                    versionInfo.textContent = versionText;
                }
                if (settingsVersion) {
                    settingsVersion.textContent = versionText;
                }
            }
        }

        // Fetch stats from unified endpoint
        const statsRes = await fetch(`${CONFIG.API_BASE}/stats`);
        if (statsRes.ok) {
            const data = await statsRes.json();

            // Update both project and machine stats
            if (data.project) {
                state.projectStats = data.project;
                updateProjectStats(data.project);
            }

            if (data.machine) {
                state.machineStats = data.machine;
                updateMachineStats(data.machine);
            }
        }
    } catch (error) {
        console.error('Error loading initial data:', error);
        const versionInfo = document.getElementById('versionInfo');
        const settingsVersion = document.getElementById('settingsVersion');

        if (versionInfo) {
            versionInfo.textContent = 'Error';
        }
        if (settingsVersion) {
            settingsVersion.textContent = 'Error';
        }
    }
}

async function loadMachineStats() {
    try {
        const res = await fetch(`${CONFIG.API_BASE}/stats`);
        if (res.ok) {
            const data = await res.json();
            if (data.machine) {
                state.machineStats = data.machine;
                updateMachineStats(data.machine);
            }
        }
    } catch (error) {
        console.error('Error loading machine stats:', error);
    }
}

// Export for testing
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        formatNumber,
        formatTime,
        escapeHtml,
    };
}
