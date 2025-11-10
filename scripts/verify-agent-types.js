const fs = require('fs');
const path = require('path');

// Get all actual agent files
const agentDir = path.join(process.env.HOME, '.claude', 'agents');
const agentFiles = fs.readdirSync(agentDir)
  .filter(f => f.endsWith('.md') && f !== 'agent-overview.md')
  .map(f => f.replace('.md', ''));

console.log(`Found ${agentFiles.length} agent files in ${agentDir}\n`);

// Load config
const config = require('../config/orchestra-config.json');

// Get all unique types used
const typesUsed = new Set();
const typeAgents = {};

Object.entries(config).forEach(([section, agents]) => {
  if(Array.isArray(agents)) {
    agents.forEach(a => {
      const type = a.type || 'undefined';
      typesUsed.add(type);
      if(!typeAgents[type]) typeAgents[type] = [];
      typeAgents[type].push({name: a.name, section});
    });
  }
});

console.log('=== TYPE VALIDATION ===\n');

// Check each type
const missingTypes = [];
const validTypes = [];

Array.from(typesUsed).sort().forEach(type => {
  const exists = agentFiles.includes(type);
  const count = typeAgents[type].length;

  if(exists) {
    validTypes.push({type, count});
    console.log(`✅ ${type} (${count} agents) - EXISTS`);
  } else {
    missingTypes.push({type, count, agents: typeAgents[type]});
    console.log(`❌ ${type} (${count} agents) - MISSING`);
  }
});

console.log('\n=== MISSING TYPES DETAILS ===\n');

if(missingTypes.length === 0) {
  console.log('✅ All types exist as agent files!');
} else {
  missingTypes.forEach(({type, count, agents}) => {
    console.log(`\n❌ ${type} (${count} agents):`);
    agents.slice(0, 5).forEach(a => console.log(`   - ${a.name}`));
    if(agents.length > 5) console.log(`   ... and ${agents.length - 5} more`);

    // Suggest replacement
    console.log(`   Suggested replacement: ${suggestReplacement(type, agentFiles)}`);
  });
}

console.log('\n=== SUMMARY ===\n');
console.log(`Total unique types: ${typesUsed.size}`);
console.log(`Valid types: ${validTypes.length}`);
console.log(`Missing types: ${missingTypes.length}`);
console.log(`Total agents: ${Object.values(typeAgents).reduce((sum, arr) => sum + arr.length, 0)}`);

if(missingTypes.length > 0) {
  console.log('\n⚠️  ACTION REQUIRED: Replace missing types with valid agent file names');
  process.exit(1);
}

function suggestReplacement(type, available) {
  const mappings = {
    'backend-dev': 'backend-architect',
    'coder': 'fullstack-developer',
    'planner': 'task-decomposition-expert',
    'python-expert': 'python-pro',
    'researcher': 'technical-researcher',
    'reviewer': 'code-reviewer',
    'system-architect': 'backend-architect',
    'ux-designer': 'ui-ux-designer'
  };

  return mappings[type] || 'unknown';
}
