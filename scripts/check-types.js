const fs = require('fs');
const config = JSON.parse(fs.readFileSync('./config/orchestra-config.json', 'utf8'));

let missing = [];
let allAgents = [];

Object.entries(config).forEach(([section, agents]) => {
  if(Array.isArray(agents)) {
    agents.forEach((a, i) => {
      allAgents.push({section, index: i, name: a.name, type: a.type});
      if(!a.type) {
        missing.push(`${section}[${i}] - ${a.name}`);
      }
    });
  }
});

console.log(`Total agents found: ${allAgents.length}`);
console.log(`Agents missing type field: ${missing.length}\n`);

if(missing.length > 0) {
  console.log('Missing type field:');
  missing.forEach(m => console.log('  ' + m));
} else {
  console.log('✓ All agents have type field');
}

// Count by type
const types = {};
allAgents.forEach(a => {
  const t = a.type || 'undefined';
  types[t] = (types[t] || 0) + 1;
});

console.log('\nType distribution:');
Object.entries(types)
  .sort((a,b) => b[1] - a[1])
  .forEach(([type, count]) => console.log(`  ${type}: ${count}`));
