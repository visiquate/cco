#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Model assignment rules based on agent complexity
const modelAssignments = {
  'opus': ['Chief Architect'], // Strategic leadership only

  'haiku': [
    // Basic language specialists
    'Python Specialist', 'Swift Specialist', 'Go Specialist', 'Rust Specialist', 'Flutter Specialist',
    'Python Pro', 'Typescript Pro', 'Javascript Pro', 'Golang Pro', 'Rust Pro',

    // Simple documentation
    'Documentation Expert', 'Technical Writer', 'Api Documenter', 'Changelog Generator',
    'Markdown Syntax Formatter', 'Report Generator',

    // Lightweight utilities
    'Dx Optimizer', 'Git Flow Manager', 'Dependency Manager', 'Monitoring Specialist',
    'Command Expert', 'Connection Agent', 'Metadata Agent', 'Tag Agent',
    'Document Structure Analyzer', 'Url Link Extractor', 'Project Supervisor Orchestrator',
    'Unused Code Cleaner', 'Cli Ui Designer',

    // Simple research
    'Research Brief Generator', 'Fact Checker', 'Query Clarifier', 'Search Specialist',

    // Business support
    'Business Analyst', 'Content Marketer',

    // Security checks
    'Web Accessibility Checker', 'Risk Manager',

    // ML support
    'Llms Maintainer'
  ],

  'sonnet-4.5': [
    // ALL OTHER AGENTS - Intelligent managers, reviewers, architects, complex coders
    // This is the default for any agent not in opus or haiku lists
  ]
};

function updateConfigModels() {
  const configPath = path.join(__dirname, '../config/orchestra-config.json');
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

  let updateCount = 0;

  // Helper function to update model and remove ccproxy
  function updateAgent(agent) {
    const agentName = agent.name;
    let targetModel = 'sonnet-4.5'; // Default to sonnet-4.5

    if (modelAssignments.opus.includes(agentName)) {
      targetModel = 'opus';
    } else if (modelAssignments.haiku.includes(agentName)) {
      targetModel = 'haiku';
    }

    // Update model if different
    if (agent.model !== targetModel) {
      console.log(`  ${agentName}: ${agent.model} → ${targetModel}`);
      agent.model = targetModel;
      updateCount++;
    }

    // Remove ccproxyMapping
    if (agent.ccproxyMapping) {
      delete agent.ccproxyMapping;
      console.log(`  ${agentName}: Removed ccproxyMapping`);
    }

    return agent;
  }

  // Update architect
  console.log('\n📐 Updating Chief Architect:');
  config.architect = updateAgent(config.architect);

  // Update all agent categories
  const categories = [
    'codingAgents', 'integrationAgents', 'developmentAgents', 'dataAgents',
    'infrastructureAgents', 'securityAgents', 'aiMlAgents', 'mcpAgents',
    'documentationAgents', 'researchAgents', 'supportAgents', 'businessAgents'
  ];

  categories.forEach(category => {
    if (config[category]) {
      console.log(`\n📦 Updating ${category}:`);
      config[category] = config[category].map(updateAgent);
    }
  });

  // Remove llmRouting section (ccproxy not implemented yet)
  if (config.llmRouting) {
    console.log('\n🗑️  Removing llmRouting section (ccproxy future plans)');
    delete config.llmRouting;
  }

  // Write updated config
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));

  console.log(`\n✅ Updated ${updateCount} agent model assignments`);
  console.log('✅ Removed all ccproxy references');
  console.log('✅ Config saved to:', configPath);

  // Count agents by model
  const modelCounts = { 'opus': 0, 'sonnet-4.5': 0, 'haiku': 0 };

  modelCounts[config.architect.model]++;
  categories.forEach(category => {
    if (config[category]) {
      config[category].forEach(agent => {
        modelCounts[agent.model]++;
      });
    }
  });

  console.log('\n📊 Agent Model Distribution:');
  console.log(`  Opus 4.1: ${modelCounts.opus} agent(s)`);
  console.log(`  Sonnet 4.5: ${modelCounts['sonnet-4.5']} agents`);
  console.log(`  Haiku 4.5: ${modelCounts.haiku} agents`);
  console.log(`  Total: ${Object.values(modelCounts).reduce((a, b) => a + b, 0)} agents`);
}

// Run the update
try {
  updateConfigModels();
  console.log('\n✨ Configuration update complete!\n');
} catch (error) {
  console.error('❌ Error updating configuration:', error);
  process.exit(1);
}
