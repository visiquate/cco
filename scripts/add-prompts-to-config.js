#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Paths
const configPath = '/Users/brent/git/cc-orchestra/config/orchestra-config.json';
const agentsDir = path.join(process.env.HOME, '.claude', 'agents');
const outputPath = configPath; // Overwrite the existing file

// Read the orchestra config
console.log('Reading orchestra-config.json...');
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

// Statistics
const stats = {
  totalAgents: 0,
  agentsWithFiles: 0,
  agentsMissingFiles: 0,
  errors: [],
  missingFiles: []
};

/**
 * Process an agent by reading its markdown file and adding the prompt field
 */
function processAgent(agent) {
  stats.totalAgents++;

  // Skip if no agentFile specified
  if (!agent.agentFile) {
    console.log(`  ⚠️  Agent "${agent.name}" has no agentFile field`);
    stats.agentsMissingFiles++;
    // Use role as fallback
    agent.prompt = agent.role || 'No prompt available';
    return;
  }

  // Resolve the agentFile path (handle ~ prefix)
  let agentFilePath = agent.agentFile;
  if (agentFilePath.startsWith('~')) {
    agentFilePath = agentFilePath.replace('~', process.env.HOME);
  }

  // If it's just a filename, prepend the agents directory
  if (!path.isAbsolute(agentFilePath) && !agentFilePath.includes('/')) {
    agentFilePath = path.join(agentsDir, agentFilePath);
  }

  try {
    // Read the markdown file
    const promptContent = fs.readFileSync(agentFilePath, 'utf8');
    agent.prompt = promptContent;
    stats.agentsWithFiles++;
    console.log(`  ✅ Agent "${agent.name}" - loaded prompt from ${path.basename(agentFilePath)}`);
  } catch (error) {
    console.log(`  ❌ Agent "${agent.name}" - file not found: ${agentFilePath}`);
    stats.missingFiles.push({
      agent: agent.name,
      file: agentFilePath
    });
    stats.agentsMissingFiles++;
    // Use role as fallback
    agent.prompt = agent.role || 'No prompt available';
    stats.errors.push({
      agent: agent.name,
      error: error.message
    });
  }
}

/**
 * Process agent arrays in the config
 */
function processAgentSection(sectionName, agents) {
  if (!agents || !Array.isArray(agents)) return;

  console.log(`\nProcessing ${sectionName} (${agents.length} agents)...`);
  agents.forEach(processAgent);
}

/**
 * Main execution
 */
function main() {
  console.log('=== Adding Prompts to Orchestra Config ===\n');

  // Process the architect (single agent, not array)
  console.log('Processing architect...');
  if (config.architect) {
    // The architect doesn't have an agentFile in the config, so we'll add it
    if (!config.architect.agentFile) {
      config.architect.agentFile = '~/.claude/agents/chief-architect.md';
    }
    processAgent(config.architect);
  }

  // Process all agent sections
  processAgentSection('codingAgents', config.codingAgents);
  processAgentSection('integrationAgents', config.integrationAgents);
  processAgentSection('developmentAgents', config.developmentAgents);
  processAgentSection('dataAgents', config.dataAgents);
  processAgentSection('infrastructureAgents', config.infrastructureAgents);
  processAgentSection('securityAgents', config.securityAgents);
  processAgentSection('aiMlAgents', config.aiMlAgents);
  processAgentSection('mcpAgents', config.mcpAgents);
  processAgentSection('documentationAgents', config.documentationAgents);
  processAgentSection('researchAgents', config.researchAgents);
  processAgentSection('supportAgents', config.supportAgents);
  processAgentSection('businessAgents', config.businessAgents);

  // Get file sizes
  const originalSize = fs.statSync(configPath).size;

  // Write the updated config
  console.log('\n=== Writing Updated Config ===');
  const jsonOutput = JSON.stringify(config, null, 2);
  fs.writeFileSync(outputPath, jsonOutput, 'utf8');

  const newSize = fs.statSync(outputPath).size;

  // Print summary
  console.log('\n=== SUMMARY ===');
  console.log(`Total agents processed: ${stats.totalAgents}`);
  console.log(`Agents with markdown files found: ${stats.agentsWithFiles}`);
  console.log(`Agents with missing files (using fallback): ${stats.agentsMissingFiles}`);
  console.log(`\nFile size comparison:`);
  console.log(`  Original: ${(originalSize / 1024).toFixed(2)} KB`);
  console.log(`  Updated:  ${(newSize / 1024).toFixed(2)} KB`);
  console.log(`  Increase: ${((newSize - originalSize) / 1024).toFixed(2)} KB`);

  if (stats.missingFiles.length > 0) {
    console.log('\n=== Missing Files ===');
    stats.missingFiles.forEach(item => {
      console.log(`  - ${item.agent}: ${item.file}`);
    });
  }

  if (stats.errors.length > 0) {
    console.log('\n=== Errors ===');
    stats.errors.forEach(item => {
      console.log(`  - ${item.agent}: ${item.error}`);
    });
  }

  console.log('\n✅ Config updated successfully!');
  console.log(`Output written to: ${outputPath}`);
}

// Run the script
try {
  main();
} catch (error) {
  console.error('Fatal error:', error);
  process.exit(1);
}
