#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Read current config
const currentConfig = JSON.parse(
  fs.readFileSync('/Users/brent/git/cc-army/config/orchestra-config.json', 'utf8')
);

// Read extracted agents
const agentsDir = path.join(process.env.HOME, '.claude/agents');
const files = fs.readdirSync(agentsDir).filter(f => f.endsWith('.md'));

const extractedAgents = files.map(file => {
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
}).filter(agent => agent.name);

// Categorize agents
const categories = {
  developmentAgents: [
    'frontend-developer', 'backend-architect', 'fullstack-developer', 'code-reviewer',
    'debugger', 'python-pro', 'typescript-pro', 'javascript-pro', 'golang-pro', 'rust-pro',
    'mobile-developer', 'ios-developer', 'nextjs-architecture-expert', 'react-performance-optimization',
    'react-performance-optimizer', 'graphql-architect', 'graphql-performance-optimizer',
    'graphql-security-specialist', 'shell-scripting-pro', 'legacy-modernizer',
    'architecture-modernizer', 'dx-optimizer', 'git-flow-manager', 'dependency-manager',
    'error-detective'
  ],
  dataAgents: [
    'database-architect', 'database-admin', 'database-optimization', 'database-optimizer',
    'data-engineer', 'data-scientist', 'data-analyst', 'nosql-specialist', 'sql-pro'
  ],
  infrastructureAgents: [
    'devops-engineer', 'deployment-engineer', 'cloud-architect', 'cloud-migration-specialist',
    'terraform-specialist', 'network-engineer', 'monitoring-specialist', 'devops-troubleshooter',
    'incident-responder', 'load-testing-specialist'
  ],
  securityAgents: [
    'security-auditor', 'security-engineer', 'api-security-audit', 'penetration-tester',
    'compliance-specialist', 'mcp-security-auditor', 'web-accessibility-checker', 'risk-manager'
  ],
  aiMlAgents: [
    'ai-engineer', 'ml-engineer', 'mlops-engineer', 'model-evaluator',
    'prompt-engineer', 'llms-maintainer'
  ],
  mcpAgents: [
    'mcp-expert', 'mcp-server-architect', 'mcp-integration-engineer', 'mcp-deployment-orchestrator',
    'mcp-protocol-specialist', 'mcp-testing-engineer'
  ],
  documentationAgents: [
    'documentation-expert', 'technical-writer', 'api-documenter', 'changelog-generator',
    'markdown-syntax-formatter', 'llms-maintainer', 'report-generator'
  ],
  researchAgents: [
    'technical-researcher', 'academic-researcher', 'research-orchestrator', 'research-coordinator',
    'research-synthesizer', 'research-brief-generator', 'comprehensive-researcher',
    'fact-checker', 'query-clarifier', 'search-specialist', 'agent-overview'
  ],
  supportAgents: [
    'test-engineer', 'test-automator', 'ui-ux-designer', 'cli-ui-designer', 'performance-engineer',
    'performance-profiler', 'context-manager', 'task-decomposition-expert', 'architect-review',
    'command-expert', 'connection-agent', 'metadata-agent', 'tag-agent', 'document-structure-analyzer',
    'url-link-extractor', 'project-supervisor-orchestrator'
  ],
  businessAgents: [
    'product-strategist', 'business-analyst', 'content-marketer', 'quant-analyst'
  ]
};

// Existing agents that should be preserved
const existingAgentNames = [
  'TDD Coding Agent', 'Python Specialist', 'Swift Specialist', 'Go Specialist',
  'Rust Specialist', 'Flutter Specialist', 'API Explorer', 'Salesforce API Specialist',
  'Authentik API Specialist', 'Documentation Lead', 'Technical Writer',
  'User Experience Designer', 'QA Engineer', 'Security Auditor',
  'Credential Manager', 'DevOps Engineer'
];

// Map subagent types
const typeMap = {
  'code-reviewer': 'reviewer',
  'frontend-developer': 'coder',
  'backend-architect': 'system-architect',
  'ui-ux-designer': 'ux-designer',
  'fullstack-developer': 'coder',
  'debugger': 'debugger',
  'python-pro': 'python-expert',
  'typescript-pro': 'coder',
  'javascript-pro': 'coder',
  'golang-pro': 'backend-dev',
  'rust-pro': 'backend-dev',
  'mobile-developer': 'mobile-developer',
  'ios-developer': 'ios-developer',
  'database-architect': 'system-architect',
  'database-admin': 'coder',
  'database-optimization': 'coder',
  'database-optimizer': 'coder',
  'data-engineer': 'coder',
  'data-scientist': 'researcher',
  'data-analyst': 'researcher',
  'test-engineer': 'test-automator',
  'test-automator': 'test-automator',
  'security-auditor': 'security-auditor',
  'security-engineer': 'security-auditor',
  'devops-engineer': 'deployment-engineer',
  'deployment-engineer': 'deployment-engineer',
  'cloud-architect': 'system-architect',
  'technical-writer': 'technical-writer',
  'documentation-expert': 'technical-writer',
  'api-documenter': 'technical-writer',
  'ai-engineer': 'coder',
  'ml-engineer': 'coder',
  'mlops-engineer': 'deployment-engineer',
  'prompt-engineer': 'coder',
  'mcp-expert': 'coder',
  'mcp-server-architect': 'system-architect',
  'performance-engineer': 'coder',
  'performance-profiler': 'coder',
  'technical-researcher': 'researcher',
  'academic-researcher': 'researcher',
  'context-manager': 'system-architect',
  'task-decomposition-expert': 'planner',
  'architect-review': 'reviewer'
};

// Function to convert agent to config format
function convertAgent(agent, category) {
  const agentName = agent.name
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');

  // Determine type
  const type = typeMap[agent.name] || 'coder';

  // Extract key specialties from description
  const specialties = [];
  if (agent.description) {
    const keywords = agent.description.match(/\b([A-Z][a-z]+(\s+[A-Z][a-z]+)*)/g) || [];
    specialties.push(...keywords.slice(0, 6));
  }

  return {
    name: agentName,
    type: type,
    model: 'sonnet-4.5',
    agentFile: `~/.claude/agents/${agent.file}`,
    role: agent.description ? agent.description.split('.')[0] : agentName,
    specialties: specialties.length > 0 ? specialties : [agentName],
    autonomousAuthority: {
      lowRisk: true,
      mediumRisk: category !== 'securityAgents' && category !== 'infrastructureAgents',
      highRisk: false,
      requiresArchitectApproval: true
    }
  };
}

// Build new config - preserve core structure
const newConfig = {
  name: currentConfig.name,
  version: currentConfig.version,
  description: currentConfig.description,
  architect: currentConfig.architect,
  codingAgents: currentConfig.codingAgents,
  integrationAgents: currentConfig.integrationAgents,
  developmentAgents: [],
  dataAgents: [],
  infrastructureAgents: [],
  securityAgents: [],
  aiMlAgents: [],
  mcpAgents: [],
  documentationAgents: [],
  researchAgents: [],
  supportAgents: currentConfig.supportAgents, // Keep existing support agents
  businessAgents: []
};

// Add new agents to categories
Object.entries(categories).forEach(([category, agentNames]) => {
  const newAgents = agentNames
    .map(name => extractedAgents.find(a => a.name === name))
    .filter(Boolean)
    .map(agent => convertAgent(agent, category));

  // For supportAgents, append to existing
  if (category === 'supportAgents') {
    newConfig[category] = [...currentConfig.supportAgents, ...newAgents];
  } else {
    newConfig[category] = newAgents;
  }
});

// Add back coordination, llmRouting, etc. sections
newConfig.coordination = currentConfig.coordination;
newConfig.llmRouting = currentConfig.llmRouting;
newConfig.knowledgeManager = currentConfig.knowledgeManager;
newConfig.workflow = currentConfig.workflow;
newConfig.decisionAuthority = currentConfig.decisionAuthority;

// Write new config
fs.writeFileSync(
  '/Users/brent/git/cc-army/config/orchestra-config.json',
  JSON.stringify(newConfig, null, 2)
);

console.log('✅ Comprehensive orchestra-config.json created with all 107 agents!');
console.log(`   - Architect: 1`);
console.log(`   - Coding Agents: ${newConfig.codingAgents.length}`);
console.log(`   - Development Agents: ${newConfig.developmentAgents.length}`);
console.log(`   - Data Agents: ${newConfig.dataAgents.length}`);
console.log(`   - Infrastructure Agents: ${newConfig.infrastructureAgents.length}`);
console.log(`   - Security Agents: ${newConfig.securityAgents.length}`);
console.log(`   - AI/ML Agents: ${newConfig.aiMlAgents.length}`);
console.log(`   - MCP Agents: ${newConfig.mcpAgents.length}`);
console.log(`   - Documentation Agents: ${newConfig.documentationAgents.length}`);
console.log(`   - Research Agents: ${newConfig.researchAgents.length}`);
console.log(`   - Support Agents: ${newConfig.supportAgents.length}`);
console.log(`   - Business Agents: ${newConfig.businessAgents.length}`);
console.log(`   - Integration Agents: ${newConfig.integrationAgents.length}`);

const total = 1 + newConfig.codingAgents.length + newConfig.developmentAgents.length +
              newConfig.dataAgents.length + newConfig.infrastructureAgents.length +
              newConfig.securityAgents.length + newConfig.aiMlAgents.length +
              newConfig.mcpAgents.length + newConfig.documentationAgents.length +
              newConfig.researchAgents.length + newConfig.supportAgents.length +
              newConfig.businessAgents.length + newConfig.integrationAgents.length;
console.log(`\n   TOTAL: ${total} agents`);
