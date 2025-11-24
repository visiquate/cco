#!/usr/bin/env node

/**
 * LLM Router for Claude Orchestra
 *
 * Routes different types of tasks to appropriate LLM endpoints:
 * - Architecture/Planning → Claude API (via Claude Code)
 * - Coding/Implementation → coder.visiquate.com
 */

const https = require('https');
const http = require('http');
const CredentialManager = require('./credential-manager');

class LLMRouter {
  constructor(config) {
    this.config = config || this.loadDefaultConfig();
    this.endpoints = this.config.llmRouting?.endpoints || {};
    this.routingRules = this.config.llmRouting?.rules || {};
    this.credentialManager = new CredentialManager();
  }

  /**
   * Load default configuration
   */
  loadDefaultConfig() {
    try {
      return require('../config/orchestra-config.json');
    } catch (error) {
      console.error('Failed to load orchestra config:', error.message);
      return { llmRouting: { endpoints: {}, rules: {} } };
    }
  }

  /**
   * Determine which LLM endpoint to use based on task type
   */
  routeTask(agentType, taskType) {
    // Architecture and planning always go to Claude
    if (this.isArchitectureTask(agentType, taskType)) {
      return {
        endpoint: 'claude',
        useClaudeCode: true,
        reason: 'Architecture and planning tasks use Claude'
      };
    }

    // Coding tasks can be routed to custom endpoint
    if (this.isCodingTask(agentType, taskType)) {
      const customEndpoint = this.endpoints.coding;
      if (customEndpoint && customEndpoint.enabled) {
        return {
          endpoint: 'custom',
          url: customEndpoint.url,
          useClaudeCode: false,
          reason: 'Coding tasks routed to custom LLM'
        };
      }
    }

    // Default to Claude via Claude Code
    return {
      endpoint: 'claude',
      useClaudeCode: true,
      reason: 'Default routing to Claude'
    };
  }

  /**
   * Check if task is architecture/planning related
   */
  isArchitectureTask(agentType, taskType) {
    const architectureTypes = [
      'system-architect',
      'architecture',
      'specification',
      'planner'
    ];

    const architectureTasks = [
      'design',
      'architecture',
      'planning',
      'specification',
      'requirements',
      'coordination'
    ];

    return architectureTypes.includes(agentType) ||
           architectureTasks.some(t => taskType?.toLowerCase().includes(t));
  }

  /**
   * Check if task is coding related
   */
  isCodingTask(agentType, taskType) {
    const codingTypes = [
      'python-expert',
      'ios-developer',
      'backend-dev',
      'mobile-developer',
      'coder',
      'frontend-dev',
      'deployment-engineer'  // DevOps implementation tasks
    ];

    const codingTasks = [
      'implement',
      'code',
      'develop',
      'build',
      'write code',
      'programming'
    ];

    return codingTypes.includes(agentType) ||
           codingTasks.some(t => taskType?.toLowerCase().includes(t));
  }

  /**
   * Call custom LLM endpoint
   */
  async callCustomEndpoint(prompt, options = {}) {
    const endpoint = this.endpoints.coding;
    if (!endpoint || !endpoint.enabled) {
      throw new Error('Custom coding endpoint not configured or not enabled');
    }

    const url = new URL(endpoint.url);
    const isHttps = url.protocol === 'https:';
    const client = isHttps ? https : http;

    // Detect if this is an Ollama endpoint and use appropriate format
    const isOllama = endpoint.type === 'ollama' || endpoint.url.includes('ollama') ||
                     endpoint.defaultModel?.includes('qwen') || endpoint.defaultModel?.includes('llama');

    let requestBody, requestPath;

    if (isOllama) {
      // Ollama API format
      requestPath = '/api/generate';
      requestBody = JSON.stringify({
        model: options.model || endpoint.defaultModel || 'qwen2.5-coder:32b-instruct',
        prompt: prompt,
        stream: false,
        options: {
          temperature: options.temperature || endpoint.temperature || 0.7,
          num_predict: options.maxTokens || endpoint.maxTokens || 4096
        }
      });
    } else {
      // Generic OpenAI-compatible format
      requestPath = url.pathname + url.search;
      requestBody = JSON.stringify({
        prompt: prompt,
        model: options.model || endpoint.defaultModel || 'default',
        temperature: options.temperature || endpoint.temperature || 0.7,
        max_tokens: options.maxTokens || endpoint.maxTokens || 4096,
        ...endpoint.additionalParams
      });
    }

    // Retrieve bearer token from credential manager or environment for coder.visiquate.com
    let bearerToken = null;
    if (url.hostname === 'coder.visiquate.com') {
      // Try environment variable first (more reliable for now)
      bearerToken = process.env.CODER_LLM_TOKEN;

      // Fallback to credential manager if environment variable not set
      if (!bearerToken) {
        try {
          bearerToken = await this.credentialManager.retrieveCredential('CODER_LLM_TOKEN');
        } catch (error) {
          console.warn('Bearer token not found. Set CODER_LLM_TOKEN environment variable or use credential manager.');
        }
      }
    }

    // Build authorization header - prioritize bearer token, then apiKey
    const authHeader = bearerToken
      ? { 'Authorization': `Bearer ${bearerToken}` }
      : (endpoint.apiKey ? { 'Authorization': `Bearer ${endpoint.apiKey}` } : {});

    const requestOptions = {
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: requestPath,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(requestBody),
        ...authHeader,
        ...endpoint.headers
      }
    };

    return new Promise((resolve, reject) => {
      const req = client.request(requestOptions, (res) => {
        let data = '';

        res.on('data', (chunk) => {
          data += chunk;
        });

        res.on('end', () => {
          if (res.statusCode >= 200 && res.statusCode < 300) {
            try {
              const response = JSON.parse(data);

              // Extract response text based on format
              if (isOllama && response.response) {
                // Ollama format: response is in 'response' field
                resolve({
                  text: response.response,
                  model: response.model,
                  raw: response
                });
              } else if (response.choices && response.choices[0]) {
                // OpenAI format: response is in choices[0].text or message.content
                resolve({
                  text: response.choices[0].text || response.choices[0].message?.content,
                  model: response.model,
                  raw: response
                });
              } else {
                // Return raw response if format unknown
                resolve({
                  text: JSON.stringify(response),
                  raw: response
                });
              }
            } catch (error) {
              reject(new Error(`Failed to parse response: ${error.message}`));
            }
          } else {
            reject(new Error(`HTTP ${res.statusCode}: ${data}`));
          }
        });
      });

      req.on('error', (error) => {
        reject(new Error(`Request failed: ${error.message}`));
      });

      req.write(requestBody);
      req.end();
    });
  }

  /**
   * Generate agent instructions with appropriate endpoint routing
   */
  generateRoutedInstructions(agent, requirement, taskType = 'default') {
    const routing = this.routeTask(agent.type, taskType);

    if (routing.useClaudeCode) {
      // Standard Claude Code Task tool usage
      return {
        method: 'claudeCode',
        routing: routing,
        agent: agent,
        requirement: requirement
      };
    } else {
      // Custom endpoint - provide instructions for manual calling
      return {
        method: 'custom',
        routing: routing,
        agent: agent,
        requirement: requirement,
        instructions: `This coding task should use the custom LLM at ${routing.url}.
Use the LLM router to call:
node src/llm-router.js call-coding-llm "${requirement}"`
      };
    }
  }

  /**
   * Get routing statistics
   */
  getRoutingStats() {
    return {
      endpoints: Object.keys(this.endpoints).map(key => ({
        name: key,
        enabled: this.endpoints[key].enabled,
        url: this.endpoints[key].url
      })),
      rules: this.routingRules,
      architectureTasks: 'Always route to Claude',
      codingTasks: this.endpoints.coding?.enabled ?
        `Route to ${this.endpoints.coding.url}` :
        'Route to Claude (custom endpoint not configured)'
    };
  }
}

// CLI usage
if (require.main === module) {
  const router = new LLMRouter();

  const command = process.argv[2];

  if (command === 'stats') {
    console.log('LLM Routing Configuration');
    console.log('========================\n');
    console.log(JSON.stringify(router.getRoutingStats(), null, 2));
  } else if (command === 'route') {
    const agentType = process.argv[3];
    const taskType = process.argv[4];

    if (!agentType) {
      console.error('Usage: node llm-router.js route <agent-type> [task-type]');
      process.exit(1);
    }

    const routing = router.routeTask(agentType, taskType);
    console.log('Routing Decision:');
    console.log(JSON.stringify(routing, null, 2));
  } else if (command === 'call-coding-llm') {
    const prompt = process.argv.slice(3).join(' ');

    if (!prompt) {
      console.error('Usage: node llm-router.js call-coding-llm "<prompt>"');
      process.exit(1);
    }

    console.log('Calling custom coding LLM...');
    router.callCustomEndpoint(prompt)
      .then(response => {
        console.log('\nResponse:');
        console.log(JSON.stringify(response, null, 2));
      })
      .catch(error => {
        console.error('\nError:', error.message);
        process.exit(1);
      });
  } else {
    console.log('LLM Router for Claude Orchestra');
    console.log('==========================\n');
    console.log('Commands:');
    console.log('  stats                           - Show routing configuration');
    console.log('  route <agent-type> [task-type]  - Show routing decision');
    console.log('  call-coding-llm "<prompt>"      - Call custom coding LLM\n');
    console.log('Examples:');
    console.log('  node llm-router.js stats');
    console.log('  node llm-router.js route python-expert implement');
    console.log('  node llm-router.js call-coding-llm "Write a Python function"');
  }
}

module.exports = LLMRouter;
