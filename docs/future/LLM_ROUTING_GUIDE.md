# LLM Routing Guide

## Overview

The Claude Orchestra now supports intelligent LLM routing to use different AI models for different types of tasks. This allows architecture and planning decisions to continue using Claude while routing coding implementation tasks to a custom LLM endpoint.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  User Requirement                       │
└────────────────────┬────────────────────────────────────┘
                                                          │
                     ▼
         ┌───────────────────────┐
         │  Orchestra Conductor  │
         │  with LLM Router      │
         └───────────┬───────────┘
                                 │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
┌──────────────┐          ┌──────────────┐
│ Architecture │          │   Coding     │
│   Tasks      │          │   Tasks      │
│              │          │              │
│ → Claude API │          │ → Custom LLM │
│ (via Claude  │          │ (coder.      │
│  Code)       │          │  visiquate   │
│              │          │  .com)       │
└──────────────┘          └──────────────┘
```

## Configuration

### 1. Orchestra Config (`config/orchestra-config.json`)

The routing configuration is defined in the `llmRouting` section:

```json
{
  "llmRouting": {
    "enabled": true,
    "endpoints": {
      "coding": {
        "enabled": true,
        "url": "https://coder.visiquate.com",
        "defaultModel": "default",
        "temperature": 0.7,
        "maxTokens": 4096,
        "apiKey": "YOUR_API_KEY_HERE",
        "headers": {},
        "additionalParams": {}
      }
    },
    "rules": {
      "architectureTasks": "claude",
      "codingTasks": "custom-if-enabled",
      "fallbackToClaude": true
    }
  }
}
```

### 2. LLM Router (`src/llm-router.js`)

The router determines which endpoint to use based on:
- **Agent type**: system-architect, python-expert, ios-developer, backend-dev, etc.
- **Task type**: planning, design, implement, code, build, etc.

**Routing Rules:**

1. **Architecture Tasks** → Claude API
   - Agent types: `system-architect`, `architecture`, `specification`, `planner`
   - Task keywords: `design`, `architecture`, `planning`, `specification`, `requirements`, `coordination`

2. **Coding Tasks** → Custom LLM (if enabled)
   - Agent types: `python-expert`, `ios-developer`, `backend-dev`, `mobile-developer`, `coder`, `frontend-dev`
   - Task keywords: `implement`, `code`, `develop`, `build`, `write code`, `programming`

3. **Support Tasks** → Claude API (default)
   - All other agent types and task types default to Claude

## Usage

### CLI Commands

```bash
# View routing configuration
node src/llm-router.js stats

# Test routing decision for specific agent/task
node src/llm-router.js route <agent-type> [task-type]

# Examples
node src/llm-router.js route python-expert implement
node src/llm-router.js route system-architect planning
node src/llm-router.js route backend-dev code

# Call custom coding LLM directly
node src/llm-router.js call-coding-llm "Write a Python function"
```

### Sample Output

```bash
$ node src/llm-router.js stats
LLM Routing Configuration
========================

{
  "endpoints": [
    {
      "name": "coding",
      "enabled": true,
      "url": "https://coder.visiquate.com"
    }
  ],
  "architectureTasks": "Always route to Claude",
  "codingTasks": "Route to https://coder.visiquate.com"
}
```

```bash
$ node src/llm-router.js route python-expert implement
Routing Decision:
{
  "endpoint": "custom",
  "url": "https://coder.visiquate.com",
  "useClaudeCode": false,
  "reason": "Coding tasks routed to custom LLM"
}
```

```bash
$ node src/llm-router.js route system-architect planning
Routing Decision:
{
  "endpoint": "claude",
  "useClaudeCode": true,
  "reason": "Architecture and planning tasks use Claude"
}
```

## Integration with Orchestrator

The `orchestra-conductor.js` automatically uses the LLM router when generating agent instructions:

```javascript
const routing = this.llmRouter.routeTask(agent.type, 'implement');

if (!routing.useClaudeCode) {
  // Route to custom endpoint
  baseInstructions.customEndpoint = routing.url;
  baseInstructions.note = `This agent will use the custom LLM at ${routing.url} for coding tasks.`;
}
```

## Agent Routing Behavior

### Architect (Chief Architect)
- **Routes to**: Claude API (opus model)
- **Reason**: Strategic decisions, architecture design, team coordination
- **Execution**: Via Claude Code's Task tool

### Coding Agents
- **Python Specialist**: Routes to `coder.visiquate.com`
- **Swift Specialist**: Routes to `coder.visiquate.com`
- **Go Specialist**: Routes to `coder.visiquate.com`
- **Rust Specialist**: Routes to `coder.visiquate.com`
- **Flutter Specialist**: Routes to `coder.visiquate.com`
- **Reason**: Implementation tasks benefit from specialized coding LLM
- **Execution**: Custom endpoint (future: direct API calls)

### Support Agents
- **Documentation Lead**: Routes to Claude (haiku model)
- **QA Engineer**: Routes to Claude (sonnet model)
- **Security Auditor**: Routes to Claude (sonnet model)
- **Credential Manager**: Routes to Claude (haiku model)
- **DevOps Engineer**: Routes to Claude (sonnet model)
- **Reason**: These tasks benefit from Claude's reasoning capabilities
- **Execution**: Via Claude Code's Task tool

### Integration Agents
- **API Explorer**: Routes to Claude (sonnet model)
- **Salesforce API Specialist**: Routes to Claude (sonnet model)
- **Authentik API Specialist**: Routes to Claude (sonnet model)
- **Reason**: API exploration and integration require analytical reasoning
- **Execution**: Via Claude Code's Task tool

## Benefits

1. **Optimized for Task Type**: Architecture uses Claude's strategic reasoning, coding uses specialized implementation LLM
2. **Cost Optimization**: Route expensive tasks to appropriate models
3. **Performance**: Use faster models for specific task types
4. **Flexibility**: Easy to add new endpoints or change routing rules
5. **Transparent**: Clear routing decisions visible in orchestrator output

## Customization

### Adding New Endpoints

Edit `config/orchestra-config.json`:

```json
{
  "llmRouting": {
    "endpoints": {
      "coding": { ... },
      "testing": {
        "enabled": true,
        "url": "https://test-llm.example.com",
        "defaultModel": "test-model"
      }
    }
  }
}
```

### Modifying Routing Rules

Edit `src/llm-router.js`:

```javascript
isArchitectureTask(agentType, taskType) {
  const architectureTypes = [
    'system-architect',
    'architecture',
    'specification',
    'planner',
    'your-custom-type'  // Add new type
  ];
  // ...
}
```

## Testing

Run the test suite to verify routing:

```bash
# Test all routing decisions
node src/llm-router.js route python-expert implement
node src/llm-router.js route ios-developer code
node src/llm-router.js route system-architect planning
node src/llm-router.js route test-automator testing
node src/llm-router.js route security-auditor review

# Test orchestrator integration
node src/orchestra-conductor.js "Build a Python REST API"
```

## Troubleshooting

### Issue: All tasks route to Claude

**Solution**: Check that `llmRouting.enabled` is `true` and `endpoints.coding.enabled` is `true` in `orchestra-config.json`.

### Issue: Custom endpoint returns errors

**Solution**:
1. Verify the endpoint URL is accessible
2. Check API key configuration
3. Verify request format matches endpoint expectations
4. Check `llm-router.js` logs for detailed error messages

### Issue: Routing not applied to agents

**Solution**: Verify `orchestra-conductor.js` imports and initializes `LLMRouter`:
```javascript
const LLMRouter = require('./llm-router');
this.llmRouter = new LLMRouter(config);
```

## Future Enhancements

1. **Direct API Integration**: Directly call custom endpoints instead of providing instructions
2. **Dynamic Model Selection**: Choose models based on task complexity
3. **Fallback Strategies**: Automatic fallback to Claude on custom endpoint failures
4. **Load Balancing**: Distribute tasks across multiple endpoints
5. **Cost Tracking**: Monitor usage and costs per endpoint
6. **A/B Testing**: Compare output quality between endpoints

## Version History

- **v1.0** (2025-01-16): Initial implementation with coder.visiquate.com integration
  - Basic routing by agent type and task keywords
  - CLI interface for testing
  - Integration with orchestra-conductor.js
  - Support for custom endpoint configuration
