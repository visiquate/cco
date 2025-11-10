#!/usr/bin/env node

/**
 * Orchestra Configuration Reconciliation Script
 * Applies all phases of the reconciliation plan systematically
 */

const fs = require('fs');
const path = require('path');

// Load configuration
const configPath = path.join(__dirname, '../config/orchestra-config.json');
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

// Track changes
const changes = {
  typeChanges: 0,
  modelChanges: 0,
  agentsAdded: 0,
  errors: []
};

// Phase 3: Critical Type Fixes (23 agents)
const criticalTypeFixes = {
  // TDD Agent
  'TDD Coding Agent': 'test-automator',

  // Security (6)
  'Api Security Audit': 'security-auditor',
  'Penetration Tester': 'security-auditor',
  'Compliance Specialist': 'security-auditor',
  'Mcp Security Auditor': 'security-auditor',
  'Graphql Security Specialist': 'security-auditor',

  // Architecture (4)
  'Nextjs Architecture Expert': 'system-architect',
  'Graphql Architect': 'system-architect',
  'Legacy Modernizer': 'system-architect',
  'Architecture Modernizer': 'system-architect',

  // Infrastructure (7)
  'Cloud Migration Specialist': 'deployment-engineer',
  'Terraform Specialist': 'deployment-engineer',
  'Incident Responder': 'deployment-engineer',
  'Mcp Deployment Orchestrator': 'deployment-engineer',
  'Network Engineer': 'deployment-engineer',
  'Monitoring Specialist': 'deployment-engineer',
  'Devops Troubleshooter': 'deployment-engineer',

  // Other Critical
  'Error Detective': 'debugger',
  'Mcp Testing Engineer': 'test-automator'
};

// Phase 4: High Priority Type Fixes (38 agents)
const highPriorityTypeFixes = {
  // Backend Developers (12)
  'Typescript Pro': 'backend-dev',
  'Javascript Pro': 'backend-dev',
  'Database Admin': 'backend-dev',
  'Database Optimization': 'backend-dev',
  'Database Optimizer': 'backend-dev',
  'Data Engineer': 'backend-dev',
  'Nosql Specialist': 'backend-dev',
  'Sql Pro': 'backend-dev',
  'Mcp Expert': 'backend-dev',
  'Mcp Integration Engineer': 'backend-dev',
  'Ai Engineer': 'backend-dev',
  'Ml Engineer': 'backend-dev',

  // Researchers (11)
  'Research Synthesizer': 'researcher',
  'Research Brief Generator': 'researcher',
  'Comprehensive Researcher': 'researcher',
  'Fact Checker': 'researcher',
  'Search Specialist': 'researcher',
  'Model Evaluator': 'researcher',
  'Business Analyst': 'researcher',
  'Quant Analyst': 'researcher',
  'Document Structure Analyzer': 'researcher',

  // Documentation (3)
  'Changelog Generator': 'technical-writer',
  'Markdown Syntax Formatter': 'technical-writer',
  'Report Generator': 'technical-writer',

  // DevOps (3)
  'Shell Scripting Pro': 'deployment-engineer',
  'Git Flow Manager': 'deployment-engineer',
  'Dependency Manager': 'deployment-engineer',

  // Planners (6)
  'Research Orchestrator': 'planner',
  'Research Coordinator': 'planner',
  'Query Clarifier': 'planner',
  'Product Strategist': 'planner',
  'Project Supervisor Orchestrator': 'planner',
  'Risk Manager': 'planner'
};

// Phase 5: Medium Priority (15 agents)
const mediumPriorityTypeFixes = {
  'Performance Engineer': 'backend-dev',
  'Performance Profiler': 'backend-dev',
  'Cli Ui Designer': 'ux-designer',
  'Frontend Developer': 'backend-dev',
  'Fullstack Developer': 'backend-dev',
  'React Performance Optimization': 'backend-dev',
  'React Performance Optimizer': 'backend-dev',
  'Web Accessibility Checker': 'test-automator',
  'Dx Optimizer': 'deployment-engineer',
  'Command Expert': 'backend-dev',
  'Mcp Protocol Specialist': 'researcher',
  'Prompt Engineer': 'researcher',
  'Content Marketer': 'technical-writer'
};

// Phase 6: Low Priority (5 agents)
const lowPriorityTypeFixes = {
  'Connection Agent': 'backend-dev',
  'Metadata Agent': 'backend-dev',
  'Tag Agent': 'backend-dev',
  'Url Link Extractor': 'researcher',
  'Llms Maintainer': 'deployment-engineer'
};

// Phase 7: Haiku Model Optimization (30 agents)
const haikuAgents = [
  // Documentation (6)
  'Technical Writer',
  'Documentation Expert',
  'API Documenter',
  'Changelog Generator',
  'Report Generator',
  'Markdown Syntax Formatter',

  // Utility (8)
  'Tag Agent',
  'Metadata Agent',
  'URL Link Extractor',
  'Unused Code Cleaner',
  'Document Structure Analyzer',
  'Connection Agent',
  'Command Expert',
  'CLI UI Designer',

  // Research (4)
  'Fact Checker',
  'Query Clarifier',
  'Search Specialist',
  'Research Brief Generator',

  // DevOps (6)
  'Git Flow Manager',
  'Dependency Manager',
  'Dx Optimizer',
  'Monitoring Specialist',

  // Business (3)
  'Business Analyst',
  'Content Marketer',
  'Risk Manager',

  // Other (3)
  'Web Accessibility Checker',
  'Llms Maintainer',
  'Project Supervisor Orchestrator'
];

// Helper function to find agent in config
function findAgent(name) {
  const sections = ['codingAgents', 'integrationAgents', 'developmentAgents',
                    'dataAgents', 'infrastructureAgents', 'securityAgents',
                    'aiMlAgents', 'mcpAgents', 'documentationAgents',
                    'researchAgents', 'supportAgents', 'businessAgents'];

  for (const section of sections) {
    if (config[section]) {
      const agent = config[section].find(a => a.name === name);
      if (agent) return agent;
    }
  }
  return null;
}

// Apply type fixes
function applyTypeFixes(fixes, priority) {
  console.log(`\nApplying ${priority} priority type fixes...`);
  let count = 0;

  for (const [name, newType] of Object.entries(fixes)) {
    const agent = findAgent(name);
    if (agent) {
      if (agent.type !== newType) {
        console.log(`  ${name}: ${agent.type} → ${newType}`);
        agent.type = newType;
        changes.typeChanges++;
        count++;
      }
    } else {
      console.log(`  WARNING: Agent not found: ${name}`);
      changes.errors.push(`Agent not found: ${name}`);
    }
  }

  console.log(`  Changed ${count} agents`);
}

// Apply haiku model optimization
function applyHaikuOptimization() {
  console.log('\nApplying Haiku model optimization...');
  let count = 0;

  for (const name of haikuAgents) {
    const agent = findAgent(name);
    if (agent) {
      if (agent.model !== 'haiku') {
        console.log(`  ${name}: ${agent.model} → haiku`);
        agent.model = 'haiku';

        // Add ccproxyMapping for haiku
        agent.ccproxyMapping = {
          apiAlias: 'claude-3-haiku',
          ollama: 'qwen-fast:latest',
          phase: 'Phase 1 - Lightweight'
        };

        changes.modelChanges++;
        count++;
      }
    } else {
      console.log(`  WARNING: Agent not found: ${name}`);
      changes.errors.push(`Haiku agent not found: ${name}`);
    }
  }

  console.log(`  Changed ${count} agents to haiku`);
}

// Execute all phases
console.log('='.repeat(60));
console.log('ORCHESTRA CONFIGURATION RECONCILIATION');
console.log('='.repeat(60));

// Phase 3: Critical
applyTypeFixes(criticalTypeFixes, 'CRITICAL');

// Phase 4: High Priority
applyTypeFixes(highPriorityTypeFixes, 'HIGH');

// Phase 5: Medium Priority
applyTypeFixes(mediumPriorityTypeFixes, 'MEDIUM');

// Phase 6: Low Priority
applyTypeFixes(lowPriorityTypeFixes, 'LOW');

// Phase 7: Haiku Optimization
applyHaikuOptimization();

// Save updated configuration
fs.writeFileSync(configPath, JSON.stringify(config, null, 2));

// Summary
console.log('\n' + '='.repeat(60));
console.log('RECONCILIATION COMPLETE');
console.log('='.repeat(60));
console.log(`Type changes: ${changes.typeChanges}`);
console.log(`Model changes: ${changes.modelChanges}`);
console.log(`Agents added: ${changes.agentsAdded}`);
console.log(`Errors: ${changes.errors.length}`);

if (changes.errors.length > 0) {
  console.log('\nErrors encountered:');
  changes.errors.forEach(err => console.log(`  - ${err}`));
}

console.log('\nValidating JSON...');
try {
  JSON.parse(fs.readFileSync(configPath, 'utf8'));
  console.log('✓ JSON is valid');
} catch (e) {
  console.error('✗ JSON validation failed:', e.message);
  process.exit(1);
}

console.log('\nConfiguration file updated successfully!');
