#!/usr/bin/env node

/**
 * Agent Loader - Fetch agent model from CCO API
 *
 * Usage: node agent-loader.js <agent-name>
 *
 * Environment variables:
 *   CCO_API_URL - Base URL for CCO API (default: http://localhost:3000/api)
 *
 * Example:
 *   node agent-loader.js rust-specialist
 *   # Output: haiku
 *
 * Integration with orchestra:
 *   export CCO_API_URL=http://localhost:3000/api
 *   AGENT_MODEL=$(node agent-loader.js chief-architect)
 *   echo "Agent model: $AGENT_MODEL"
 */

const https = require('https');
const http = require('http');

/**
 * Get the agent model from CCO API
 * @param {string} agentName - The agent name to look up
 * @returns {Promise<string>} The agent model (opus, sonnet, haiku)
 */
async function getAgentModel(agentName) {
  const baseUrl = process.env.CCO_API_URL || 'http://localhost:3000/api';
  const apiUrl = `${baseUrl}/agents/${agentName}`;

  return new Promise((resolve, reject) => {
    // Use WHATWG URL API instead of url.parse()
    let urlObj;
    try {
      urlObj = new URL(apiUrl);
    } catch (e) {
      reject(new Error(`Invalid URL: ${apiUrl}`));
      return;
    }

    const protocol = urlObj.protocol === 'https:' ? https : http;

    const options = {
      hostname: urlObj.hostname,
      port: urlObj.port || (urlObj.protocol === 'https:' ? 443 : 80),
      path: urlObj.pathname + (urlObj.search || ''),
      method: 'GET',
      timeout: 5000,
      headers: {
        'User-Agent': 'agent-loader/1.0'
      }
    };

    const req = protocol.request(options, (res) => {
      let data = '';

      res.on('data', (chunk) => {
        data += chunk;
      });

      res.on('end', () => {
        if (res.statusCode === 200) {
          try {
            const json = JSON.parse(data);
            if (json.model) {
              resolve(json.model);
            } else {
              reject(new Error(`Agent ${agentName} has no model field`));
            }
          } catch (e) {
            reject(new Error(`Invalid JSON response: ${e.message}`));
          }
        } else if (res.statusCode === 404) {
          reject(new Error(`Agent ${agentName} not found`));
        } else {
          reject(new Error(`HTTP ${res.statusCode}: ${data}`));
        }
      });
    });

    req.on('error', (e) => {
      reject(e);
    });

    req.on('timeout', () => {
      req.abort();
      reject(new Error('Request timeout'));
    });

    req.end();
  });
}

/**
 * Main entry point
 */
async function main() {
  // Check if agent name provided
  if (process.argv.length < 3) {
    console.error('Usage: node agent-loader.js <agent-name>');
    console.error('');
    console.error('Examples:');
    console.error('  node agent-loader.js chief-architect');
    console.error('  node agent-loader.js rust-specialist');
    console.error('');
    console.error('Environment variables:');
    console.error('  CCO_API_URL - Base URL for CCO API');
    process.exit(1);
  }

  const agentName = process.argv[2];

  try {
    const model = await getAgentModel(agentName);
    console.log(model);
    process.exit(0);
  } catch (error) {
    console.error(`Error: ${error.message}`, file=process.stderr);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

module.exports = { getAgentModel };
