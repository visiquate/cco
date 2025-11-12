# Bearer Token Authentication Setup

**STATUS: PLANNED - NOT YET IMPLEMENTED**

**Complete guide to configuring bearer token authentication for coder.visiquate.com (future deployment)**

## Overview

The LLM router now supports bearer token authentication for secure access to the coder.visiquate.com API. This document explains how to configure and use the authentication system.

---

## Bearer Token

**Token**: `da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c`

This token is required for all API requests to `coder.visiquate.com`.

---

## Configuration Methods

### Method 1: Environment Variable (Recommended for Development)

**Advantages**:
- ✅ Simple to set up
- ✅ No encryption complexity
- ✅ Easy to test
- ✅ Works across all tools

**Setup**:
```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c

# Reload your shell
source ~/.bashrc  # or ~/.zshrc
```

**Usage**:
```bash
# Token is automatically used
node src/llm-router.js call-coding-llm "Write a Python function"
```

---

### Method 2: Credential Manager (Recommended for Production)

**Advantages**:
- ✅ Encrypted storage
- ✅ Rotation tracking
- ✅ Audit logging
- ✅ Enterprise-ready

**Setup**:
```bash
# 1. Set encryption key (required for consistent encryption)
export CREDENTIAL_ENCRYPTION_KEY=your-secret-encryption-key-here

# 2. Store token
node src/credential-manager.js store CODER_LLM_TOKEN da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c api-token

# 3. Verify storage
node src/credential-manager.js list
```

**Usage**:
```bash
# Set encryption key
export CREDENTIAL_ENCRYPTION_KEY=your-secret-encryption-key-here

# Token is automatically retrieved from credential manager
node src/llm-router.js call-coding-llm "Write a Python function"
```

**Credential Rotation**:
```bash
# Check if token needs rotation
node src/credential-manager.js check-rotation

# Update token
node src/credential-manager.js store CODER_LLM_TOKEN new-token-here api-token
```

---

### Method 3: Configuration File (Alternative)

**Advantages**:
- ✅ Centralized configuration
- ✅ Version controlled (if not sensitive)

**Setup**:
```javascript
// config/orchestra-config.json
{
  "llmRouting": {
    "endpoints": {
      "coding": {
        "enabled": true,
        "type": "ollama",
        "url": "https://coder.visiquate.com",
        "apiKey": "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c",
        "defaultModel": "qwen2.5-coder:32b-instruct",
        "temperature": 0.7,
        "maxTokens": 4096
      }
    }
  }
}
```

**⚠️ Warning**: Don't commit sensitive tokens to version control!

---

## How Authentication Works

### Priority Order

The router checks for bearer tokens in this order:

1. **Environment variable** (`CODER_LLM_TOKEN`)
2. **Credential manager** (encrypted storage)
3. **Config file** (`apiKey` field)

### Implementation Details

```javascript
// From src/llm-router.js

// 1. Check environment variable
let bearerToken = process.env.CODER_LLM_TOKEN;

// 2. Fallback to credential manager
if (!bearerToken) {
  try {
    bearerToken = await this.credentialManager.retrieveCredential('CODER_LLM_TOKEN');
  } catch (error) {
    console.warn('Bearer token not found.');
  }
}

// 3. Fallback to config file
const authHeader = bearerToken
  ? { 'Authorization': `Bearer ${bearerToken}` }
  : (endpoint.apiKey ? { 'Authorization': `Bearer ${endpoint.apiKey}` } : {});
```

### Request Format

All API requests include the bearer token:

```http
POST /api/generate HTTP/1.1
Host: coder.visiquate.com
Content-Type: application/json
Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c

{
  "model": "qwen2.5-coder:32b-instruct",
  "prompt": "Write a Python function",
  "stream": false,
  "options": {
    "temperature": 0.7,
    "num_predict": 4096
  }
}
```

---

## Testing Authentication

### Test 1: Environment Variable

```bash
# Set token
export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c

# Test simple call
node src/llm-router.js call-coding-llm "Write hello world in Python"

# Expected: Should return Python code without authentication errors
```

### Test 2: Credential Manager

```bash
# Set encryption key
export CREDENTIAL_ENCRYPTION_KEY=my-secret-key

# Store token
node src/credential-manager.js store CODER_LLM_TOKEN da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c api-token

# Unset environment variable to test credential manager
unset CODER_LLM_TOKEN

# Test with credential manager
node src/llm-router.js call-coding-llm "Write hello world in Python"

# Expected: Should retrieve token from credential manager and work correctly
```

### Test 3: Direct API Call

```bash
# Test with curl
curl -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" \
  -d '{
    "model": "qwen-fast:latest",
    "prompt": "Write hello world in Python",
    "stream": false
  }' | jq -r '.response'

# Expected: Python hello world code
```

### Test 4: Both Models

```bash
export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c

# Test qwen-fast (7B model)
echo "Testing qwen-fast..."
node src/llm-router.js call-coding-llm "Write hello world" 2>&1 | grep -A 5 "Response:"

# Test qwen-quality-128k (32B model)
echo -e "\nTesting qwen-quality-128k..."
node src/llm-router.js call-coding-llm "Create a FastAPI endpoint with JWT auth" 2>&1 | grep -A 10 "Response:"
```

---

## Troubleshooting

### Issue 1: "Bearer token not found"

**Symptoms**:
```
Bearer token not found. Set CODER_LLM_TOKEN environment variable or use credential manager.
```

**Solution**:
```bash
# Check if environment variable is set
echo $CODER_LLM_TOKEN

# If not set, set it
export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c

# Or use credential manager
node src/credential-manager.js store CODER_LLM_TOKEN da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c api-token
```

### Issue 2: "Bad decrypt" error with credential manager

**Symptoms**:
```
error:1C800064:Provider routines::bad decrypt
```

**Solution**:
```bash
# Ensure CREDENTIAL_ENCRYPTION_KEY is consistent
export CREDENTIAL_ENCRYPTION_KEY=same-key-used-for-storing

# If key is lost, re-store the token
node src/credential-manager.js store CODER_LLM_TOKEN da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c api-token
```

### Issue 3: 401 Unauthorized

**Symptoms**:
```
HTTP 401: Unauthorized
```

**Solution**:
```bash
# Verify token is correct
echo $CODER_LLM_TOKEN

# Test token with curl
curl -X POST https://coder.visiquate.com/api/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $CODER_LLM_TOKEN" \
  -d '{"model": "qwen-fast:latest", "prompt": "test", "stream": false}'

# If fails, contact admin for new token
```

### Issue 4: Token not being sent in request

**Symptoms**:
- Requests fail with authentication errors
- Token is set but not being used

**Debug**:
```bash
# Enable verbose logging (if implemented)
export DEBUG=llm-router:*
node src/llm-router.js call-coding-llm "test" --verbose

# Check if token is being retrieved
node -e "console.log(process.env.CODER_LLM_TOKEN ? 'Token set' : 'Token NOT set')"
```

---

## Security Best Practices

### DO ✅

1. **Use environment variables for development**
   ```bash
   export CODER_LLM_TOKEN=...
   ```

2. **Use credential manager for production**
   ```bash
   export CREDENTIAL_ENCRYPTION_KEY=secure-random-key
   node src/credential-manager.js store CODER_LLM_TOKEN ... api-token
   ```

3. **Add to .gitignore**
   ```gitignore
   # Never commit credentials
   /tmp/credentials.json
   .env
   config/credentials.json
   ```

4. **Rotate tokens regularly**
   ```bash
   node src/credential-manager.js check-rotation
   ```

5. **Use secure encryption keys**
   ```bash
   # Generate secure key
   openssl rand -hex 32
   export CREDENTIAL_ENCRYPTION_KEY=$(openssl rand -hex 32)
   ```

### DON'T ❌

1. **Don't hardcode tokens in source code**
   ```javascript
   // ❌ BAD
   const token = "da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c";
   ```

2. **Don't commit tokens to version control**
   ```bash
   # ❌ BAD
   git add config/orchestra-config.json  # Contains apiKey
   git commit -m "Added config with token"  # DON'T DO THIS
   ```

3. **Don't share tokens in public channels**
   - Use secure communication channels
   - Rotate tokens if exposed

4. **Don't use temporary encryption keys in production**
   ```bash
   # ❌ BAD (warning appears)
   ⚠️  Using temporary encryption key. Set CREDENTIAL_ENCRYPTION_KEY for production.
   ```

---

## Integration with Claude Orchestra

When using the Claude Orchestra, the bearer token is automatically applied to all coding agents:

```javascript
// Agents that use coder.visiquate.com automatically get bearer token:
- Python Specialist
- Swift Specialist
- Go Specialist
- Rust Specialist
- Flutter Specialist
- Salesforce API Specialist
- Authentik API Specialist
- Documentation Lead
- DevOps Engineer

// Agents that use Claude API (no bearer token needed):
- Chief Architect
- QA Engineer
- Security Auditor
- Credential Manager
- Technical Writer
- User Experience Designer
```

The routing and authentication happen automatically - no manual configuration needed per agent.

---

## Summary

**Quick Setup (Development)**:
```bash
# 1. Set environment variable
export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c

# 2. Test
node src/llm-router.js call-coding-llm "Write Python hello world"

# 3. Add to shell profile for persistence
echo 'export CODER_LLM_TOKEN=da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c' >> ~/.bashrc
```

**Quick Setup (Production)**:
```bash
# 1. Set encryption key
export CREDENTIAL_ENCRYPTION_KEY=$(openssl rand -hex 32)

# 2. Store token
node src/credential-manager.js store CODER_LLM_TOKEN da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c api-token

# 3. Test
node src/llm-router.js call-coding-llm "Write Python hello world"

# 4. Add encryption key to environment
echo "export CREDENTIAL_ENCRYPTION_KEY=$CREDENTIAL_ENCRYPTION_KEY" >> ~/.bashrc
```

---

**Version**: 1.0.0
**Last Updated**: 2025-11-04
**Maintained By**: VisiQuate DevOps Team
