#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Read the orchestra config
const configPath = path.join(__dirname, 'config/orchestra-config.json');
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

// Build a map of agent types to their expected models
const expectedModels = new Map();

// Add architect
expectedModels.set(config.architect.type, config.architect.model);

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
      expectedModels.set(agent.type, agent.model);
    }
  }
}

// Check each agent file
const agentsDir = path.join(process.env.HOME, '.claude/agents');
const agentFiles = fs.readdirSync(agentsDir).filter(f => f.endsWith('.md'));

const mismatches = [];
const missing = [];
const noModelInFile = [];

for (const file of agentFiles) {
  const agentType = file.replace('.md', '');
  const filePath = path.join(agentsDir, file);
  const content = fs.readFileSync(filePath, 'utf8');

  // Extract model from frontmatter
  const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);

  if (!frontmatterMatch) {
    console.log(`⚠️  No frontmatter in ${file}`);
    continue;
  }

  const frontmatter = frontmatterMatch[1];
  const modelMatch = frontmatter.match(/^model:\s*(.+)$/m);

  const expectedModel = expectedModels.get(agentType);

  if (!expectedModel) {
    missing.push({ file, agentType });
    continue;
  }

  if (!modelMatch) {
    noModelInFile.push({ file, agentType, expected: expectedModel });
    continue;
  }

  const actualModel = modelMatch[1].trim();

  // Normalize model names for comparison
  const normalizedExpected = expectedModel.toLowerCase();
  const normalizedActual = actualModel.toLowerCase();

  if (normalizedExpected !== normalizedActual) {
    mismatches.push({
      file,
      agentType,
      expected: expectedModel,
      actual: actualModel
    });
  }
}

// Report findings
console.log('\n🔍 Agent Model Verification Report\n');
console.log(`Total agent files: ${agentFiles.length}`);
console.log(`Expected agents in config: ${expectedModels.size}`);
console.log('');

if (mismatches.length > 0) {
  console.log(`❌ Model Mismatches Found: ${mismatches.length}\n`);
  for (const m of mismatches) {
    console.log(`  ${m.file}`);
    console.log(`    Expected: ${m.expected}`);
    console.log(`    Actual:   ${m.actual}`);
    console.log('');
  }
}

if (noModelInFile.length > 0) {
  console.log(`⚠️  Missing Model Field: ${noModelInFile.length}\n`);
  for (const m of noModelInFile) {
    console.log(`  ${m.file}`);
    console.log(`    Expected: ${m.expected}`);
    console.log('');
  }
}

if (missing.length > 0) {
  console.log(`📝 Agent Files Not in Config: ${missing.length}\n`);
  for (const m of missing) {
    console.log(`  ${m.file} (${m.agentType})`);
  }
  console.log('');
}

if (mismatches.length === 0 && noModelInFile.length === 0 && missing.length === 0) {
  console.log('✅ All agent model assignments match the config!\n');
  process.exit(0);
} else {
  console.log(`\n📊 Summary:`);
  console.log(`  ❌ Mismatches: ${mismatches.length}`);
  console.log(`  ⚠️  Missing model field: ${noModelInFile.length}`);
  console.log(`  📝 Not in config: ${missing.length}`);
  process.exit(1);
}
