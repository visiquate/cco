#!/usr/bin/env node

/**
 * Claude Army Orchestrator
 *
 * Coordinates multi-agent development teams with specialized coding agents,
 * an architect for strategic decisions, and support agents for QA, security,
 * documentation, and credential management.
 */

const config = require('../config/orchestra-config.json');
const LLMRouter = require('./llm-router');
const KnowledgeManager = require('./knowledge-manager');

class ClaudeArmy {
  constructor(options = {}) {
    this.config = config;
    this.activeAgents = new Map();
    this.sharedMemory = new Map();
    this.taskQueue = [];
    this.llmRouter = new LLMRouter(config);

    // Initialize Knowledge Manager if enabled
    if (config.knowledgeManager?.enabled) {
      const repoPath = options.repoPath || process.cwd();
      this.knowledgeManager = new KnowledgeManager({
        repoPath,
        baseDir: config.knowledgeManager.baseDir,
        embeddingDim: config.knowledgeManager.embeddingDim
      });
      this.knowledgeManagerInitialized = false;
    } else {
      this.knowledgeManager = null;
    }
  }

  /**
   * Initialize Knowledge Manager (lazy initialization)
   */
  async initializeKnowledgeManager() {
    if (this.knowledgeManager && !this.knowledgeManagerInitialized) {
      try {
        await this.knowledgeManager.initialize();
        this.knowledgeManagerInitialized = true;
        console.log('✅ Knowledge Manager initialized');
        return true;
      } catch (error) {
        console.error('⚠️  Knowledge Manager initialization failed:', error.message);
        return false;
      }
    }
    return this.knowledgeManagerInitialized;
  }

  /**
   * Initialize the army with Knowledge Manager coordination
   * This sets up the coordination but doesn't spawn agents yet
   */
  async initializeCoordination() {
    console.log('🏗️  Initializing Claude Army coordination...');

    console.log('Coordination: Knowledge Manager (LanceDB vector search)');
    console.log('Leader: Chief Architect (Opus 4.1)');
    console.log(`Total agents: ${this.getTotalAgentCount()}`);

    // Initialize Knowledge Manager if enabled
    if (this.knowledgeManager) {
      await this.initializeKnowledgeManager();
    }

    return {
      topology: 'hierarchical',
      knowledgeManager: this.knowledgeManager !== null,
      ready: true
    };
  }

  /**
   * Get total agent count
   */
  getTotalAgentCount() {
    return 1 + // architect
           this.config.codingAgents.length +
           (this.config.integrationAgents?.length || 0) + // API integration specialists
           this.config.supportAgents.length; // Support agents
  }

  /**
   * Generate Claude Code Task tool invocations
   * This returns the instructions for spawning all agents in parallel
   */
  generateAgentSpawnInstructions(userRequirement) {
    const instructions = {
      architect: this.generateArchitectInstructions(userRequirement),
      codingAgents: this.config.codingAgents.map(agent =>
        this.generateCodingAgentInstructions(agent, userRequirement)
      ),
      integrationAgents: (this.config.integrationAgents || []).map(agent =>
        this.generateIntegrationAgentInstructions(agent, userRequirement)
      ),
      supportAgents: this.config.supportAgents.map(agent =>
        this.generateSupportAgentInstructions(agent, userRequirement)
      )
    };

    return instructions;
  }

  /**
   * Generate architect instructions
   */
  generateArchitectInstructions(requirement) {
    return {
      name: this.config.architect.name,
      type: this.config.architect.type,
      model: this.config.architect.model,
      prompt: `You are the Chief Architect for this project.

USER REQUIREMENT: ${requirement}

YOUR RESPONSIBILITIES:
1. Analyze the user requirement and break it down into technical components
2. Make strategic architecture decisions
3. Determine which coding agents are needed for each component
4. Coordinate with support agents (QA, Security, Documentation, Credentials)
5. Store all decisions in shared memory using hooks
6. Guide the coding agents with clear technical specifications

COORDINATION PROTOCOL:
- Search Knowledge Manager for relevant context before starting
- Store architecture decisions: 'node ~/git/cc-army/src/knowledge-manager.js store "Decision: ..." --type decision'
- Share decisions with agents via Knowledge Manager storage
- Review code from all agents before approval
- Ensure security and QA agents review all implementations

OUTPUT:
- Architecture document
- Component breakdown
- Technology stack recommendations
- Task assignments for coding agents
- Security requirements
- Testing requirements`,
      description: "Architect analyzes requirements and guides team"
    };
  }

  /**
   * Generate coding agent instructions
   */
  generateCodingAgentInstructions(agent, requirement) {
    // Determine routing for this agent's coding tasks
    const routing = this.llmRouter.routeTask(agent.type, 'implement');

    const baseInstructions = {
      name: agent.name,
      type: agent.type,
      model: agent.model,
      routing: routing,
      prompt: `You are a ${agent.languages.join('/')} specialist.

SPECIALTIES: ${agent.specialties.join(', ')}

PROJECT REQUIREMENT: ${requirement}

YOUR RESPONSIBILITIES:
1. Check shared memory for architecture decisions from the Chief Architect
2. Implement components assigned to you in ${agent.languages.join(' or ')}
3. Follow the architecture and coding standards
4. Write clean, well-documented code
5. Coordinate with other agents via shared memory
6. Notify QA agent when features are ready for testing
7. Address security concerns raised by Security Auditor

COORDINATION PROTOCOL:
- Before coding: 'node ~/git/cc-army/src/knowledge-manager.js search "architect decisions"'
- After coding: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: [filename] - [changes]" --type edit --agent ${agent.name}'
- Store your decisions: 'node ~/git/cc-army/src/knowledge-manager.js store "Implementation: ..." --type implementation --agent ${agent.name}'
- Share completion status in Knowledge Manager

QUALITY STANDARDS:
- Write comprehensive tests
- Include inline documentation
- Follow language-specific best practices
- Ensure security best practices
- Optimize for performance`,
      description: `${agent.name} implements features`
    };

    // If routing to custom LLM endpoint, add additional instructions
    if (!routing.useClaudeCode) {
      baseInstructions.customEndpoint = routing.url;
      baseInstructions.note = `This agent will use the custom LLM at ${routing.url} for coding tasks.`;
      baseInstructions.prompt += `

NOTE: This coding task should be executed using the custom LLM endpoint.
The Claude Army orchestrator will handle routing your implementation requests appropriately.`;
    }

    return baseInstructions;
  }

  /**
   * Generate integration agent instructions
   */
  generateIntegrationAgentInstructions(agent, requirement) {
    const roleSpecificInstructions = {
      'API Explorer': `
FOCUS: Explore and understand third-party APIs
- Test API endpoints and authentication
- Document API capabilities and limitations
- Create integration POCs
- Analyze rate limits and quotas
- Generate API client code
- Monitor API changes`,

      'Salesforce API Specialist': `
FOCUS: Salesforce API integration
- Connect to Salesforce via REST/SOAP API
- Write optimized SOQL queries
- Handle OAuth 2.0 authentication
- Implement bulk operations
- Set up streaming API integrations
- Map Salesforce objects to application models
- Handle rate limits and governor limits`,

      'Authentik API Specialist': `
FOCUS: Authentik authentication and API integration
- Configure OAuth2/OIDC flows with Authentik
- Manage users and groups via API
- Set up application providers
- Configure SAML integration
- Implement MFA workflows
- Synchronize user attributes
- Handle Authentik webhooks and events`
    };

    return {
      name: agent.name,
      type: agent.type,
      model: agent.model,
      prompt: `You are the ${agent.name} for this project.

PROJECT REQUIREMENT: ${requirement}

YOUR ROLE: ${agent.role}

${roleSpecificInstructions[agent.name] || ''}

COORDINATION PROTOCOL:
- Retrieve architecture decisions from Knowledge Manager
- Coordinate with coding agents via Knowledge Manager
- Share API schemas and client code in Knowledge Manager
- Report integration challenges to Architect via Knowledge Manager
- Coordinate with Security Auditor on API authentication
- Store all findings and decisions in Knowledge Manager

API-SPECIFIC TASKS:
${agent.specialties ? '- ' + agent.specialties.join('\n- ') : ''}

${agent.apis ? '\nAPI VERSIONS:\n- ' + agent.apis.join('\n- ') : ''}

OUTPUT:
- ${agent.responsibilities.join('\n- ')}`,
      description: `${agent.name} performs ${agent.role}`
    };
  }

  /**
   * Generate support agent instructions
   */
  generateSupportAgentInstructions(agent, requirement) {
    const roleSpecificInstructions = {
      'Documentation Lead': `
FOCUS: Code-level documentation and API reference
- Inline code comments and docstrings
- API reference documentation with code examples
- Function/method documentation (JSDoc, docstrings, etc.)
- Code snippets and usage examples
- README code sections with examples
- Developer-focused documentation`,

      'Technical Writer': `
FOCUS: Architecture documentation and user guides
- Architecture documentation and system design
- System design diagrams and flowcharts
- User guides and tutorials for end users
- How-to guides and best practices
- Conceptual documentation
- Integration guides and deployment guides
- High-level technical communication`,

      'User Experience Designer': `
FOCUS: User experience design and validation
- Design UI/UX mockups and wireframes
- Analyze user flows and journeys
- Ensure accessibility compliance (WCAG 2.1 AA)
- Perform usability testing and validation
- Review mobile-first design implementation
- Final quality validation before completion
- Can block deployment if UX standards not met
- Coordinate with QA on usability testing`,

      'QA Engineer': `
FOCUS: Integration and end-to-end testing
- Create integration test suites
- Test cross-component interactions
- Performance testing
- CI/CD pipeline integration
- Test coverage reports
- Coordinate with UX Designer on usability tests`,

      'Security Auditor': `
FOCUS: Security analysis and vulnerability detection
- Review all code for security vulnerabilities
- Check for OWASP Top 10 issues
- Audit authentication/authorization
- Review credential handling
- Dependency vulnerability scanning
- Generate security reports`,

      'Credential Manager': `
FOCUS: Secure credential management
- Design credential storage strategy (environment variables, secrets manager, etc.)
- Track all credentials used in the project
- Implement secure retrieval mechanisms
- Document credential rotation procedures
- Never store credentials in code
- Use /tmp/credentials.json for temporary storage during development
- Coordinate with Security Auditor`,

      'DevOps Engineer': `
FOCUS: Infrastructure, builds, and deployments
- Docker and docker-compose configuration
- Kubernetes manifests and deployments
- CI/CD pipeline setup (GitHub Actions, GitLab CI)
- Infrastructure as Code (Terraform, CloudFormation)
- AWS infrastructure setup (ECS, ECR, CloudFormation)
- Monitoring and logging configuration
- Zero-downtime deployment strategies
- Container orchestration and scaling`
    };

    return {
      name: agent.name,
      type: agent.type,
      model: agent.model,
      prompt: `You are the ${agent.name} for this project.

PROJECT REQUIREMENT: ${requirement}

YOUR ROLE: ${agent.role}

${roleSpecificInstructions[agent.name] || ''}

COORDINATION PROTOCOL:
- Monitor Knowledge Manager for updates from coding agents
- Review implementations from your perspective
- Report findings to Knowledge Manager
- Coordinate with Chief Architect on critical issues
- Store all analysis and findings in Knowledge Manager

OUTPUT:
- ${agent.responsibilities.join('\n- ')}`,
      description: `${agent.name} performs ${agent.role}`
    };
  }

  /**
   * Store knowledge in the knowledge base
   */
  async storeKnowledge(knowledge) {
    if (!this.knowledgeManager) {
      console.log('⚠️  Knowledge Manager not enabled');
      return null;
    }

    await this.initializeKnowledgeManager();
    return await this.knowledgeManager.store(knowledge);
  }

  /**
   * Search knowledge base
   */
  async searchKnowledge(query, options = {}) {
    if (!this.knowledgeManager) {
      console.log('⚠️  Knowledge Manager not enabled');
      return [];
    }

    await this.initializeKnowledgeManager();
    return await this.knowledgeManager.search(query, options);
  }

  /**
   * Pre-compaction hook: Capture critical knowledge
   */
  async preCompactionHook(conversation, context = {}) {
    if (!this.knowledgeManager || !this.config.knowledgeManager?.autoCapture?.preCompaction) {
      return null;
    }

    console.log('🔄 Running pre-compaction knowledge capture...');
    await this.initializeKnowledgeManager();
    return await this.knowledgeManager.preCompaction(conversation, context);
  }

  /**
   * Post-compaction hook: Retrieve relevant context
   */
  async postCompactionHook(currentTask, context = {}) {
    if (!this.knowledgeManager || !this.config.knowledgeManager?.autoCapture?.postCompaction) {
      return null;
    }

    console.log('🔄 Running post-compaction knowledge retrieval...');
    await this.initializeKnowledgeManager();
    return await this.knowledgeManager.postCompaction(currentTask, context);
  }

  /**
   * Get knowledge statistics
   */
  async getKnowledgeStats() {
    if (!this.knowledgeManager) {
      return null;
    }

    await this.initializeKnowledgeManager();
    return await this.knowledgeManager.getStats();
  }

  /**
   * Generate a complete workflow for a user request
   */
  generateWorkflow(userRequirement) {
    const workflow = {
      phase1_agent_spawn: {
        description: "Spawn all agents in parallel using Claude Code's Task tool",
        note: "ALL agents must be spawned in a SINGLE message",
        agents: this.generateAgentSpawnInstructions(userRequirement)
      },

      phase2_execution: {
        description: "Agents execute their tasks with Knowledge Manager coordination",
        flow: [
          "1. Architect analyzes requirement and creates architecture",
          "2. Architect stores decisions in Knowledge Manager",
          "3. Coding agents retrieve architecture from Knowledge Manager and implement",
          "4. QA agent monitors for completed features and tests",
          "5. Security agent reviews code for vulnerabilities",
          "6. Documentation agent creates docs for all components",
          "7. Credential manager tracks and secures all credentials",
          "8. All agents store status updates in Knowledge Manager"
        ]
      },

      phase3_integration: {
        description: "Integration and final review",
        steps: [
          "QA agent runs full integration test suite",
          "Security agent performs final security audit",
          "Documentation agent finalizes all docs",
          "Architect reviews all outputs and approves",
          "Credential manager documents all credential requirements"
        ]
      },

      knowledge_management: {
        description: "Knowledge capture and retention",
        enabled: this.config.knowledgeManager?.enabled || false,
        operations: [
          "Automatic knowledge capture during implementation",
          "Pre-compaction knowledge storage",
          "Post-compaction context retrieval",
          "Per-repository knowledge isolation",
          "Semantic search for relevant context"
        ]
      }
    };

    return workflow;
  }
}

// Export for use
module.exports = ClaudeArmy;

// CLI usage
if (require.main === module) {
  const army = new ClaudeArmy();
  console.log('Claude Army Orchestrator');
  console.log('========================\n');
  console.log(`Total Agents: ${army.getTotalAgentCount()}`);
  console.log(`- 1 Architect (${config.architect.model})`);
  console.log(`- ${config.codingAgents.length} Coding Specialists`);
  console.log(`- ${config.supportAgents.length} Support Agents\n`);

  if (process.argv[2]) {
    const requirement = process.argv.slice(2).join(' ');
    console.log('Generating workflow for requirement:');
    console.log(`"${requirement}"\n`);

    const workflow = army.generateWorkflow(requirement);
    console.log(JSON.stringify(workflow, null, 2));
  } else {
    console.log('Usage: node orchestra-conductor.js "<your requirement>"');
    console.log('Example: node orchestra-conductor.js "Build a REST API with authentication"');
  }
}
