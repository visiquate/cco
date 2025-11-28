#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const configPath = '/Users/brent/git/cc-orchestra/config/orchestra-config.json';
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

console.log('=== Verification Report ===\n');

// Check architect
console.log('1. Chief Architect:');
console.log(`   - Has prompt: ${!!config.architect.prompt}`);
console.log(`   - Prompt length: ${config.architect.prompt?.length || 0} characters`);
console.log(`   - First 150 chars: ${config.architect.prompt?.substring(0, 150)}...\n`);

// Check a few sample agents from different sections
const samples = [
  { section: 'codingAgents', name: 'Python Specialist' },
  { section: 'integrationAgents', name: 'API Explorer' },
  { section: 'securityAgents', name: 'Security Auditor' },
  { section: 'documentationAgents', name: 'Documentation Expert' }
];

samples.forEach((sample, idx) => {
  const agent = config[sample.section].find(a => a.name === sample.name);
  console.log(`${idx + 2}. ${sample.name}:`);
  console.log(`   - Has prompt: ${!!agent.prompt}`);
  console.log(`   - Prompt length: ${agent.prompt?.length || 0} characters`);
  console.log(`   - First 150 chars: ${agent.prompt?.substring(0, 150)}...\n`);
});

// Count all agents with prompts
let totalAgents = 1; // architect
let agentsWithPrompts = config.architect.prompt ? 1 : 0;

const sections = [
  'codingAgents', 'integrationAgents', 'developmentAgents', 'dataAgents',
  'infrastructureAgents', 'securityAgents', 'aiMlAgents', 'mcpAgents',
  'documentationAgents', 'researchAgents', 'supportAgents', 'businessAgents'
];

sections.forEach(section => {
  if (config[section]) {
    totalAgents += config[section].length;
    agentsWithPrompts += config[section].filter(a => a.prompt).length;
  }
});

console.log('=== Summary ===');
console.log(`Total agents: ${totalAgents}`);
console.log(`Agents with prompts: ${agentsWithPrompts}`);
console.log(`Coverage: ${((agentsWithPrompts / totalAgents) * 100).toFixed(1)}%`);
