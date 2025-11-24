# Claude Code Integration Guide

## Integrating CCO Agent API with Claude Code

This guide explains how to configure Claude Code to use the CCO Agent Definition API instead of local agent files.

## Environment Setup

### 1. Required Environment Variables
```bash
# ~/.bashrc or ~/.zshrc
export CCO_API_URL="http://localhost:3000"      # CCO server URL
export CCO_API_TIMEOUT="5000"                    # Timeout in milliseconds
export CCO_FALLBACK_TO_FILES="true"             # Use local files as fallback
export CCO_CACHE_AGENTS="true"                  # Cache agent definitions
export CCO_CACHE_TTL="3600"                     # Cache time-to-live in seconds
export CCO_AGENT_DIR="$HOME/.claude/agents"     # Local agent directory
```

### 2. Verify CCO is Running
```bash
# Start CCO if not running
cco run --port 3000 &

# Verify health
curl http://localhost:3000/health | jq

# Verify agent API
curl http://localhost:3000/api/agents/health | jq
```

## JavaScript Client Library

### Installation
```bash
# Create directory for Claude Code integration
mkdir -p ~/.claude/integrations
cd ~/.claude/integrations

# Install dependencies
npm init -y
npm install axios
```

### Client Implementation
```javascript
// ~/.claude/integrations/cco-client.js

const axios = require('axios');
const fs = require('fs');
const path = require('path');
const os = require('os');

class CCOAgentClient {
    constructor(options = {}) {
        this.apiUrl = options.apiUrl || process.env.CCO_API_URL || 'http://localhost:3000';
        this.timeout = options.timeout || parseInt(process.env.CCO_API_TIMEOUT || '5000');
        this.fallbackToFiles = options.fallbackToFiles !== false && process.env.CCO_FALLBACK_TO_FILES !== 'false';
        this.cacheEnabled = options.cacheEnabled !== false && process.env.CCO_CACHE_AGENTS !== 'false';
        this.cacheTTL = (options.cacheTTL || parseInt(process.env.CCO_CACHE_TTL || '3600')) * 1000;
        this.agentDir = options.agentDir || process.env.CCO_AGENT_DIR || path.join(os.homedir(), '.claude', 'agents');

        this.cache = new Map();
        this.cacheTimestamps = new Map();

        // Create axios instance with defaults
        this.client = axios.create({
            baseURL: this.apiUrl,
            timeout: this.timeout,
            headers: {
                'User-Agent': 'Claude-Code-Integration/1.0',
                'Accept': 'application/json'
            }
        });
    }

    /**
     * Get a specific agent definition
     */
    async getAgent(agentType) {
        // Check cache first
        if (this.cacheEnabled) {
            const cached = this.getCached(agentType);
            if (cached) {
                console.log(`[CCO] Using cached agent: ${agentType}`);
                return cached;
            }
        }

        // Try API
        try {
            console.log(`[CCO] Fetching agent from API: ${agentType}`);
            const response = await this.client.get(`/api/agents/${agentType}`);

            if (this.cacheEnabled) {
                this.setCached(agentType, response.data);
            }

            return response.data;
        } catch (error) {
            console.warn(`[CCO] API error for ${agentType}: ${error.message}`);

            // Fallback to local files
            if (this.fallbackToFiles) {
                console.log(`[CCO] Falling back to local file for: ${agentType}`);
                return this.loadLocalAgent(agentType);
            }

            throw error;
        }
    }

    /**
     * List all available agents
     */
    async listAgents() {
        try {
            console.log('[CCO] Listing all agents from API');
            const response = await this.client.get('/api/agents');
            return response.data;
        } catch (error) {
            console.warn(`[CCO] Failed to list agents: ${error.message}`);

            if (this.fallbackToFiles) {
                return this.listLocalAgents();
            }

            throw error;
        }
    }

    /**
     * Search agents by query
     */
    async searchAgents(query) {
        try {
            console.log(`[CCO] Searching agents with query: ${query}`);
            const response = await this.client.get('/api/agents/search', {
                params: { q: query }
            });
            return response.data;
        } catch (error) {
            console.warn(`[CCO] Search failed: ${error.message}`);
            throw error;
        }
    }

    /**
     * Get agents by category
     */
    async getAgentsByCategory(category) {
        try {
            console.log(`[CCO] Getting agents in category: ${category}`);
            const response = await this.client.get(`/api/agents/category/${category}`);
            return response.data;
        } catch (error) {
            console.warn(`[CCO] Category fetch failed: ${error.message}`);
            throw error;
        }
    }

    /**
     * Get model configuration
     */
    async getModelConfig() {
        try {
            console.log('[CCO] Getting model configuration');
            const response = await this.client.get('/api/agents/models');
            return response.data;
        } catch (error) {
            console.warn(`[CCO] Model config fetch failed: ${error.message}`);
            throw error;
        }
    }

    /**
     * Check API health
     */
    async checkHealth() {
        try {
            const response = await this.client.get('/api/agents/health');
            return response.data;
        } catch (error) {
            return {
                status: 'unhealthy',
                error: error.message
            };
        }
    }

    // === Cache Management ===

    getCached(key) {
        if (!this.cache.has(key)) return null;

        const timestamp = this.cacheTimestamps.get(key);
        if (Date.now() - timestamp > this.cacheTTL) {
            this.cache.delete(key);
            this.cacheTimestamps.delete(key);
            return null;
        }

        return this.cache.get(key);
    }

    setCached(key, value) {
        this.cache.set(key, value);
        this.cacheTimestamps.set(key, Date.now());
    }

    clearCache() {
        this.cache.clear();
        this.cacheTimestamps.clear();
    }

    // === Local File Fallback ===

    loadLocalAgent(agentType) {
        const agentPath = path.join(this.agentDir, `${agentType}.md`);

        if (!fs.existsSync(agentPath)) {
            throw new Error(`Local agent file not found: ${agentPath}`);
        }

        const content = fs.readFileSync(agentPath, 'utf8');
        return this.parseLocalAgent(content, agentType);
    }

    parseLocalAgent(content, agentType) {
        const lines = content.split('\n');
        const metadata = {};
        let bodyStart = 0;

        // Parse frontmatter
        if (lines[0] === '---') {
            for (let i = 1; i < lines.length; i++) {
                if (lines[i] === '---') {
                    bodyStart = i + 1;
                    break;
                }
                const [key, ...valueParts] = lines[i].split(': ');
                if (key && valueParts.length > 0) {
                    metadata[key.trim()] = valueParts.join(': ').trim();
                }
            }
        }

        // Determine category based on type
        const category = this.inferCategory(agentType);

        return {
            name: metadata.name || agentType,
            type: agentType,
            model: metadata.model || 'haiku',
            category: category,
            description: metadata.description || '',
            content: lines.slice(bodyStart).join('\n'),
            metadata: {
                version: '1.0.0',
                lastUpdated: new Date().toISOString(),
                tools: (metadata.tools || 'Read, Write, Edit, Bash').split(', ').map(t => t.trim()),
                frontmatter: metadata
            },
            specialties: this.extractSpecialties(content),
            capabilities: this.extractCapabilities(content)
        };
    }

    listLocalAgents() {
        if (!fs.existsSync(this.agentDir)) {
            return {
                version: '2.0.0',
                totalAgents: 0,
                agents: []
            };
        }

        const files = fs.readdirSync(this.agentDir)
            .filter(file => file.endsWith('.md'))
            .map(file => file.slice(0, -3));

        const agents = files.map(agentType => {
            try {
                return this.loadLocalAgent(agentType);
            } catch {
                return null;
            }
        }).filter(Boolean);

        return {
            version: '2.0.0',
            totalAgents: agents.length,
            agents: agents
        };
    }

    inferCategory(agentType) {
        const typeMap = {
            'architect': 'leadership',
            'test': 'testing',
            'qa': 'testing',
            'security': 'security',
            'audit': 'security',
            'doc': 'documentation',
            'writer': 'documentation',
            'devops': 'devops',
            'deploy': 'devops',
            'research': 'research',
            'api': 'integration',
            'integration': 'integration',
            'performance': 'performance',
            'specialist': 'coding',
            'developer': 'coding'
        };

        for (const [keyword, category] of Object.entries(typeMap)) {
            if (agentType.includes(keyword)) {
                return category;
            }
        }

        return 'support';
    }

    extractSpecialties(content) {
        const specialties = [];
        const specialtiesMatch = content.match(/## Specialties\n([\s\S]*?)(?=\n##|\n$)/);

        if (specialtiesMatch) {
            const lines = specialtiesMatch[1].split('\n');
            for (const line of lines) {
                const match = line.match(/^[-*]\s+\*\*(.*?)\*\*/);
                if (match) {
                    specialties.push(match[1]);
                }
            }
        }

        return specialties;
    }

    extractCapabilities(content) {
        const capabilities = [];
        const capMatch = content.match(/## Capabilities\n([\s\S]*?)(?=\n##|\n$)/);

        if (capMatch) {
            const lines = capMatch[1].split('\n');
            for (const line of lines) {
                const match = line.match(/^[-*]\s+(.*?)$/);
                if (match && match[1]) {
                    capabilities.push(match[1].trim());
                }
            }
        }

        return capabilities;
    }
}

// Export for use in Claude Code
module.exports = CCOAgentClient;
```

## Usage Examples

### Basic Usage
```javascript
const CCOAgentClient = require('./cco-client');

async function main() {
    const client = new CCOAgentClient();

    // Get a specific agent
    const pythonAgent = await client.getAgent('python-specialist');
    console.log('Python Agent:', pythonAgent.name);
    console.log('Model:', pythonAgent.model);
    console.log('Specialties:', pythonAgent.specialties);

    // List all agents
    const allAgents = await client.listAgents();
    console.log(`Total agents: ${allAgents.totalAgents}`);
    console.log('Model distribution:', allAgents.modelDistribution);

    // Search for agents
    const searchResults = await client.searchAgents('python');
    console.log(`Found ${searchResults.results} agents matching "python"`);

    // Get agents by category
    const codingAgents = await client.getAgentsByCategory('coding');
    console.log(`${codingAgents.count} coding agents available`);

    // Check health
    const health = await client.checkHealth();
    console.log('API Health:', health.status);
}

main().catch(console.error);
```

### Advanced Usage with Fallback
```javascript
const CCOAgentClient = require('./cco-client');

class AgentManager {
    constructor() {
        this.client = new CCOAgentClient({
            apiUrl: 'http://localhost:3000',
            timeout: 3000,
            fallbackToFiles: true,
            cacheEnabled: true,
            cacheTTL: 1800 // 30 minutes
        });
    }

    async getAgentForTask(taskType) {
        // Map task types to agent types
        const agentMap = {
            'python-api': 'python-specialist',
            'ios-app': 'swift-specialist',
            'flutter-mobile': 'flutter-specialist',
            'security-review': 'security-auditor',
            'test-automation': 'test-engineer',
            'architecture': 'chief-architect'
        };

        const agentType = agentMap[taskType] || 'fullstack-developer';

        try {
            return await this.client.getAgent(agentType);
        } catch (error) {
            console.error(`Failed to get agent for ${taskType}:`, error.message);
            // Return a default agent configuration
            return {
                name: 'Generic Developer',
                type: 'generic',
                model: 'haiku',
                description: 'General purpose development agent',
                content: 'You are a general purpose development agent.',
                metadata: {
                    tools: ['Read', 'Write', 'Edit', 'Bash']
                }
            };
        }
    }

    async selectAgentsForProject(requirements) {
        const selectedAgents = [];

        // Always include architect for complex projects
        if (requirements.complexity === 'high') {
            selectedAgents.push(await this.getAgentForTask('architecture'));
        }

        // Add language-specific agents
        for (const lang of requirements.languages || []) {
            const agentType = `${lang.toLowerCase()}-specialist`;
            try {
                const agent = await this.client.getAgent(agentType);
                selectedAgents.push(agent);
            } catch {
                console.log(`No specialist found for ${lang}`);
            }
        }

        // Add testing agent if needed
        if (requirements.testing) {
            selectedAgents.push(await this.getAgentForTask('test-automation'));
        }

        // Add security agent for production systems
        if (requirements.production) {
            selectedAgents.push(await this.getAgentForTask('security-review'));
        }

        return selectedAgents;
    }
}

// Example usage
async function orchestrateProject() {
    const manager = new AgentManager();

    const projectRequirements = {
        complexity: 'high',
        languages: ['Python', 'Swift'],
        testing: true,
        production: true
    };

    const agents = await manager.selectAgentsForProject(projectRequirements);

    console.log('Selected agents for project:');
    agents.forEach(agent => {
        console.log(`- ${agent.name} (${agent.model}): ${agent.description}`);
    });
}

orchestrateProject().catch(console.error);
```

## Troubleshooting

### Common Issues and Solutions

#### 1. CCO Server Not Running
```bash
# Check if CCO is running
ps aux | grep cco

# Start CCO
cco run --port 3000 &

# Verify it's listening
netstat -an | grep 3000
```

#### 2. API Connection Failures
```javascript
// Test connection
const client = new CCOAgentClient();
const health = await client.checkHealth();

if (health.status !== 'healthy') {
    console.error('CCO API is not healthy:', health.error);
    // Fallback to local files will be automatic if enabled
}
```

#### 3. Cache Issues
```javascript
// Clear cache if getting stale data
client.clearCache();

// Disable cache temporarily
const client = new CCOAgentClient({ cacheEnabled: false });
```

#### 4. Local File Fallback Not Working
```bash
# Verify agent files exist
ls -la ~/.claude/agents/*.md

# Check permissions
chmod -R u+r ~/.claude/agents/
```

## Performance Optimization

### Caching Strategy
```javascript
// Aggressive caching for production
const client = new CCOAgentClient({
    cacheEnabled: true,
    cacheTTL: 7200 // 2 hours
});

// Pre-load frequently used agents
async function preloadAgents() {
    const commonAgents = [
        'python-specialist',
        'swift-specialist',
        'test-engineer',
        'security-auditor'
    ];

    for (const agentType of commonAgents) {
        await client.getAgent(agentType);
    }
}
```

### Parallel Fetching
```javascript
// Fetch multiple agents in parallel
async function getMultipleAgents(agentTypes) {
    const promises = agentTypes.map(type => client.getAgent(type));
    return await Promise.all(promises);
}

const agents = await getMultipleAgents([
    'python-specialist',
    'swift-specialist',
    'test-engineer'
]);
```

## Monitoring

### Health Check Script
```javascript
// health-check.js
const CCOAgentClient = require('./cco-client');

async function healthCheck() {
    const client = new CCOAgentClient();

    console.log('CCO Agent API Health Check');
    console.log('==========================');

    // Check API health
    const health = await client.checkHealth();
    console.log(`Status: ${health.status}`);
    console.log(`Agents loaded: ${health.agents_loaded}`);
    console.log(`Version: ${health.version}`);

    // Test critical endpoints
    try {
        await client.listAgents();
        console.log('✅ List agents: OK');
    } catch {
        console.log('❌ List agents: FAILED');
    }

    try {
        await client.getAgent('python-specialist');
        console.log('✅ Get specific agent: OK');
    } catch {
        console.log('❌ Get specific agent: FAILED');
    }

    try {
        await client.searchAgents('test');
        console.log('✅ Search agents: OK');
    } catch {
        console.log('❌ Search agents: FAILED');
    }
}

healthCheck().catch(console.error);
```

### Usage Metrics
```javascript
// Track API usage
class MetricsClient extends CCOAgentClient {
    constructor(options) {
        super(options);
        this.metrics = {
            apiCalls: 0,
            cacheHits: 0,
            fallbacks: 0,
            errors: 0
        };
    }

    async getAgent(agentType) {
        this.metrics.apiCalls++;

        try {
            if (this.getCached(agentType)) {
                this.metrics.cacheHits++;
            }
            return await super.getAgent(agentType);
        } catch (error) {
            this.metrics.errors++;
            if (this.fallbackToFiles) {
                this.metrics.fallbacks++;
            }
            throw error;
        }
    }

    getMetrics() {
        return {
            ...this.metrics,
            cacheHitRate: this.metrics.cacheHits / this.metrics.apiCalls,
            errorRate: this.metrics.errors / this.metrics.apiCalls,
            fallbackRate: this.metrics.fallbacks / this.metrics.apiCalls
        };
    }
}
```

## Migration Checklist

- [ ] Install CCO and verify it's running
- [ ] Set environment variables
- [ ] Install Node.js client library
- [ ] Test API connectivity
- [ ] Verify fallback mechanism
- [ ] Test with sample agents
- [ ] Monitor performance
- [ ] Document any custom configurations