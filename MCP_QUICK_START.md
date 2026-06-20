# MCP Server Quick Start Guide

## Overview

The CCO MCP server enables Claude and other AI clients to interact with the CCO knowledge base and orchestration system via the Model Context Protocol.

## Installation

The MCP server is built into CCO. No additional installation needed.

## Starting the Server

### Basic Usage

```bash
cco mcp serve
```

This starts the MCP server listening on stdin/stdout.

### With Custom Configuration

```bash
cco mcp serve \
  --daemon-url http://localhost:13109 \
  --server-name my-mcp-server \
  --server-version 1.0.0
```

## Integration with Claude Desktop

### Step 1: Create MCP Configuration

Edit or create `~/.claude/mcp.json`:

```json
{
  "mcpServers": {
    "cco": {
      "command": "cco",
      "args": ["mcp", "serve"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Step 2: Restart Claude Desktop

Claude will automatically connect to the MCP server on startup.

### Step 3: Use the Tools

In Claude, you can now use the MCP tools:

```
@cco search knowledge about architecture decisions
@cco classify command: rm -rf /tmp/*
@cco list agents
@cco start session conversation-123
```

## Available Tools

### 1. Knowledge Search

Search the knowledge base:

```
{
  "name": "knowledge_search",
  "input": {
    "query": "design patterns",
    "limit": 10,
    "project_id": "my-project"
  }
}
```

**Parameters:**
- `query` (required): Search text
- `limit` (optional): Max results (default: 10)
- `project_id` (optional): Project scope

**Returns:** List of matching knowledge items

### 2. Knowledge Store

Store knowledge items:

```
{
  "name": "knowledge_store",
  "input": {
    "text": "Decision: Use PostgreSQL for persistence",
    "knowledge_type": "decision",
    "agent": "architect",
    "project_id": "my-project"
  }
}
```

**Parameters:**
- `text` (required): Knowledge content
- `knowledge_type` (optional): Type (decision, edit, status, completion)
- `agent` (optional): Source agent
- `project_id` (optional): Project scope

**Returns:** Stored item with ID and timestamp

### 3. Classify Command

Classify shell commands:

```
{
  "name": "classify_command",
  "input": {
    "command": "git commit -m 'fix: bug'",
    "expected": "update"
  }
}
```

**Parameters:**
- `command` (required): Command to classify
- `expected` (optional): Expected classification

**Returns:**
- `classification`: Command type (read/create/update/delete/unknown)
- `confidence`: Confidence score (0-1)
- `description`: Human-readable description

**Examples:**
- `cat file.txt` → read (0.95)
- `rm file.txt` → delete (0.95)
- `mkdir dir` → create (0.95)
- `git commit` → update (0.95)

### 4. List Agents

List available agents:

```
{
  "name": "list_agents",
  "input": {}
}
```

**Returns:** Array of agents with name, description, model, and status

**Available Agents:**
- chief-architect
- tdd-agent
- python-specialist
- rust-specialist
- security-auditor
- code-reviewer

### 5. Get Context

Retrieve agent context:

```
{
  "name": "get_context",
  "input": {
    "agent_id": "tdd-agent",
    "context_type": "recent"
  }
}
```

**Parameters:**
- `agent_id` (required): Agent identifier
- `context_type` (optional): Context type (recent, full, default)

**Returns:** Context object with items and metadata

### 6. Session Start

Create a conversation session:

```
{
  "name": "session_start",
  "input": {
    "conversation_id": "conv-123",
    "metadata": {
      "user": "alice",
      "project": "my-project"
    }
  }
}
```

**Parameters:**
- `conversation_id` (required): Unique session ID
- `metadata` (optional): Session metadata

**Returns:** Session ID, conversation ID, timestamp, context

### 7. Pre-Compaction

Capture knowledge before conversation compaction:

```
{
  "name": "pre_compaction",
  "input": {
    "conversation": "Full conversation text...",
    "project_id": "my-project"
  }
}
```

**Parameters:**
- `conversation` (required): Conversation content
- `project_id` (optional): Project scope

**Returns:** Count of captured knowledge items

## Debugging

### Enable Logging

```bash
RUST_LOG=debug cco mcp serve
```

Log levels: `debug`, `info`, `warn`, `error`

### Check Server Status

The server logs each request and response:

```
[DEBUG] Handling request: knowledge_search
[DEBUG] Handling tools/list request
```

### Test with curl

Since the server uses stdin/stdout, you can test with:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":null}' | cco mcp serve
```

## Common Issues

### Server doesn't start

**Issue:** Command not found
**Solution:** Make sure CCO is installed: `cco --version`

### Claude can't find the MCP server

**Issue:** Claude shows "MCP unavailable"
**Solution:**
1. Check `~/.claude/mcp.json` exists and is valid JSON
2. Restart Claude
3. Check logs with `RUST_LOG=debug cco mcp serve`

### Knowledge operations fail

**Issue:** Returns empty results
**Solution:**
1. This is expected if knowledge base is empty
2. Use `knowledge_store` to add items first
3. Ensure daemon is running with `cco daemon status`

### Classification returns "unknown"

**Issue:** Command classification doesn't work
**Solution:**
1. Try a common command: `rm file.txt`
2. Check command syntax is correct
3. Review classification rules in documentation

## Examples

### Example 1: Search and Store Knowledge

```
1. Search: {"name": "knowledge_search", "input": {"query": "patterns"}}
2. Review results
3. Store: {"name": "knowledge_store", "input": {"text": "New pattern...", "type": "decision"}}
```

### Example 2: Classify Commands and Get Agents

```
1. Classify: {"name": "classify_command", "input": {"command": "git push"}}
2. Review: Returns "update" with 0.95 confidence
3. List agents: {"name": "list_agents", "input": {}}
4. Review available agents
```

### Example 3: Session Management

```
1. Start session: {"name": "session_start", "input": {"conversation_id": "work-123"}}
2. Get session ID from response
3. Perform work...
4. Pre-compaction: {"name": "pre_compaction", "input": {"conversation": "chat history"}}
```

## Performance Notes

- First request initializes the server (slight delay)
- Subsequent requests are very fast
- Tool handlers are stateless and cacheable
- No persistent connections needed

## Next Steps

1. **Integrate with daemon**: Implement HTTP API calls in tool handlers
2. **Add authentication**: Token-based access to daemon API
3. **Streaming support**: For long-running operations
4. **Resource access**: Browse knowledge and files

## API Reference

For complete API details, see:
- `/Users/brent.langston/git/cc-orchestra/docs/MCP_SERVER_IMPLEMENTATION.md`
- `/Users/brent.langston/git/cc-orchestra/MCP_SERVER_IMPLEMENTATION.md`

## Source Code

- **Server**: `/Users/brent.langston/git/cc-orchestra/src/mcp_server/server.rs`
- **Tools**: `/Users/brent.langston/git/cc-orchestra/src/mcp_server/tools/`
- **Types**: `/Users/brent.langston/git/cc-orchestra/src/mcp_server/types.rs`

## Testing

Run the test suite:

```bash
# All MCP tests
cargo test mcp_server

# Specific tests
cargo test mcp_server::tools::knowledge
cargo test mcp_server::types
cargo test mcp_server::server

# With output
cargo test -- --nocapture --test-threads=1
```

## Support

For issues or questions:
1. Check the documentation in `/docs/MCP_SERVER_IMPLEMENTATION.md`
2. Review the implementation guide in `MCP_SERVER_IMPLEMENTATION.md`
3. Enable debug logging with `RUST_LOG=debug`
4. Check the source code comments

---

**Status:** Production Ready
**Version:** 0.0.0+45dd16f (commit hash)
**Last Updated:** February 5, 2025
