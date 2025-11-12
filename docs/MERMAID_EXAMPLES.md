# Mermaid Diagram Examples

This document shows examples of the converted diagrams to demonstrate the improved visualization.

---

## Example 1: Orchestra Architecture

### Before (ASCII)
```
┌─────────────────────────────────────────────────────────┐
│                  Chief Architect (Opus)                 │
│              Strategic Decisions & Coordination         │
└───────────────────┬─────────────────────────────────────┘
                    │
        ┌───────────┴────────────┐
        │                        │
┌───────▼───────┐      ┌────────▼────────┐
│ Coding Agents │      │  Support Agents │
├───────────────┤      ├─────────────────┤
│ • Python      │      │ • Documentation │
│ • Swift       │      │ • QA/Testing    │
│ • Go          │      │ • Security      │
│ • Rust        │      │ • Credentials   │
│ • Flutter     │      │ • DevOps        │
└───────────────┘      └─────────────────┘
```

### After (Mermaid)
```mermaid
graph TD
    Architect["Chief Architect (Opus)<br/>Strategic Decisions & Coordination"]

    subgraph CodingAgents["Coding Agents"]
        Python["Python"]
        Swift["Swift"]
        Go["Go"]
        Rust["Rust"]
        Flutter["Flutter"]
    end

    subgraph SupportAgents["Support Agents"]
        Documentation["Documentation"]
        QA["QA/Testing"]
        Security["Security"]
        Credentials["Credentials"]
        DevOps["DevOps"]
    end

    Architect --> CodingAgents
    Architect --> SupportAgents
```

**Improvements**:
- Cleaner visual grouping with subgraphs
- Better rendering in Markdown viewers
- Easier to modify agent lists
- Maintains all original information

---

## Example 2: Model Routing

### Before (ASCII)
```
┌─────────────────────────────────────────────────────┐
│             Claude Code (Orchestrator)              │
│          Spawns agents with model parameter         │
└──────────────────────┬──────────────────────────────┘
                       │
                       │ Agent requests with API aliases
                       │
┌──────────────────────▼──────────────────────────────┐
│             ccproxy (LiteLLM Proxy)                 │
│           https://coder.visiquate.com               │
│             API Alias Mapping:                      │
│   • claude-3-5-sonnet → qwen2.5-coder:32b-instruct  │
│   • claude-3-haiku    → qwen-fast:latest            │
│   • gpt-4             → qwen-quality-128k:latest    │
└──────────────────────┬──────────────────────────────┘
                       │
                       │ Forward to Ollama
                       │
┌──────────────────────▼──────────────────────────────┐
│             Ollama (localhost:11434)                │
│            Mac mini at 192.168.9.123                │
│                       Models:                       │
│        • qwen2.5-coder:32b-instruct (20GB)          │
│              • qwen-fast:latest (5GB)               │
│         • qwen-quality-128k:latest (35GB)           │
└─────────────────────────────────────────────────────┘
```

### After (Mermaid)
```mermaid
graph TB
    Orchestrator["Claude Code (Orchestrator)<br/>Spawns agents with model parameter"]

    CCProxy["ccproxy (LiteLLM Proxy)<br/>https://coder.visiquate.com<br/><br/>API Alias Mapping:<br/>• claude-3-5-sonnet → qwen2.5-coder:32b-instruct<br/>• claude-3-haiku → qwen-fast:latest<br/>• gpt-4 → qwen-quality-128k:latest"]

    Ollama["Ollama (localhost:11434)<br/>Mac mini at 192.168.9.123<br/><br/>Models:<br/>• qwen2.5-coder:32b-instruct (20GB)<br/>• qwen-fast:latest (5GB)<br/>• qwen-quality-128k:latest (35GB)"]

    Orchestrator -->|"Agent requests<br/>with API aliases"| CCProxy
    CCProxy -->|"Forward to Ollama"| Ollama
```

**Improvements**:
- Edge labels clearly show data flow
- Multi-line labels preserve all details
- Better visual balance
- Professional appearance

---

## Example 3: Security Layers

### Before (ASCII)
```
┌─────────────────────────────────────────────────────────┐
│ Layer 1: Network Isolation                              │
│ • LiteLLM bound to localhost only (127.0.0.1:8081)      │
│ • Ollama bound to localhost only (11434)                │
│ • No direct external access                             │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│ Layer 2: Reverse Proxy (Traefik)                        │
│ • Public-facing gateway (coder.visiquate.com:8080)      │
│ • TLS termination (HTTPS encryption)                    │
│ • Bearer token authentication                           │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│ Layer 3: Application Security (LiteLLM)                 │
│ • Model whitelist (only configured models)              │
│ • Parameter validation                                  │
│ • Request logging                                       │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│ Layer 4: Inference Isolation (Ollama)                   │
│ • Local execution only                                  │
│ • No external model downloads at runtime                │
│ • Resource limits via macOS                             │
└─────────────────────────────────────────────────────────┘
```

### After (Mermaid)
```mermaid
graph TB
    Layer1["Layer 1: Network Isolation<br/>• LiteLLM bound to localhost only (127.0.0.1:8081)<br/>• Ollama bound to localhost only (11434)<br/>• No direct external access"]

    Layer2["Layer 2: Reverse Proxy (Traefik)<br/>• Public-facing gateway (coder.visiquate.com:8080)<br/>• TLS termination (HTTPS encryption)<br/>• Bearer token authentication"]

    Layer3["Layer 3: Application Security (LiteLLM)<br/>• Model whitelist (only configured models)<br/>• Parameter validation<br/>• Request logging"]

    Layer4["Layer 4: Inference Isolation (Ollama)<br/>• Local execution only<br/>• No external model downloads at runtime<br/>• Resource limits via macOS"]

    Layer1 --> Layer2
    Layer2 --> Layer3
    Layer3 --> Layer4
```

**Improvements**:
- Clear layer progression
- All security controls preserved
- Simplified connection logic
- Better readability

---

## Example 4: Knowledge Manager Workflow

### Before (ASCII)
```
┌─────────────────────────────────────────────────────────┐
│              Claude Orchestra Orchestrator              │
│         (Before/After Compaction Hooks)                 │
└───────────────────────┬─────────────────────────────────┘
                        │
        ┌───────────────┴────────────────┐
        │                                │
        ▼                                ▼
┌────────────────┐              ┌───────────────────┐
│ Pre-Compaction │              │ Post-Compaction   │
│   Knowledge    │              │    Knowledge      │
│    Capture     │              │   Retrieval       │
└───────┬────────┘              └────────┬──────────┘
        │                                │
        ▼                                ▼
┌─────────────────────────────────────────────────────────┐
│           LanceDB Vector Database                       │
│   ┌──────────────────────────────────────────────────┐ │
│   │  Per-Repository Databases:                       │ │
│   │  - data/knowledge/statushub/                     │ │
│   │  - data/knowledge/cc-orchestra/                  │ │
│   │  - data/knowledge/slack-broker/                  │ │
│   │  Each with semantic vector embeddings            │ │
│   └──────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### After (Mermaid)
```mermaid
graph TD
    Orchestrator["Claude Orchestra Orchestrator<br/>(Before/After Compaction Hooks)"]

    PreCompaction["Pre-Compaction<br/>Knowledge Capture"]
    PostCompaction["Post-Compaction<br/>Knowledge Retrieval"]

    subgraph LanceDB["LanceDB Vector Database"]
        Repos["Per-Repository Databases:<br/>- data/knowledge/statushub/<br/>- data/knowledge/cc-orchestra/<br/>- data/knowledge/slack-broker/<br/>Each with semantic vector embeddings"]
    end

    Orchestrator --> PreCompaction
    Orchestrator --> PostCompaction

    PreCompaction --> LanceDB
    PostCompaction --> LanceDB
```

**Improvements**:
- Subgraph shows database as container
- Bi-directional workflow clear
- Repository isolation evident
- Clean visual structure

---

## Mermaid Syntax Tips

### Basic Graph Types
```
graph TD   - Top Down (vertical)
graph TB   - Top to Bottom (same as TD)
graph LR   - Left to Right (horizontal)
graph RL   - Right to Left
```

### Node Shapes
```
Node["Text"]           - Rectangle (default)
Node("Text")           - Rounded rectangle
Node[("Text")]         - Circle
Node{{"Text"}}         - Hexagon
Node[/"Text"/]         - Parallelogram
```

### Edge Types
```
A --> B                - Solid arrow
A ---|"label"| B       - Labeled edge
A -.-> B               - Dotted arrow
A ==> B                - Thick arrow
```

### Subgraphs
```mermaid
graph TD
    subgraph GroupName["Display Name"]
        Node1
        Node2
    end
```

### Multi-line Labels
Use HTML `<br/>` tags:
```
Node["Line 1<br/>Line 2<br/>Line 3"]
```

---

## Best Practices

1. **Use Descriptive Node Names**
   - Good: `Orchestrator["Claude Code Orchestrator"]`
   - Bad: `A["Orchestrator"]`

2. **Group Related Components**
   - Use subgraphs for logical grouping
   - Label subgraphs clearly

3. **Label Important Edges**
   - Show data flow: `A -->|"HTTP Request"| B`
   - Indicate protocols: `A -->|"HTTPS (TLS)"| B`

4. **Preserve Details**
   - Use multi-line labels for component features
   - Include ports, protocols, and key settings

5. **Keep It Readable**
   - Don't overcrowd diagrams
   - Break complex diagrams into multiple views
   - Use consistent naming conventions

---

## Rendering Mermaid Diagrams

### GitHub/GitLab
Mermaid diagrams render automatically in Markdown files on:
- GitHub (since 2022)
- GitLab (since v13.5)
- Gitea

### VS Code
Install the "Markdown Preview Mermaid Support" extension:
```
ext install bierner.markdown-mermaid
```

### CLI Tools
```bash
# Install mermaid-cli
npm install -g @mermaid-js/mermaid-cli

# Render to PNG
mmdc -i diagram.md -o diagram.png

# Render to SVG
mmdc -i diagram.md -o diagram.svg -t forest
```

### Online Editor
Visit https://mermaid.live for live editing and preview.

---

**Status**: All examples demonstrate successful ASCII to Mermaid conversion
**Rendering**: Tested on GitHub, VS Code, and mermaid.live
**Compatibility**: ✅ All modern Markdown renderers supported
