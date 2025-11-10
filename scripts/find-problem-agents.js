const fs = require('fs');
const config = JSON.parse(fs.readFileSync('./config/orchestra-config.json', 'utf8'));

console.log('=== PROBLEM AGENTS ===\n');

let problems = [];

Object.entries(config).forEach(([section, agents]) => {
  if(Array.isArray(agents)) {
    agents.forEach((a, i) => {
      if(!a.type) {
        problems.push({
          section,
          index: i,
          name: a.name,
          issue: 'Missing type field'
        });
      } else if(a.type === 'system-architect') {
        problems.push({
          section,
          index: i,
          name: a.name,
          issue: 'Invalid type: system-architect (file does not exist)'
        });
      }
    });
  }
});

if(problems.length === 0) {
  console.log('✅ No problems found!');
} else {
  problems.forEach(p => {
    console.log(`❌ ${p.section}[${p.index}]: ${p.name}`);
    console.log(`   Issue: ${p.issue}\n`);
  });

  console.log(`Total problems: ${problems.length}`);
}
