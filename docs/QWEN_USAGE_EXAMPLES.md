# Qwen 2.5 Coder - Practical Usage Examples

## Quick Reference Card

| Model | Best For | Context | Speed | Example Task |
|-------|----------|---------|-------|--------------|
| **7B** (qwen-fast) | Quick fixes, simple code | 32k | 50 tok/s | "Add error handling to this function" |
| **32B** (qwen-quality) | Complex features, production code | 128k | 20 tok/s | "Build a REST API with OAuth2 authentication" |

---

## Direct API Usage

### Testing the 32B Model

```bash
#!/bin/bash
# Test the 32B high-quality model

BEARER_TOKEN="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
ENDPOINT="https://coder.visiquate.com"

# Complex coding task (use 32B)
curl -X POST "$ENDPOINT/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-coder:32b-instruct",
    "prompt": "Create a Python FastAPI application with:\n1. JWT authentication\n2. User registration and login\n3. SQLAlchemy models\n4. Proper error handling\n5. Input validation with Pydantic\n\nProvide complete, production-ready code.",
    "stream": false,
    "options": {
      "temperature": 0.7,
      "num_predict": 4096
    }
  }' | jq -r '.response'
```

### Testing the 7B Model

```bash
# Quick task (use 7B)
curl -X POST "$ENDPOINT/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-coder:7b-instruct",
    "prompt": "Add error handling to this Python function:\ndef divide(a, b):\n    return a / b",
    "stream": false
  }' | jq -r '.response'
```

---

## Claude Orchestra Agent Usage

### Example 1: Complex Full-Stack Feature (Use 32B)

```javascript
// Spawn agents for complex task - all use 32B model
Task("Chief Architect",
  "Design authentication system with JWT, refresh tokens, and role-based access control",
  "backend-architect", "opus")

Task("Python Expert",
  "Implement FastAPI backend with authentication endpoints and middleware. Use qwen2.5-coder:32b-instruct",
  "python-pro", "sonnet")

Task("Security Auditor",
  "Review authentication implementation for vulnerabilities (OWASP Top 10). Use qwen2.5-coder:32b-instruct",
  "security-auditor", "sonnet")

Task("QA Engineer",
  "Create comprehensive integration tests for auth flow. Use qwen2.5-coder:32b-instruct",
  "test-automator", "sonnet")
```

### Example 2: Simple Utility Function (Use 7B)

```javascript
// Quick task - could use 7B for faster results
Task("Python Expert",
  "Write a utility function to validate email addresses with regex. Use qwen2.5-coder:7b-instruct for speed",
  "python-pro", "sonnet")
```

---

## Via LLM Router

### Using the Router CLI

```bash
# Automatically uses 32B (default)
node src/llm-router.js call-coding-llm \
  "Implement OAuth2 authorization code flow with PKCE"

# Explicitly specify 32B
node src/llm-router.js call-coding-llm \
  "Build GraphQL API with authentication" \
  --model qwen2.5-coder:32b-instruct

# Force 7B for speed
node src/llm-router.js call-coding-llm \
  "Add docstrings to this function" \
  --model qwen2.5-coder:7b-instruct
```

### Programmatic Usage

```javascript
const { callCodingLLM } = require('./src/llm-router');

// Complex task - use 32B
const complexCode = await callCodingLLM(
  "Create a Python class for database connection pooling with retry logic",
  {
    model: "qwen2.5-coder:32b-instruct",
    temperature: 0.7,
    maxTokens: 4096
  }
);

// Simple task - use 7B
const simpleCode = await callCodingLLM(
  "Add type hints to this Python function",
  {
    model: "qwen2.5-coder:7b-instruct",
    temperature: 0.3,
    maxTokens: 1024
  }
);
```

---

## Streaming Responses

### For Long Code Generation (32B Model)

```bash
#!/bin/bash
# Stream output for better UX with 32B's longer generation time

curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-coder:32b-instruct",
    "prompt": "Create a complete REST API for a todo app with authentication",
    "stream": true
  }' | while IFS= read -r line; do
    echo "$line" | jq -r '.response // empty' | tr -d '\n'
  done
echo ""
```

---

## Task Complexity Decision Tree

```
Is the task complex?
├─ YES → Use 32B (qwen2.5-coder:32b-instruct)
│   ├─ Multi-file changes
│   ├─ Architecture design
│   ├─ Integration with APIs
│   ├─ Security-critical code
│   ├─ Production deployment
│   └─ >1000 lines of code
│
└─ NO → Use 7B (qwen2.5-coder:7b-instruct)
    ├─ Single function
    ├─ Bug fix
    ├─ Documentation
    ├─ Code formatting
    ├─ Simple refactoring
    └─ <100 lines of code
```

---

## Real-World Scenarios

### Scenario 1: Building a Microservice (32B)

```bash
# Complex task requiring architecture + implementation
PROMPT="Design and implement a Python microservice with:
- FastAPI framework
- PostgreSQL database with SQLAlchemy
- Redis caching layer
- JWT authentication
- Health check endpoint
- Docker containerization
- Prometheus metrics
- Structured logging

Provide complete, production-ready code with proper error handling."

curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"qwen2.5-coder:32b-instruct\",
    \"prompt\": \"$PROMPT\",
    \"stream\": false,
    \"options\": {
      \"temperature\": 0.7,
      \"num_predict\": 8192
    }
  }"
```

### Scenario 2: Quick Bug Fix (7B)

```bash
# Simple task - 7B is sufficient
PROMPT="Fix this Python function that crashes on empty list:
def get_first(items):
    return items[0]"

curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"qwen2.5-coder:7b-instruct\",
    \"prompt\": \"$PROMPT\",
    \"stream\": false
  }"
```

### Scenario 3: Salesforce Integration (32B)

```bash
# API integration - requires 32B for quality
PROMPT="Create a Python module to:
1. Authenticate with Salesforce using OAuth2
2. Query Opportunities using SOQL
3. Create new Leads with error handling
4. Implement retry logic with exponential backoff
5. Handle rate limiting (Salesforce API limits)
6. Proper logging and monitoring

Include unit tests with mocked Salesforce API."

curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"qwen2.5-coder:32b-instruct\",
    \"prompt\": \"$PROMPT\",
    \"stream\": false,
    \"options\": {
      \"temperature\": 0.7,
      \"num_predict\": 6144
    }
  }"
```

### Scenario 4: Code Documentation (7B)

```bash
# Documentation - 7B handles this well
PROMPT="Add comprehensive docstrings to this Python class:
class UserRepository:
    def __init__(self, db):
        self.db = db

    def create_user(self, email, password):
        # implementation
        pass

    def get_user(self, user_id):
        # implementation
        pass"

curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"qwen2.5-coder:7b-instruct\",
    \"prompt\": \"$PROMPT\",
    \"stream\": false
  }"
```

---

## Performance Comparison

### Benchmark Tests

```bash
#!/bin/bash
# Compare 7B vs 32B performance

echo "Testing 7B model..."
time curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-coder:7b-instruct",
    "prompt": "Write a Python function to calculate fibonacci",
    "stream": false
  }' -o /tmp/7b_output.json

echo -e "\n\nTesting 32B model..."
time curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-coder:32b-instruct",
    "prompt": "Write a Python function to calculate fibonacci",
    "stream": false
  }' -o /tmp/32b_output.json

echo -e "\n\n=== Results ==="
echo "7B Model:"
jq -r '.response' /tmp/7b_output.json | wc -c
echo "32B Model:"
jq -r '.response' /tmp/32b_output.json | wc -c
```

**Expected Results**:
- 7B: ~2-3 seconds for simple tasks
- 32B: ~5-8 seconds for simple tasks
- 32B produces higher quality, more complete code

---

## Integration with Claude Code Tasks

### Pattern: Smart Model Selection

```javascript
// In Claude Code, spawn agents with explicit model preferences

// For complex implementation work
Task("Python Expert",
  `Implement complete REST API with authentication.
  IMPORTANT: Use qwen2.5-coder:32b-instruct for high quality.
  Store implementation decisions in memory.
  Coordinate with Security Auditor via hooks.`,
  "python-pro",
  "sonnet")

// For quick documentation
Task("Documentation Lead",
  `Document the API endpoints with examples.
  OPTIONAL: Can use qwen2.5-coder:7b-instruct for speed.
  Retrieve API spec from memory.`,
  "fullstack-developer",
  "haiku")
```

---

## Troubleshooting

### Model Not Found

```bash
# Check available models
curl -s "https://coder.visiquate.com/api/tags" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  | jq -r '.models[].name' | grep qwen

# If 32B is missing, pull it on the Mac mini:
ssh coder@192.168.1.101 "ollama pull qwen2.5-coder:32b-instruct"
```

### Slow Response Times

```bash
# Check if 32B model is loaded in memory
ssh coder@192.168.1.101 "ollama ps"

# Warm up the model with a small request
curl -X POST "https://coder.visiquate.com/api/generate" \
  -H "Authorization: Bearer $BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-coder:32b-instruct",
    "prompt": "hello",
    "stream": false,
    "options": {"num_predict": 10}
  }'
```

### Out of Memory Errors

```bash
# Check Mac mini resources
ssh coder@192.168.1.101 "free -h"

# If RAM is full, consider:
# 1. Use 7B model instead
# 2. Reduce max_tokens
# 3. Restart Ollama: sudo systemctl restart ollama
```

---

## Best Practices

### ✅ DO

1. **Use 32B for production code** - Higher quality, fewer bugs
2. **Use 7B for iterations** - Fast feedback during development
3. **Stream long generations** - Better UX for 32B's longer response times
4. **Cache common patterns** - Store frequently used code snippets
5. **Test with both models** - Compare quality vs speed trade-offs

### ❌ DON'T

1. **Don't use 7B for security code** - Always use 32B for security-critical implementations
2. **Don't use 32B for trivial tasks** - Wastes time and resources
3. **Don't forget to specify model** - Defaults to 32B (good but slower)
4. **Don't exceed context limits** - 7B has 32k, 32B has 128k
5. **Don't ignore streaming for long tasks** - Non-streaming can timeout

---

## Summary Table

| Task Type | Model | Reason |
|-----------|-------|--------|
| REST API implementation | 32B | Complex, production-critical |
| Database schema design | 32B | Architecture, long-term impact |
| Security implementation | 32B | Critical, must be correct |
| API integration (Salesforce, etc) | 32B | Complex, requires context |
| Unit test generation | 32B | Comprehensive coverage needed |
| Bug fix (simple) | 7B | Fast iteration |
| Code formatting | 7B | Simple transformation |
| Documentation | 7B | Straightforward, fast |
| Quick prototypes | 7B | Speed over perfection |
| Config file generation | 7B | Simple, templated |

---

**Quick Tip**: When in doubt, use 32B. It's slower but produces significantly better code. The extra 5-10 seconds is worth it for production-quality results.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-04
