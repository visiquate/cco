#!/usr/bin/env node

/**
 * Orchestra Configuration Sync & Maintenance Script
 *
 * Purpose: Ensures the orchestra-config.json stays in sync with agent files
 *
 * Features:
 * - Detects new agent files in ~/.claude/agents/
 * - Validates existing agent type assignments
 * - Checks for duplicates
 * - Monitors model distribution balance
 * - Suggests proper section placement for new agents
 * - Generates maintenance reports
 */

const fs = require('fs');
const path = require('path');

// Paths
const AGENT_DIR = path.join(process.env.HOME, '.claude/agents');
const CONFIG_PATH = path.join(__dirname, '../config/orchestra-config.json');
const BACKUP_DIR = path.join(__dirname, '../config/backups');

// Color output
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m'
};

function log(msg, color = 'reset') {
  console.log(`${colors[color]}${msg}${colors.reset}`);
}

// Load configuration
function loadConfig() {
  try {
    const content = fs.readFileSync(CONFIG_PATH, 'utf8');
    return JSON.parse(content);
  } catch (err) {
    log(`Error loading config: ${err.message}`, 'red');
    process.exit(1);
  }
}

// Get all agent files
function getAgentFiles() {
  try {
    return fs.readdirSync(AGENT_DIR)
      .filter(f => f.endsWith('.md'))
      .map(f => f.replace('.md', ''))
      .sort();
  } catch (err) {
    log(`Error reading agent directory: ${err.message}`, 'red');
    process.exit(1);
  }
}

// Extract agents from config
function extractConfigAgents(config) {
  const sections = [
    'architect', 'codingAgents', 'integrationAgents', 'developmentAgents',
    'dataAgents', 'infrastructureAgents', 'securityAgents', 'aiMlAgents',
    'mcpAgents', 'documentationAgents', 'researchAgents', 'supportAgents', 'businessAgents'
  ];

  const agents = [];
  const typeUsage = {};
  const nameUsage = {};

  sections.forEach(section => {
    const sectionAgents = Array.isArray(config[section]) ? config[section] : [config[section]];
    sectionAgents.forEach(agent => {
      if (agent && agent.name) {
        agents.push({
          name: agent.name,
          type: agent.type,
          model: agent.model,
          section: section
        });

        // Track type usage
        typeUsage[agent.type] = (typeUsage[agent.type] || 0) + 1;

        // Track name usage (for duplicates)
        nameUsage[agent.name] = (nameUsage[agent.name] || 0) + 1;
      }
    });
  });

  return { agents, typeUsage, nameUsage };
}

// Analyze agent file metadata
function analyzeAgentFile(agentType) {
  const filePath = path.join(AGENT_DIR, `${agentType}.md`);
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    const lines = content.split('\n').slice(0, 10);

    // Extract frontmatter
    const nameMatch = lines.find(l => l.startsWith('name:'));
    const descMatch = lines.find(l => l.startsWith('description:'));
    const modelMatch = lines.find(l => l.startsWith('model:'));

    return {
      name: nameMatch ? nameMatch.split(':')[1].trim() : null,
      description: descMatch ? descMatch.split(':')[1].trim() : null,
      suggestedModel: modelMatch ? modelMatch.split(':')[1].trim() : null
    };
  } catch (err) {
    return { name: null, description: null, suggestedModel: null };
  }
}

// Suggest section for new agent
function suggestSection(agentType, metadata) {
  const type = agentType.toLowerCase();
  const desc = (metadata.description || '').toLowerCase();

  // Section mapping logic
  if (type.includes('architect') || type.includes('chief')) return 'architect';
  if (type.includes('coder') || type.includes('coding')) return 'codingAgents';
  if (type.includes('api') && (type.includes('salesforce') || type.includes('authentik'))) return 'integrationAgents';
  if (type.includes('frontend') || type.includes('backend') || type.includes('fullstack')) return 'developmentAgents';
  if (type.includes('mobile') || type.includes('ios') || type.includes('flutter') || type.includes('swift')) return 'developmentAgents';
  if (type.includes('typescript') || type.includes('javascript') || type.includes('python') || type.includes('golang') || type.includes('rust')) return 'developmentAgents';
  if (type.includes('database') || type.includes('data-engineer') || type.includes('data-scientist')) return 'dataAgents';
  if (type.includes('devops') || type.includes('cloud') || type.includes('terraform') || type.includes('deployment')) return 'infrastructureAgents';
  if (type.includes('security') || type.includes('penetration') || type.includes('compliance')) return 'securityAgents';
  if (type.includes('ai-') || type.includes('ml-') || type.includes('llm') || type.includes('model-')) return 'aiMlAgents';
  if (type.includes('mcp-')) return 'mcpAgents';
  if (type.includes('documentation') || type.includes('technical-writer') || type.includes('api-documenter')) return 'documentationAgents';
  if (type.includes('research') || type.includes('fact-') || type.includes('query-') || type.includes('search-')) return 'researchAgents';
  if (type.includes('business') || type.includes('product') || type.includes('content-') || type.includes('quant-')) return 'businessAgents';

  // Default to support
  return 'supportAgents';
}

// Main sync function
function syncConfig() {
  log('\n=== Orchestra Config Sync & Maintenance ===\n', 'cyan');

  const config = loadConfig();
  const agentFiles = getAgentFiles();
  const { agents, typeUsage, nameUsage } = extractConfigAgents(config);

  // Get unique types used in config
  const configuredTypes = [...new Set(agents.map(a => a.type))].sort();

  log(`📊 Summary:`, 'blue');
  log(`  Agent files: ${agentFiles.length}`);
  log(`  Config agents: ${agents.length}`);
  log(`  Unique types: ${configuredTypes.length}\n`);

  // 1. Find missing agents (in files but not in config)
  const missingAgents = agentFiles.filter(type => !configuredTypes.includes(type));

  if (missingAgents.length > 0) {
    log(`🆕 Missing Agents (${missingAgents.length}):`, 'yellow');
    missingAgents.forEach(type => {
      const metadata = analyzeAgentFile(type);
      const section = suggestSection(type, metadata);
      const model = metadata.suggestedModel || 'sonnet';

      log(`  • ${type}`, 'yellow');
      log(`    Suggested section: ${section}`);
      log(`    Suggested model: ${model}`);
      log(`    Description: ${metadata.description || 'N/A'}\n`);
    });
  } else {
    log(`✅ No missing agents\n`, 'green');
  }

  // 2. Find unused types (in config but not in files)
  const unusedTypes = configuredTypes.filter(type => !agentFiles.includes(type));

  if (unusedTypes.length > 0) {
    log(`⚠️  Unused Types (${unusedTypes.length}):`, 'red');
    unusedTypes.forEach(type => {
      const using = agents.filter(a => a.type === type);
      log(`  • ${type} (used by ${using.length} agents)`);
      using.forEach(a => log(`    - ${a.name} in ${a.section}`));
    });
    log('');
  } else {
    log(`✅ No unused types\n`, 'green');
  }

  // 3. Check for duplicate names
  const duplicates = Object.entries(nameUsage).filter(([name, count]) => count > 1);

  if (duplicates.length > 0) {
    log(`🔴 Duplicate Names (${duplicates.length}):`, 'red');
    duplicates.forEach(([name, count]) => {
      log(`  • "${name}" appears ${count} times`);
      const instances = agents.filter(a => a.name === name);
      instances.forEach(a => log(`    - ${a.section} (type: ${a.type})`));
    });
    log('');
  } else {
    log(`✅ No duplicate names\n`, 'green');
  }

  // 4. Check model distribution
  const modelDist = {
    'opus': 0,
    'opus-4.1': 0,
    'sonnet': 0,
    'sonnet-4.5': 0,
    'haiku': 0,
    'haiku-4.5': 0,
    'other': 0
  };

  agents.forEach(a => {
    if (a.model in modelDist) {
      modelDist[a.model]++;
    } else {
      modelDist.other++;
    }
  });

  const opusTotal = modelDist.opus + modelDist['opus-4.1'];
  const sonnetTotal = modelDist.sonnet + modelDist['sonnet-4.5'];
  const haikuTotal = modelDist.haiku + modelDist['haiku-4.5'];

  log(`📈 Model Distribution:`, 'blue');
  log(`  Opus: ${opusTotal} (${(opusTotal / agents.length * 100).toFixed(1)}%) - Target: 1`);
  log(`  Sonnet: ${sonnetTotal} (${(sonnetTotal / agents.length * 100).toFixed(1)}%) - Target: ~66%`);
  log(`  Haiku: ${haikuTotal} (${(haikuTotal / agents.length * 100).toFixed(1)}%) - Target: ~32%`);

  if (modelDist.other > 0) {
    log(`  Other: ${modelDist.other}`, 'yellow');
  }
  log('');

  // 5. Type usage analysis
  const heavyTypes = Object.entries(typeUsage).filter(([type, count]) => count > 5).sort((a, b) => b[1] - a[1]);

  if (heavyTypes.length > 0) {
    log(`📌 Heavily Used Types (>5 instances):`, 'blue');
    heavyTypes.forEach(([type, count]) => {
      log(`  • ${type}: ${count} agents`);
    });
    log('');
  }

  // 6. Generate recommendations
  log(`💡 Recommendations:\n`, 'cyan');

  if (missingAgents.length > 0) {
    log(`  1. Add ${missingAgents.length} new agents to config`);
  }

  if (unusedTypes.length > 0) {
    log(`  2. Fix ${unusedTypes.length} agents using non-existent types`);
  }

  if (duplicates.length > 0) {
    log(`  3. Resolve ${duplicates.length} duplicate agent names`);
  }

  if (opusTotal > 1) {
    log(`  4. Too many Opus agents (${opusTotal}), should be 1`);
  }

  if (Math.abs(sonnetTotal - agents.length * 0.66) > agents.length * 0.05) {
    log(`  5. Sonnet distribution off target (${sonnetTotal} vs ~${Math.round(agents.length * 0.66)})`);
  }

  if (missingAgents.length === 0 && unusedTypes.length === 0 && duplicates.length === 0) {
    log(`  ✅ Configuration is in good shape!`, 'green');
  }

  log('\n');
}

// Run sync
if (require.main === module) {
  syncConfig();
}

module.exports = { syncConfig };
