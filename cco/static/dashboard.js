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
    claudeMetrics: null,
    activity: [],
    eventSource: null,
    terminal: null,
    fitAddon: null,
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
        updateLastUpdateTime(data.project.lastUpdated || new Date().toISOString());
    }

    // Update machine stats (use claude_metrics for cost/token data)
    if (data.machine || data.claude_metrics) {
        state.machineStats = data.machine;
        // Pass claude_metrics so machine-wide analytics can show conversation costs
        updateMachineStats(data.machine, data.claude_metrics);
    }

    // Handle Claude metrics (actual conversation history)
    if (data.claude_metrics) {
        state.claudeMetrics = data.claude_metrics;
        // Always update Claude metrics - they represent real conversation history
        updateClaudeMetrics(data.claude_metrics);
    }

    // Handle activity - can be array or single object
    if (data.activity) {
        if (Array.isArray(data.activity)) {
            // New format: activity is an array
            state.activity = data.activity.slice(0, CONFIG.MAX_ACTIVITY_ROWS);
        } else {
            // Legacy format: activity is single object
            addActivity(data.activity);
        }
        updateActivityTable();
    }

    // Handle chart data
    if (data.chart_data) {
        state.chartData = data.chart_data;
        updateCharts(data.chart_data);
    }
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
}

function updateClaudeMetrics(metrics) {
    // Update the Current Project tab with Claude conversation history
    const totalTokens = metrics.total_input_tokens + metrics.total_output_tokens;
    const costDisplay = (metrics.total_cost).toLocaleString('en-US', {
        style: 'currency',
        currency: 'USD',
        minimumFractionDigits: 2,
        maximumFractionDigits: 2
    });

    // Update DOM elements using actual element IDs
    const costEl = document.getElementById('projectCost');
    if (costEl) costEl.textContent = costDisplay;

    const tokenEl = document.getElementById('projectTokens');
    if (tokenEl) tokenEl.textContent = totalTokens.toLocaleString();

    const callsEl = document.getElementById('projectCalls');
    if (callsEl) callsEl.textContent = metrics.messages_count.toLocaleString();

    // Update model breakdown (create if doesn't exist)
    updateModelBreakdown(metrics.model_breakdown);
}

function updateModelBreakdown(breakdown) {
    // Display model usage percentages
    // Find or create the model breakdown section in the project tab panel (not the button)
    let modelsDiv = document.querySelector('[data-section="model-breakdown"]');

    if (!modelsDiv) {
        // Create the section if it doesn't exist
        // Append to the tab-panel content area, not the button
        const projectPanel = document.querySelector('[data-panel="project"]');
        if (!projectPanel) return;

        const sectionCard = document.createElement('div');
        sectionCard.className = 'section-card';
        sectionCard.style.marginTop = '2rem';
        sectionCard.innerHTML = `
            <div class="section-header">
                <h3>Model Breakdown</h3>
            </div>
            <div data-section="model-breakdown"></div>
        `;
        projectPanel.appendChild(sectionCard);
        modelsDiv = sectionCard.querySelector('[data-section="model-breakdown"]');
    }

    const totalCost = Object.values(breakdown).reduce((sum, model) => sum + (model.total_cost || 0), 0);

    let html = '<div class="model-breakdown">';
    for (const [model, data] of Object.entries(breakdown)) {
        const modelTotalCost = data.total_cost || 0;
        if (modelTotalCost === 0 && data.input_tokens === 0 && data.output_tokens === 0) continue; // Skip zero models

        const percentage = totalCost > 0 ? (modelTotalCost / totalCost * 100).toFixed(1) : 0;
        const totalModelTokens = data.input_tokens + data.output_tokens;
        // Use model name directly - it's already clean from the backend
        const modelName = model;

        const cost = modelTotalCost;
        const messageCount = data.message_count !== undefined && data.message_count !== null ? data.message_count : 0;

        html += `
            <div class="model-row" style="padding: 12px 0; border-bottom: 1px solid rgba(59,130,246,0.1);">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                    <span class="model-name" style="font-weight: 600; text-transform: capitalize;">${escapeHtml(modelName)}</span>
                    <span class="model-percentage" style="color: #f59e0b; font-weight: bold;">${percentage}%</span>
                </div>
                <div style="display: flex; justify-content: space-between; font-size: 0.875rem; color: #94a3b8;">
                    <span class="model-cost" style="font-weight: 500;">$${cost.toFixed(2)}</span>
                    <span class="model-tokens">${totalModelTokens.toLocaleString()} tokens (${messageCount} msgs)</span>
                </div>
            </div>
        `;
    }
    html += '</div>';
    modelsDiv.innerHTML = html;
}

// ========================================
// Machine Stats Update
// ========================================

function updateMachineStats(stats, claudeMetrics) {
    // Use claudeMetrics for real cost/token data from conversations
    // Fall back to stats object if claudeMetrics not available

    const metrics = claudeMetrics || {};

    // Update total cost
    const totalCost = metrics.total_cost !== undefined ? metrics.total_cost : 0;
    const totalCostEl = document.getElementById('totalCost');
    if (totalCostEl) {
        totalCostEl.textContent = `$${totalCost.toFixed(2)}`;
    }

    // Update active projects (use conversations count)
    const conversationsCount = metrics.conversations_count !== undefined ? metrics.conversations_count : 0;
    const activeProjectsEl = document.getElementById('activeProjects');
    if (activeProjectsEl) {
        activeProjectsEl.textContent = conversationsCount;
    }

    // Update total API calls (sum of all model message counts)
    let totalCalls = 0;
    if (metrics.model_breakdown) {
        Object.values(metrics.model_breakdown).forEach(model => {
            totalCalls += model.message_count || 0;
        });
    }
    const totalCallsEl = document.getElementById('totalCalls');
    if (totalCallsEl) {
        totalCallsEl.textContent = formatNumber(totalCalls);
    }

    // Update total tokens
    const totalInputTokens = metrics.total_input_tokens !== undefined ? metrics.total_input_tokens : 0;
    const totalOutputTokens = metrics.total_output_tokens !== undefined ? metrics.total_output_tokens : 0;
    const totalTokens = totalInputTokens + totalOutputTokens;
    const totalTokensEl = document.getElementById('totalTokens');
    if (totalTokensEl) {
        totalTokensEl.textContent = formatNumber(totalTokens);
    }

    // Update projects table
    if (stats.projects) {
        updateProjectsTable(stats.projects);
    }

    // Update models table from claude_metrics model breakdown
    if (metrics.model_breakdown) {
        updateModelsTableFromBreakdown(metrics.model_breakdown);
    }

    // Update discrepancies
    if (stats.discrepancies) {
        updateDiscrepancies(stats.discrepancies);
    }

    // Update charts
    if (stats.chartData) {
        updateCharts(stats.chartData);
    }
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

    tbody.innerHTML = projects.map(project => {
        const cost = project.cost !== undefined && project.cost !== null ? project.cost : 0;
        // Handle both snake_case (from server) and camelCase (legacy)
        const apiCalls = project.api_calls !== undefined ? project.api_calls : project.calls || 0;
        const inputTokens = project.input_tokens !== undefined ? project.input_tokens : project.inputTokens || 0;
        const outputTokens = project.output_tokens !== undefined ? project.output_tokens : project.outputTokens || 0;
        const lastActivity = project.last_activity !== undefined ? project.last_activity : project.lastActivity || new Date().toISOString();

        return `
        <tr>
            <td><strong>${escapeHtml(project.name)}</strong></td>
            <td>${formatNumber(apiCalls)}</td>
            <td>${formatNumber(inputTokens)} / ${formatNumber(outputTokens)}</td>
            <td><strong>$${cost.toFixed(2)}</strong></td>
            <td>${formatTime(lastActivity)}</td>
        </tr>
    `;
    }).join('');
}

function updateModelsTable(models) {
    const tbody = document.getElementById('modelsTableBody');
    if (!tbody) return;

    if (!models || models.length === 0) {
        tbody.innerHTML = '<tr class="loading-row"><td colspan="6" class="text-center">No model data</td></tr>';
        return;
    }

    tbody.innerHTML = models.map(model => {
        const cost = model.cost !== undefined && model.cost !== null ? model.cost : 0;
        const calls = model.calls !== undefined && model.calls !== null ? model.calls : 0;
        const avgCost = calls > 0 ? (cost / calls).toFixed(6) : '0.000000';
        return `
        <tr>
            <td><strong>${escapeHtml(model.name)}</strong></td>
            <td>${formatNumber(model.calls)}</td>
            <td>${formatNumber(model.inputTokens)}</td>
            <td>${formatNumber(model.outputTokens)}</td>
            <td>$${cost.toFixed(2)}</td>
            <td>$${avgCost}</td>
        </tr>
    `;
    }).join('');
}

function updateModelsTableFromBreakdown(breakdown) {
    const tbody = document.getElementById('modelsTableBody');
    if (!tbody) return;

    if (!breakdown || Object.keys(breakdown).length === 0) {
        tbody.innerHTML = '<tr class="loading-row"><td colspan="6" class="text-center">No model data</td></tr>';
        return;
    }

    // Sort entries by cost descending for consistent ordering
    const sortedEntries = Object.entries(breakdown).sort((a, b) => {
        const costA = (a[1].total_cost || 0);
        const costB = (b[1].total_cost || 0);
        return costB - costA;
    });

    tbody.innerHTML = sortedEntries.map(([modelName, data]) => {
        const cost = data.total_cost !== undefined ? data.total_cost : 0;
        const calls = data.message_count !== undefined ? data.message_count : 0;
        const inputTokens = data.input_tokens !== undefined ? data.input_tokens : 0;
        const outputTokens = data.output_tokens !== undefined ? data.output_tokens : 0;
        const avgCost = calls > 0 ? (cost / calls).toFixed(6) : '0.000000';

        return `
        <tr>
            <td><strong>${escapeHtml(modelName)}</strong></td>
            <td>${formatNumber(calls)}</td>
            <td>${formatNumber(inputTokens)}</td>
            <td>${formatNumber(outputTokens)}</td>
            <td>$${cost.toFixed(2)}</td>
            <td>$${avgCost}</td>
        </tr>
        `;
    }).join('');
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
        // Handle different field names for activity type
        return (item.type || item.event_type || '').toLowerCase() === filterValue.toLowerCase();
    });

    if (filtered.length === 0) {
        tbody.innerHTML = '<tr class="loading-row"><td colspan="6" class="text-center">No matching activity</td></tr>';
        return;
    }

    tbody.innerHTML = filtered.slice(0, 20).map(item => {
        // Handle different field names for event/event_type
        const eventType = item.type || item.event_type || 'event';
        const eventName = item.event || item.agent_name || 'system';
        const statusClass = `status-${item.status || 'pending'}`;
        const duration = item.duration || item.tokens || 0;
        const cost = (item.cost !== undefined && item.cost !== null) ? item.cost : 0;

        return `
            <tr class="${statusClass}">
                <td>${formatTime(item.timestamp)}</td>
                <td>${escapeHtml(eventName)}</td>
                <td><span class="status-badge ${eventType}">${eventType}</span></td>
                <td>${duration}${typeof duration === 'number' && duration > 100 ? 'ms' : ''}</td>
                <td>$${(cost).toFixed(4)}</td>
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

// Define terminal themes
const darkTheme = {
    background: '#0f172a',
    foreground: '#e2e8f0',
    cursor: '#60a5fa',
    cursorAccent: '#1e293b',
    selection: 'rgba(96, 165, 250, 0.3)',
    black: '#1e293b',
    red: '#ef4444',
    green: '#10b981',
    yellow: '#f59e0b',
    blue: '#3b82f6',
    magenta: '#a855f7',
    cyan: '#06b6d4',
    white: '#e2e8f0',
    brightBlack: '#475569',
    brightRed: '#f87171',
    brightGreen: '#34d399',
    brightYellow: '#fbbf24',
    brightBlue: '#60a5fa',
    brightMagenta: '#c084fc',
    brightCyan: '#22d3ee',
    brightWhite: '#f1f5f9'
};

const lightTheme = {
    background: '#ffffff',
    foreground: '#1e293b',
    cursor: '#2563eb',
    cursorAccent: '#f1f5f9',
    selection: 'rgba(37, 99, 235, 0.3)',
    black: '#1e293b',
    red: '#dc2626',
    green: '#059669',
    yellow: '#d97706',
    blue: '#2563eb',
    magenta: '#9333ea',
    cyan: '#0891b2',
    white: '#f1f5f9',
    brightBlack: '#475569',
    brightRed: '#ef4444',
    brightGreen: '#10b981',
    brightYellow: '#f59e0b',
    brightBlue: '#3b82f6',
    brightMagenta: '#a855f7',
    brightCyan: '#06b6d4',
    brightWhite: '#ffffff'
};

function initTerminal() {
    const terminalElement = document.getElementById('terminal');
    if (!terminalElement) return;

    try {
        // Determine current theme
        const isDark = document.documentElement.getAttribute('data-theme') === 'dark';

        // Create terminal instance with proper configuration
        state.terminal = new Terminal({
            fontSize: 14,
            fontFamily: 'Monaco, Menlo, Ubuntu Mono, Consolas, "Courier New", monospace',
            cursorBlink: true,
            cursorStyle: 'block',
            theme: isDark ? darkTheme : lightTheme,
            scrollback: 1000,
            cols: 120,
            rows: 30,
            allowProposedApi: true
        });

        // Load FitAddon
        state.fitAddon = null;
        try {
            let FitAddonClass = null;
            if (typeof window.FitAddon === 'function') {
                FitAddonClass = window.FitAddon;
            } else if (typeof window.FitAddon === 'object' && window.FitAddon !== null) {
                FitAddonClass = window.FitAddon.FitAddon || window.FitAddon.default;
            }

            if (typeof FitAddonClass === 'function') {
                state.fitAddon = new FitAddonClass();
                state.terminal.loadAddon(state.fitAddon);
            }
        } catch (error) {
            console.warn('FitAddon not available:', error);
        }

        // Open terminal in DOM
        state.terminal.open(terminalElement);

        // Fit terminal to container
        if (state.fitAddon) {
            setTimeout(() => {
                try {
                    state.fitAddon.fit();
                } catch (error) {
                    console.warn('Error fitting terminal:', error);
                }
            }, 100);
        }

        // Handle window resize
        const resizeHandler = () => {
            if (state.terminal && state.currentTab === 'terminal' && state.fitAddon) {
                try {
                    state.fitAddon.fit();
                } catch (error) {
                    console.warn('Error fitting terminal:', error);
                }
            }
        };
        window.addEventListener('resize', resizeHandler);

        // Initialize WebSocket connection
        initTerminalWebSocket();

        // Handle terminal input (keyboard)
        state.terminal.onData(data => {
            if (state.ws && state.ws.readyState === WebSocket.OPEN) {
                const encoder = new TextEncoder();
                state.ws.send(encoder.encode(data));
            }
        });

        // Handle terminal resize events
        state.terminal.onResize((size) => {
            if (state.ws && state.ws.readyState === WebSocket.OPEN) {
                // Send resize message to PTY backend as control message (not displayed to user)
                const msg = `RESIZE${size.cols}x${size.rows}`;
                const encoder = new TextEncoder();
                const resizeData = new Uint8Array(msg.length);
                for (let i = 0; i < msg.length; i++) {
                    resizeData[i] = msg.charCodeAt(i);
                }
                state.ws.send(resizeData);
            }
        });

        // Setup terminal control buttons
        document.getElementById('terminalClearBtn')?.addEventListener('click', () => {
            if (state.terminal) {
                state.terminal.clear();
            }
        });

        document.getElementById('terminalCopyBtn')?.addEventListener('click', () => {
            if (state.terminal) {
                const content = state.terminal.getSelection() || '';
                if (content) {
                    navigator.clipboard.writeText(content).then(() => {
                        console.log('Terminal content copied to clipboard');
                    }).catch(error => {
                        console.error('Failed to copy terminal content:', error);
                    });
                }
            }
        });

        // Theme switching support
        const themeObserver = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
                    const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
                    if (state.terminal) {
                        state.terminal.options.theme = isDark ? darkTheme : lightTheme;
                    }
                }
            });
        });
        themeObserver.observe(document.documentElement, { attributes: true });

        // Store cleanup function
        window.terminalCleanup = () => {
            themeObserver.disconnect();
            window.removeEventListener('resize', resizeHandler);
            if (state.ws) {
                state.ws.close();
            }
            if (state.terminal) {
                state.terminal.dispose();
            }
        };

    } catch (error) {
        console.error('Failed to initialize terminal:', error);
        const terminalDiv = document.getElementById('terminal');
        if (terminalDiv) {
            terminalDiv.innerHTML = '<div style="padding: 20px; color: #ef4444;">Failed to initialize terminal: ' + error.message + '</div>';
        }
    }
}

function initTerminalWebSocket() {
    // Use the correct WebSocket endpoint: /terminal
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/terminal`;

    try {
        state.ws = new WebSocket(wsUrl);
        state.ws.binaryType = 'arraybuffer';

        state.ws.onopen = () => {
            console.log('Terminal WebSocket connected to /terminal');
            updateTerminalConnectionStatus(true);

            // Send initial terminal size if available (as control message, not for display)
            if (state.terminal && state.fitAddon) {
                const size = { cols: state.terminal.cols, rows: state.terminal.rows };
                const msg = `RESIZE${size.cols}x${size.rows}`;
                const resizeData = new Uint8Array(msg.length);
                for (let i = 0; i < msg.length; i++) {
                    resizeData[i] = msg.charCodeAt(i);
                }
                state.ws.send(resizeData);
            }
        };

        state.ws.onmessage = (event) => {
            if (state.terminal && event.data instanceof ArrayBuffer) {
                const decoder = new TextDecoder();
                const text = decoder.decode(new Uint8Array(event.data));
                state.terminal.write(text);
            }
        };

        state.ws.onerror = (error) => {
            console.error('Terminal WebSocket error:', error);
            updateTerminalConnectionStatus(false);
            if (state.terminal) {
                state.terminal.write('\r\n\x1b[31m✗ WebSocket error\x1b[0m\r\n');
            }
        };

        state.ws.onclose = () => {
            console.log('Terminal WebSocket closed');
            updateTerminalConnectionStatus(false);
            if (state.terminal) {
                state.terminal.write('\r\n\x1b[33m✗ Connection closed. Reconnecting...\x1b[0m\r\n');
            }

            // Auto-reconnect after 3 seconds
            setTimeout(() => {
                if (state.currentTab === 'terminal') {
                    initTerminalWebSocket();
                }
            }, 3000);
        };
    } catch (error) {
        console.error('Error creating WebSocket:', error);
        updateTerminalConnectionStatus(false);
    }
}

function updateTerminalConnectionStatus(isConnected) {
    // Update connection status indicator if it exists
    const statusEl = document.getElementById('connectionStatus');
    if (!statusEl) return;

    const indicator = statusEl.querySelector('.status-indicator');
    const text = statusEl.querySelector('.status-text');

    if (isConnected) {
        statusEl.className = 'connection-status connected';
        if (text) text.textContent = 'Connected';
    } else {
        statusEl.className = 'connection-status disconnected';
        if (text) text.textContent = 'Disconnected';
    }
}

// Handle visibility change - reconnect terminal if tab becomes visible
document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible' &&
        state.currentTab === 'terminal' &&
        state.ws &&
        state.ws.readyState !== WebSocket.OPEN) {
        initTerminalWebSocket();
    }
});

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

function updateLastUpdateTime(timestamp) {
    if (!timestamp) {
        timestamp = new Date().toISOString();
    }

    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffHours = diffMs / (1000 * 60 * 60);

    let displayTime;
    if (diffHours < 24) {
        // Show relative time for recent updates
        const diffMins = Math.round(diffMs / (1000 * 60));
        if (diffMins < 1) {
            displayTime = 'Just now';
        } else if (diffMins < 60) {
            displayTime = `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;
        } else {
            const diffHoursRounded = Math.round(diffHours);
            displayTime = `${diffHoursRounded} hour${diffHoursRounded !== 1 ? 's' : ''} ago`;
        }
    } else {
        // Show absolute time for older updates
        displayTime = date.toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
            timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone
        });
    }

    // Update all elements that show last update time
    const projectUpdateEl = document.getElementById('projectLastUpdate');
    if (projectUpdateEl) {
        projectUpdateEl.textContent = `Last updated: ${displayTime}`;
    }

    const machineUpdateEl = document.getElementById('machineLastUpdate');
    if (machineUpdateEl) {
        machineUpdateEl.textContent = `Last updated: ${displayTime}`;
    }
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
        const cost = project.cost !== undefined && project.cost !== null ? project.cost : 0;
        rows.push([
            project.name,
            project.calls,
            project.inputTokens,
            project.outputTokens,
            cost.toFixed(2),
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

    // Initialize theme - ensure dark theme is set
    document.documentElement.setAttribute('data-theme', 'dark');
    console.log('Theme initialized to dark mode');

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

// Chart rendering functions
function updateCharts(chartData) {
    if (!chartData) return;

    if (chartData.cost_over_time && chartData.cost_over_time.length > 0) {
        renderCostOverTimeChart(chartData.cost_over_time);
    }

    if (chartData.cost_by_project && chartData.cost_by_project.length > 0) {
        renderCostByProjectChart(chartData.cost_by_project);
    }

    if (chartData.model_distribution && chartData.model_distribution.length > 0) {
        renderModelDistributionChart(chartData.model_distribution);
    }
}

function renderCostOverTimeChart(data) {
    const container = document.getElementById('costChart');
    if (!container) return;

    container.innerHTML = ''; // Clear

    if (!data || data.length === 0) {
        container.innerHTML = '<p style="color: #94a3b8; text-align: center; padding: 20px;">No data</p>';
        return;
    }

    const margin = {top: 20, right: 30, bottom: 30, left: 60};
    const width = container.clientWidth - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const svg = d3.select(container)
        .append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom)
        .append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleTime()
        .domain(d3.extent(data, d => new Date(d.date)))
        .range([0, width]);

    const y = d3.scaleLinear()
        .domain([0, d3.max(data, d => d.cost)])
        .range([height, 0]);

    const line = d3.line()
        .x(d => x(new Date(d.date)))
        .y(d => y(d.cost));

    svg.append('path')
        .datum(data)
        .attr('fill', 'none')
        .attr('stroke', '#116df8')
        .attr('stroke-width', 2)
        .attr('d', line);

    svg.append('g')
        .attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(x).tickFormat(d3.timeFormat('%m/%d')))
        .style('color', '#94a3b8');

    svg.append('g')
        .call(d3.axisLeft(y))
        .style('color', '#94a3b8');
}

function renderCostByProjectChart(data) {
    const container = document.getElementById('projectCostChart');
    if (!container) return;

    container.innerHTML = ''; // Clear

    if (!data || data.length === 0) {
        container.innerHTML = '<p style="color: #94a3b8; text-align: center; padding: 20px;">No data</p>';
        return;
    }

    const width = container.clientWidth;
    const height = 300;
    const radius = Math.min(width, height) / 2 - 20;

    const svg = d3.select(container)
        .append('svg')
        .attr('width', width)
        .attr('height', height)
        .append('g')
        .attr('transform', `translate(${width / 2},${height / 2})`);

    const color = d3.scaleOrdinal()
        .domain(data.map(d => d.project))
        .range(['#116df8', '#ff5100', '#10b981', '#f59e0b']);

    const pie = d3.pie()
        .value(d => d.cost);

    const arc = d3.arc()
        .innerRadius(0)
        .outerRadius(radius);

    svg.selectAll('path')
        .data(pie(data))
        .enter()
        .append('path')
        .attr('d', arc)
        .attr('fill', d => color(d.data.project))
        .attr('stroke', '#1e293b')
        .attr('stroke-width', 2);
}

function renderModelDistributionChart(data) {
    const container = document.getElementById('modelChart');
    if (!container) return;

    container.innerHTML = ''; // Clear

    if (!data || data.length === 0) {
        container.innerHTML = '<p style="color: #94a3b8; text-align: center; padding: 20px;">No data</p>';
        return;
    }

    const margin = {top: 20, right: 30, bottom: 30, left: 60};
    const width = container.clientWidth - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    const svg = d3.select(container)
        .append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom)
        .append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);

    const x = d3.scaleBand()
        .domain(data.map(d => d.model))
        .range([0, width])
        .padding(0.1);

    const y = d3.scaleLinear()
        .domain([0, d3.max(data, d => d.percentage)])
        .range([height, 0]);

    svg.selectAll('rect')
        .data(data)
        .enter()
        .append('rect')
        .attr('x', d => x(d.model))
        .attr('y', d => y(d.percentage))
        .attr('width', x.bandwidth())
        .attr('height', d => height - y(d.percentage))
        .attr('fill', '#116df8');

    svg.append('g')
        .attr('transform', `translate(0,${height})`)
        .call(d3.axisBottom(x))
        .selectAll('text')
        .attr('transform', 'rotate(-45)')
        .style('text-anchor', 'end')
        .style('color', '#94a3b8')
        .style('font-size', '12px');

    svg.append('g')
        .call(d3.axisLeft(y))
        .style('color', '#94a3b8');
}

// Export for testing
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        formatNumber,
        formatTime,
        escapeHtml,
    };
}
