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

// Fix each agent file
const agentsDir = path.join(process.env.HOME, '.claude/agents');
const agentFiles = fs.readdirSync(agentsDir).filter(f => f.endsWith('.md'));

let fixed = 0;
let added = 0;
let noFrontmatter = 0;

for (const file of agentFiles) {
  const agentType = file.replace('.md', '');
  const filePath = path.join(agentsDir, file);
  let content = fs.readFileSync(filePath, 'utf8');

  const expectedModel = expectedModels.get(agentType);
  if (!expectedModel) {
    console.log(`⚠️  Skipping ${file} - not in config`);
    continue;
  }

  // Check if file has frontmatter
  const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---/);

  if (!frontmatterMatch) {
    console.log(`⚠️  No frontmatter in ${file} - cannot fix automatically`);
    noFrontmatter++;
    continue;
  }

  const frontmatter = frontmatterMatch[1];
  const modelMatch = frontmatter.match(/^model:\s*(.+)$/m);

  if (!modelMatch) {
    // Add model field to frontmatter
    const newFrontmatter = frontmatter + `\nmodel: ${expectedModel}`;
    content = content.replace(frontmatterMatch[0], `---\n${newFrontmatter}\n---`);
    fs.writeFileSync(filePath, content, 'utf8');
    console.log(`✅ Added model field to ${file}: ${expectedModel}`);
    added++;
  } else {
    const actualModel = modelMatch[1].trim();
    const normalizedExpected = expectedModel.toLowerCase();
    const normalizedActual = actualModel.toLowerCase();

    if (normalizedExpected !== normalizedActual) {
      // Replace the model value
      const newFrontmatter = frontmatter.replace(
        /^model:\s*.+$/m,
        `model: ${expectedModel}`
      );
      content = content.replace(frontmatterMatch[0], `---\n${newFrontmatter}\n---`);
      fs.writeFileSync(filePath, content, 'utf8');
      console.log(`✅ Fixed ${file}: ${actualModel} → ${expectedModel}`);
      fixed++;
    }
  }
}

console.log('\n📊 Summary:');
console.log(`  ✅ Fixed model mismatches: ${fixed}`);
console.log(`  ✅ Added missing model fields: ${added}`);
console.log(`  ⚠️  Files without frontmatter (need manual fix): ${noFrontmatter}`);
console.log(`\nTotal changes: ${fixed + added}`);
