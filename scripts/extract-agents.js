#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const agentsDir = path.join(process.env.HOME, '.claude/agents');
const files = fs.readdirSync(agentsDir).filter(f => f.endsWith('.md'));

const agents = files.map(file => {
  const content = fs.readFileSync(path.join(agentsDir, file), 'utf8');
  const lines = content.split('\n');

  let name = '';
  let description = '';
  let model = 'sonnet';
  let inFrontmatter = false;

  for (let i = 0; i < Math.min(lines.length, 20); i++) {
    const line = lines[i].trim();

    if (line === '---') {
      inFrontmatter = !inFrontmatter;
      continue;
    }

    if (inFrontmatter) {
      if (line.startsWith('name:')) {
        name = line.substring(5).trim();
      } else if (line.startsWith('description:')) {
        description = line.substring(12).trim();
      } else if (line.startsWith('model:')) {
        model = line.substring(6).trim();
      }
    }
  }

  return {
    file,
    name,
    description,
    model
  };
}).filter(agent => agent.name); // Only include agents with names

console.log(JSON.stringify(agents, null, 2));
