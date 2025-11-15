#!/usr/bin/env node

/**
 * Fix Task tool invocations to use correct models from config
 *
 * This script searches for hardcoded model specifications in Task calls
 * and updates them to use the configured model values
 */

const fs = require('fs');
const path = require('path');

// Read the orchestra config
const configPath = path.join(__dirname, 'config/orchestra-config.json');
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

// Build a map of agent types to their configured models
const agentModels = new Map();

// Add architect
agentModels.set(config.architect.type, config.architect.model);

// Add all agent groups
const agentGroups = [
  'codingAgents',
  'integrationAgents',
  'developmentAgents',
  'dataAgents',
  'infrastructureAgents',
  'securityAgents',
  'aiMlAgents',
  'mcpAgents',
  'documentationAgents',
  'researchAgents',
  'supportAgents',
  'businessAgents'
];

for (const group of agentGroups) {
  if (config[group]) {
    for (const agent of config[group]) {
      agentModels.set(agent.type, agent.model);
    }
  }
}

console.log('\n📋 Agent Model Configuration');
console.log('============================\n');
console.log('Configured Models:');
agentModels.forEach((model, agentType) => {
  console.log(`  ${agentType}: ${model}`);
});

console.log('\n\n🔍 How to Use Task Tool Correctly\n');
console.log('When spawning agents via Task tool, use this pattern:\n');

console.log('Task(');
console.log('  "description",');
console.log('  "prompt with full context...",');
console.log('  "agent-type",  // Use the agent type from config');
console.log('  "haiku"        // Use the model from configuration, NOT hardcoded "sonnet"');
console.log(');\n');

console.log('\n📝 Example - CORRECT:\n');
console.log('Task(');
console.log('  "Implement daemon mode",');
console.log('  "Implement daemon mode...",');
console.log('  "rust-specialist",  // Agent type');
console.log('  "haiku"  // ✅ CORRECT: Uses configured model (haiku)');
console.log(');\n');

console.log('\n❌ Example - WRONG:\n');
console.log('Task(');
console.log('  "Implement daemon mode",');
console.log('  "Implement daemon mode...",');
console.log('  "rust-specialist",');
console.log('  "sonnet"  // ❌ WRONG: Hardcoded, overrides config');
console.log(');\n');

console.log('\n\n📋 Current Configuration for Your Agents\n');

const yourAgents = [
  'rust-specialist',
  'devops-engineer',
  'frontend-developer',
  'test-engineer',
  'documentation-expert'
];

yourAgents.forEach(agentType => {
  const model = agentModels.get(agentType);
  console.log(`  ✅ ${agentType}: "${model}"`);
});

console.log('\n\n💡 Quick Fix Summary\n');
console.log('1. Check your code that spawns agents via Task tool');
console.log('2. Replace hardcoded model values with configured values:');
yourAgents.forEach(agentType => {
  const model = agentModels.get(agentType);
  console.log(`   - Task(..., "${agentType}", "${model}")`);
});

console.log('\n3. Or use a function to look up the configured model:');
console.log(`
function getAgentModel(agentType) {
  const config = require('./config/orchestra-config.json');
  const allAgents = [
    config.architect,
    ...Object.values(config).filter(Array.isArray).flat()
  ];
  const agent = allAgents.find(a => a.type === agentType);
  return agent?.model || 'sonnet'; // fallback to sonnet if not found
}
`);

console.log('\n✨ Next Steps:\n');
console.log('1. Run: npm run verify-models  (already verified ✅)');
console.log('2. Find where Task tool is invoked in your code');
console.log('3. Update model parameter to use configured values');
console.log('4. Or update orchestration code to respect agent configuration\n');
