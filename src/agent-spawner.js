#!/usr/bin/env node

/**
 * Agent Spawner with Automatic Model Configuration
 *
 * Ensures agents are spawned with their configured models from orchestra-config.json
 * This prevents hardcoded model overrides
 */

const config = require('../config/orchestra-config.json');

class AgentSpawner {
  /**
   * Build a map of agent types to configured models
   */
  static buildAgentModelMap() {
    const modelMap = new Map();

    // Add architect
    modelMap.set(config.architect.type, config.architect.model);

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
          modelMap.set(agent.type, agent.model);
        }
      }
    }

    return modelMap;
  }

  /**
   * Get the configured model for an agent type
   * @param {string} agentType - The agent type (e.g., 'rust-specialist')
   * @returns {string} The configured model (e.g., 'haiku' or 'sonnet')
   */
  static getAgentModel(agentType) {
    const modelMap = this.buildAgentModelMap();
    const model = modelMap.get(agentType);

    if (!model) {
      console.warn(`⚠️  Agent type "${agentType}" not found in config. Defaulting to "sonnet".`);
      return 'sonnet';
    }

    return model;
  }

  /**
   * Generate Task tool parameters with correct model
   * @param {string} description - Task description
   * @param {string} prompt - Full prompt for the agent
   * @param {string} agentType - Agent type from config
   * @returns {Object} Task parameters with correct model
   */
  static generateTaskParams(description, prompt, agentType) {
    const model = this.getAgentModel(agentType);

    return {
      description,
      prompt,
      subagent_type: agentType,
      model  // Automatically uses configured model
    };
  }

  /**
   * Spawn agent with correct model configuration
   * This is a template for how to call Task tool correctly
   *
   * Usage in your code:
   *   const params = AgentSpawner.generateTaskParams(
   *     "Fix daemon mode",
   *     "Full prompt here...",
   *     "rust-specialist"
   *   );
   *   // Then use with Task tool:
   *   // Task(params.description, params.prompt, params.subagent_type, params.model)
   */
  static printUsageExample() {
    const examples = [
      {
        agent: 'rust-specialist',
        description: 'Implement daemon mode',
        expectedModel: this.getAgentModel('rust-specialist')
      },
      {
        agent: 'devops-engineer',
        description: 'Implement log rotation',
        expectedModel: this.getAgentModel('devops-engineer')
      },
      {
        agent: 'frontend-developer',
        description: 'Add shutdown button',
        expectedModel: this.getAgentModel('frontend-developer')
      },
      {
        agent: 'test-engineer',
        description: 'Test daemon functionality',
        expectedModel: this.getAgentModel('test-engineer')
      },
      {
        agent: 'documentation-expert',
        description: 'Update documentation',
        expectedModel: this.getAgentModel('documentation-expert')
      }
    ];

    console.log('\n✅ Correct Task Tool Invocations\n');
    console.log('Use these models for your agents:\n');

    examples.forEach((example, i) => {
      console.log(`${i + 1}. ${example.agent}`);
      console.log(`   Model: "${example.expectedModel}" (NOT "sonnet")\n`);
      console.log(`   Task(\n`);
      console.log(`     "${example.description}",\n`);
      console.log(`     "prompt here...",\n`);
      console.log(`     "${example.agent}",\n`);
      console.log(`     "${example.expectedModel}"\n`);
      console.log(`   );\n`);
    });
  }

  /**
   * Validate that a Task call uses the correct model
   * @param {string} agentType - Agent type
   * @param {string} actualModel - Model being used in Task call
   * @returns {boolean} True if model matches config
   */
  static validateModel(agentType, actualModel) {
    const expectedModel = this.getAgentModel(agentType);
    const isValid = expectedModel.toLowerCase() === actualModel.toLowerCase();

    if (!isValid) {
      console.warn(`\n⚠️  Model Mismatch for ${agentType}`);
      console.warn(`   Expected: "${expectedModel}"`);
      console.warn(`   Actual:   "${actualModel}"\n`);
    }

    return isValid;
  }
}

// CLI usage
if (require.main === module) {
  const command = process.argv[2];

  switch (command) {
    case 'get-model':
      const agentType = process.argv[3];
      if (!agentType) {
        console.error('Usage: node src/agent-spawner.js get-model <agent-type>');
        process.exit(1);
      }
      const model = AgentSpawner.getAgentModel(agentType);
      console.log(`${agentType}: ${model}`);
      break;

    case 'validate':
      const agentTypeVal = process.argv[3];
      const actualModelVal = process.argv[4];
      if (!agentTypeVal || !actualModelVal) {
        console.error('Usage: node src/agent-spawner.js validate <agent-type> <model>');
        process.exit(1);
      }
      const isValid = AgentSpawner.validateModel(agentTypeVal, actualModelVal);
      process.exit(isValid ? 0 : 1);
      break;

    case 'examples':
      AgentSpawner.printUsageExample();
      break;

    case 'map':
    default:
      console.log('\n📋 Agent Model Configuration\n');
      const modelMap = AgentSpawner.buildAgentModelMap();
      const sorted = Array.from(modelMap.entries()).sort();
      sorted.forEach(([agentType, model]) => {
        console.log(`  ${agentType}: ${model}`);
      });
      console.log('');
      break;
  }
}

module.exports = AgentSpawner;
