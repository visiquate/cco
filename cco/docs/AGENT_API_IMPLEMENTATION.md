# Agent API Implementation Guide

## Quick Start Implementation

This guide provides the concrete implementation steps to add the Agent Definition API to CCO.

## 1. Create Agent Module Structure

### src/agents/mod.rs
```rust
//! Agent definition management module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod embedded;
mod handlers;
mod registry;

pub use handlers::{
    agents_by_category, agents_health, get_agent_handler,
    list_agents, search_agents, agent_models_config
};
pub use registry::AgentRegistry;

// Include build-time generated agents
include!(concat!(env!("OUT_DIR"), "/embedded_agents.rs"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub r#type: String,
    pub model: String,
    pub category: String,
    pub description: String,
    pub content: String,
    pub metadata: AgentMetadata,
    pub specialties: Vec<String>,
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_authority: Option<AutonomousAuthority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub version: String,
    pub last_updated: String,
    pub tools: Vec<String>,
    pub frontmatter: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousAuthority {
    pub low_risk: bool,
    pub medium_risk: bool,
    pub high_risk: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_architect_approval: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_documentation: Option<bool>,
}

/// Get all embedded agents
pub fn get_all_agents() -> &'static [AgentDefinition] {
    &EMBEDDED_AGENTS
}

/// Get a specific agent by type
pub fn get_agent(type_id: &str) -> Option<&'static AgentDefinition> {
    EMBEDDED_AGENTS.iter().find(|a| a.r#type == type_id)
}

/// Get agents by category
pub fn get_agents_by_category(category: &str) -> Vec<&'static AgentDefinition> {
    EMBEDDED_AGENTS
        .iter()
        .filter(|a| a.category == category)
        .collect()
}

/// Search agents by query
pub fn search_agents_by_query(query: &str) -> Vec<&'static AgentDefinition> {
    let query_lower = query.to_lowercase();
    EMBEDDED_AGENTS
        .iter()
        .filter(|agent| {
            agent.name.to_lowercase().contains(&query_lower)
                || agent.r#type.contains(&query_lower)
                || agent.description.to_lowercase().contains(&query_lower)
                || agent.specialties.iter().any(|s| s.to_lowercase().contains(&query_lower))
                || agent.capabilities.iter().any(|c| c.to_lowercase().contains(&query_lower))
        })
        .collect()
}
```

### src/agents/handlers.rs
```rust
//! HTTP handlers for agent API endpoints

use super::{get_agent, get_all_agents, get_agents_by_category, search_agents_by_query};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

#[derive(Serialize)]
pub struct AgentListResponse {
    pub version: String,
    pub total_agents: usize,
    pub model_distribution: HashMap<String, usize>,
    pub agents: Vec<AgentSummary>,
}

#[derive(Serialize)]
pub struct AgentSummary {
    pub name: String,
    pub r#type: String,
    pub model: String,
    pub category: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: usize,
    pub agents: Vec<SearchResult>,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub name: String,
    pub r#type: String,
    pub model: String,
    pub category: String,
    pub matched_fields: Vec<String>,
}

#[derive(Serialize)]
pub struct CategoryResponse {
    pub category: String,
    pub count: usize,
    pub agents: Vec<AgentSummary>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub agents_loaded: usize,
    pub last_reload: String,
    pub version: String,
    pub embedded_agents: bool,
}

#[derive(Serialize)]
pub struct ModelConfigResponse {
    pub model_overrides: HashMap<String, String>,
    pub agent_models: HashMap<String, String>,
    pub statistics: ModelStatistics,
}

#[derive(Serialize)]
pub struct ModelStatistics {
    pub total_agents: usize,
    pub by_model: HashMap<String, usize>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_types: Option<Vec<String>>,
}

/// List all agents
pub async fn list_agents() -> Json<AgentListResponse> {
    let agents = get_all_agents();

    // Calculate model distribution
    let mut model_dist: HashMap<String, usize> = HashMap::new();
    for agent in agents {
        *model_dist.entry(agent.model.clone()).or_insert(0) += 1;
    }

    // Create agent summaries
    let summaries: Vec<AgentSummary> = agents
        .iter()
        .map(|agent| AgentSummary {
            name: agent.name.clone(),
            r#type: agent.r#type.clone(),
            model: agent.model.clone(),
            category: agent.category.clone(),
            description: agent.description.clone(),
            priority: agent.priority.clone(),
        })
        .collect();

    Json(AgentListResponse {
        version: "2.0.0".to_string(),
        total_agents: agents.len(),
        model_distribution: model_dist,
        agents: summaries,
    })
}

/// Get a specific agent by type
pub async fn get_agent_handler(
    Path(agent_type): Path<String>,
) -> Result<Json<super::AgentDefinition>, (StatusCode, Json<ErrorResponse>)> {
    match get_agent(&agent_type) {
        Some(agent) => Ok(Json(agent.clone())),
        None => {
            let all_types: Vec<String> = get_all_agents()
                .iter()
                .map(|a| a.r#type.clone())
                .collect();

            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Agent not found".to_string(),
                    r#type: "unknown-agent-type".to_string(),
                    available_types: Some(all_types),
                }),
            ))
        }
    }
}

/// Search agents
pub async fn search_agents(Query(params): Query<SearchParams>) -> Json<SearchResponse> {
    let agents = search_agents_by_query(&params.q);

    let results: Vec<SearchResult> = agents
        .into_iter()
        .map(|agent| {
            let mut matched_fields = Vec::new();
            let query_lower = params.q.to_lowercase();

            if agent.name.to_lowercase().contains(&query_lower) {
                matched_fields.push("name".to_string());
            }
            if agent.r#type.contains(&query_lower) {
                matched_fields.push("type".to_string());
            }
            if agent.description.to_lowercase().contains(&query_lower) {
                matched_fields.push("description".to_string());
            }
            if agent.specialties.iter().any(|s| s.to_lowercase().contains(&query_lower)) {
                matched_fields.push("specialties".to_string());
            }

            SearchResult {
                name: agent.name.clone(),
                r#type: agent.r#type.clone(),
                model: agent.model.clone(),
                category: agent.category.clone(),
                matched_fields,
            }
        })
        .collect();

    Json(SearchResponse {
        query: params.q,
        results: results.len(),
        agents: results,
    })
}

/// Get agents by category
pub async fn agents_by_category(Path(category): Path<String>) -> Json<CategoryResponse> {
    let agents = get_agents_by_category(&category);

    let summaries: Vec<AgentSummary> = agents
        .into_iter()
        .map(|agent| AgentSummary {
            name: agent.name.clone(),
            r#type: agent.r#type.clone(),
            model: agent.model.clone(),
            category: agent.category.clone(),
            description: agent.description.clone(),
            priority: agent.priority.clone(),
        })
        .collect();

    Json(CategoryResponse {
        category,
        count: summaries.len(),
        agents: summaries,
    })
}

/// Agent API health check
pub async fn agents_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        agents_loaded: get_all_agents().len(),
        last_reload: chrono::Utc::now().to_rfc3339(),
        version: "2.0.0".to_string(),
        embedded_agents: true,
    })
}

/// Get agent model configuration
pub async fn agent_models_config() -> Json<ModelConfigResponse> {
    let agents = get_all_agents();

    // Build agent type to model mapping
    let mut agent_models: HashMap<String, String> = HashMap::new();
    let mut model_stats: HashMap<String, usize> = HashMap::new();

    for agent in agents {
        agent_models.insert(agent.r#type.clone(), agent.model.clone());
        *model_stats.entry(agent.model.clone()).or_insert(0) += 1;
    }

    // Default model overrides
    let mut model_overrides = HashMap::new();
    model_overrides.insert(
        "claude-sonnet-4.5-20250929".to_string(),
        "claude-haiku-4-5-20251001".to_string(),
    );

    Json(ModelConfigResponse {
        model_overrides,
        agent_models,
        statistics: ModelStatistics {
            total_agents: agents.len(),
            by_model: model_stats,
        },
    })
}
```

## 2. Update build.rs

### build.rs additions
```rust
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

fn embed_agent_definitions() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=~/.claude/agents");

    let home = dirs::home_dir().expect("Unable to get home directory");
    let agents_dir = home.join(".claude").join("agents");

    if !agents_dir.exists() {
        eprintln!("Warning: Agents directory not found at {:?}", agents_dir);
        // Generate empty agents file
        let out_dir = std::env::var("OUT_DIR")?;
        let dest_path = Path::new(&out_dir).join("embedded_agents.rs");
        fs::write(
            dest_path,
            "pub static EMBEDDED_AGENTS: &[crate::agents::AgentDefinition] = &[];"
        )?;
        return Ok(());
    }

    let mut agents = Vec::new();

    // Read all .md files
    for entry in fs::read_dir(&agents_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "md") {
            let type_id = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let content = fs::read_to_string(&path)?;
            let (frontmatter, body) = parse_agent_file(&content);

            // Map agent type to category
            let category = determine_category(&type_id, &frontmatter);

            agents.push(AgentData {
                type_id,
                name: frontmatter.get("name").unwrap_or(&type_id).to_string(),
                model: frontmatter.get("model").unwrap_or(&"haiku".to_string()).to_string(),
                category,
                description: frontmatter.get("description").unwrap_or(&"").to_string(),
                content: body,
                tools: parse_tools(frontmatter.get("tools")),
                frontmatter: frontmatter.clone(),
            });
        }
    }

    // Sort agents by type for consistent ordering
    agents.sort_by(|a, b| a.type_id.cmp(&b.type_id));

    // Generate Rust code
    let out_dir = std::env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("embedded_agents.rs");

    let mut file = fs::File::create(&dest_path)?;

    writeln!(file, "// Auto-generated agent definitions")?;
    writeln!(file, "// Generated at: {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(file)?;
    writeln!(file, "use crate::agents::{{AgentDefinition, AgentMetadata, AutonomousAuthority}};")?;
    writeln!(file, "use std::collections::HashMap;")?;
    writeln!(file)?;
    writeln!(file, "pub static EMBEDDED_AGENTS: &[AgentDefinition] = &[")?;

    for agent in &agents {
        writeln!(file, "    AgentDefinition {{")?;
        writeln!(file, "        name: {:?}.to_string(),", agent.name)?;
        writeln!(file, "        r#type: {:?}.to_string(),", agent.type_id)?;
        writeln!(file, "        model: {:?}.to_string(),", agent.model)?;
        writeln!(file, "        category: {:?}.to_string(),", agent.category)?;
        writeln!(file, "        description: {:?}.to_string(),", agent.description)?;
        writeln!(file, "        content: r###\"{}\"###.to_string(),", agent.content)?;
        writeln!(file, "        metadata: AgentMetadata {{")?;
        writeln!(file, "            version: \"1.0.0\".to_string(),")?;
        writeln!(file, "            last_updated: {:?}.to_string(),", chrono::Utc::now().to_rfc3339())?;
        writeln!(file, "            tools: vec![{}],",
            agent.tools.iter().map(|t| format!("{:?}.to_string()", t)).collect::<Vec<_>>().join(", "))?;
        writeln!(file, "            frontmatter: {{")?;
        writeln!(file, "                let mut map = HashMap::new();")?;
        for (key, value) in &agent.frontmatter {
            writeln!(file, "                map.insert({:?}.to_string(), {:?}.to_string());", key, value)?;
        }
        writeln!(file, "                map")?;
        writeln!(file, "            }},")?;
        writeln!(file, "        }},")?;

        // Add specialties based on agent type
        let specialties = extract_specialties(&agent.type_id, &agent.content);
        writeln!(file, "        specialties: vec![{}],",
            specialties.iter().map(|s| format!("{:?}.to_string()", s)).collect::<Vec<_>>().join(", "))?;

        // Add capabilities
        let capabilities = extract_capabilities(&agent.type_id, &agent.content);
        writeln!(file, "        capabilities: vec![{}],",
            capabilities.iter().map(|c| format!("{:?}.to_string()", c)).collect::<Vec<_>>().join(", "))?;

        // Add autonomous authority if applicable
        if should_have_authority(&agent.type_id) {
            writeln!(file, "        autonomous_authority: Some(AutonomousAuthority {{")?;
            writeln!(file, "            low_risk: true,")?;
            writeln!(file, "            medium_risk: {},", is_medium_risk_agent(&agent.type_id))?;
            writeln!(file, "            high_risk: false,")?;
            writeln!(file, "            requires_architect_approval: Some(true),")?;
            writeln!(file, "            requires_documentation: None,")?;
            writeln!(file, "        }}),")?;
        } else {
            writeln!(file, "        autonomous_authority: None,")?;
        }

        // Add priority if applicable
        if let Some(priority) = determine_priority(&agent.type_id) {
            writeln!(file, "        priority: Some({:?}.to_string()),", priority)?;
        } else {
            writeln!(file, "        priority: None,")?;
        }

        writeln!(file, "    }},")?;
    }

    writeln!(file, "];")?;

    println!("Embedded {} agent definitions", agents.len());

    Ok(())
}

struct AgentData {
    type_id: String,
    name: String,
    model: String,
    category: String,
    description: String,
    content: String,
    tools: Vec<String>,
    frontmatter: HashMap<String, String>,
}

fn parse_agent_file(content: &str) -> (HashMap<String, String>, String) {
    let mut frontmatter = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut body_start = 0;

    if lines.len() > 0 && lines[0] == "---" {
        for i in 1..lines.len() {
            if lines[i] == "---" {
                body_start = i + 1;
                break;
            }
            if let Some((key, value)) = lines[i].split_once(": ") {
                frontmatter.insert(key.to_string(), value.to_string());
            }
        }
    }

    let body = lines[body_start..].join("\n");
    (frontmatter, body)
}

fn parse_tools(tools_str: Option<&String>) -> Vec<String> {
    tools_str
        .map(|s| s.split(", ").map(|t| t.to_string()).collect())
        .unwrap_or_else(|| vec!["Read".to_string(), "Write".to_string()])
}

fn determine_category(type_id: &str, _frontmatter: &HashMap<String, String>) -> String {
    match type_id {
        "chief-architect" => "leadership",
        s if s.contains("test") || s.contains("qa") => "testing",
        s if s.contains("security") || s.contains("audit") => "security",
        s if s.contains("doc") || s.contains("writer") => "documentation",
        s if s.contains("devops") || s.contains("deploy") => "devops",
        s if s.contains("research") => "research",
        s if s.contains("api") || s.contains("integration") => "integration",
        s if s.contains("performance") => "performance",
        s if s.contains("specialist") || s.contains("developer") => "coding",
        _ => "support",
    }.to_string()
}

fn extract_specialties(type_id: &str, _content: &str) -> Vec<String> {
    // Extract specialties based on agent type
    match type_id {
        "python-specialist" => vec![
            "FastAPI/Flask".to_string(),
            "Django".to_string(),
            "Data processing".to_string(),
            "ML/AI integration".to_string(),
            "Async/await patterns".to_string(),
        ],
        "swift-specialist" => vec![
            "SwiftUI".to_string(),
            "UIKit".to_string(),
            "Core Data".to_string(),
            "Combine".to_string(),
            "iOS app architecture".to_string(),
        ],
        _ => vec![],
    }
}

fn extract_capabilities(type_id: &str, _content: &str) -> Vec<String> {
    // Extract capabilities based on agent type
    match type_id {
        "chief-architect" => vec![
            "System design".to_string(),
            "Architecture decisions".to_string(),
            "Agent coordination".to_string(),
            "Technology selection".to_string(),
        ],
        _ => vec![],
    }
}

fn should_have_authority(type_id: &str) -> bool {
    matches!(
        type_id,
        "chief-architect" | "tdd-coding-agent" | "devops-engineer" |
        "security-auditor" | "test-engineer"
    )
}

fn is_medium_risk_agent(type_id: &str) -> bool {
    matches!(
        type_id,
        "chief-architect" | "tdd-coding-agent" | "devops-engineer"
    )
}

fn determine_priority(type_id: &str) -> Option<&str> {
    match type_id {
        "chief-architect" => Some("critical"),
        "security-auditor" => Some("high"),
        "test-engineer" => Some("high"),
        _ => None,
    }
}

// Add to main build function
fn main() {
    // ... existing build code ...

    // Embed agent definitions
    if let Err(e) = embed_agent_definitions() {
        eprintln!("Warning: Failed to embed agent definitions: {}", e);
    }
}
```

## 3. Update server.rs

### Add imports and routes
```rust
// Add to imports
use crate::agents::{
    agents_by_category, agents_health, agent_models_config,
    get_agent_handler, list_agents, search_agents,
};

// In run_server function, add routes to the Router
let app = Router::new()
    // ... existing routes ...

    // Agent API endpoints
    .route("/api/agents", get(list_agents))
    .route("/api/agents/health", get(agents_health))
    .route("/api/agents/models", get(agent_models_config))
    .route("/api/agents/search", get(search_agents))
    .route("/api/agents/category/:category", get(agents_by_category))
    .route("/api/agents/:agent_type", get(get_agent_handler))

    // ... rest of configuration
    .layer(CorsLayer::permissive())
    .with_state(state);
```

## 4. Update lib.rs

### Add agents module
```rust
// In src/lib.rs, add:
pub mod agents;
```

## 5. Testing the Implementation

### Manual Testing
```bash
# Build and run CCO
cargo build --release
./target/release/cco run --port 3000

# Test endpoints
curl http://localhost:3000/api/agents | jq
curl http://localhost:3000/api/agents/python-specialist | jq
curl http://localhost:3000/api/agents/search?q=python | jq
curl http://localhost:3000/api/agents/category/coding | jq
curl http://localhost:3000/api/agents/models | jq
curl http://localhost:3000/api/agents/health | jq
```

### Integration Test
```rust
// tests/agent_api_integration.rs
#[cfg(test)]
mod tests {
    use cco::server::run_server;
    use reqwest;
    use tokio;

    #[tokio::test]
    async fn test_agent_endpoints() {
        // Start server in background
        tokio::spawn(async {
            run_server("127.0.0.1", 3001, 1000000, 3600)
                .await
                .unwrap();
        });

        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let client = reqwest::Client::new();

        // Test list agents
        let resp = client
            .get("http://localhost:3001/api/agents")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        // Test get specific agent
        let resp = client
            .get("http://localhost:3001/api/agents/python-specialist")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        // Test search
        let resp = client
            .get("http://localhost:3001/api/agents/search?q=python")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }
}
```

## 6. Environment Variables

### Shell Configuration
```bash
# Add to ~/.bashrc or ~/.zshrc
export CCO_API_URL="http://localhost:3000"
export CCO_API_TIMEOUT="5000"
export CCO_FALLBACK_TO_FILES="true"
export CCO_CACHE_AGENTS="true"
export CCO_CACHE_TTL="3600"
```

### Node.js Client Example
```javascript
// cco-agent-client.js
const axios = require('axios');
const fs = require('fs');
const path = require('path');
const os = require('os');

class CCOAgentClient {
    constructor() {
        this.apiUrl = process.env.CCO_API_URL || 'http://localhost:3000';
        this.timeout = parseInt(process.env.CCO_API_TIMEOUT || '5000');
        this.cache = new Map();
    }

    async getAgent(agentType) {
        try {
            const response = await axios.get(
                `${this.apiUrl}/api/agents/${agentType}`,
                { timeout: this.timeout }
            );
            return response.data;
        } catch (error) {
            console.error(`Failed to fetch agent from API: ${error.message}`);
            return this.loadLocalAgent(agentType);
        }
    }

    async listAgents() {
        try {
            const response = await axios.get(
                `${this.apiUrl}/api/agents`,
                { timeout: this.timeout }
            );
            return response.data;
        } catch (error) {
            console.error(`Failed to list agents: ${error.message}`);
            return null;
        }
    }

    loadLocalAgent(agentType) {
        const agentPath = path.join(
            os.homedir(),
            '.claude',
            'agents',
            `${agentType}.md`
        );

        if (!fs.existsSync(agentPath)) {
            throw new Error(`Agent not found: ${agentType}`);
        }

        const content = fs.readFileSync(agentPath, 'utf8');
        return this.parseLocalAgent(content, agentType);
    }

    parseLocalAgent(content, agentType) {
        // Basic parsing logic
        const lines = content.split('\n');
        const metadata = {};
        let bodyStart = 0;

        if (lines[0] === '---') {
            for (let i = 1; i < lines.length; i++) {
                if (lines[i] === '---') {
                    bodyStart = i + 1;
                    break;
                }
                const [key, ...value] = lines[i].split(': ');
                if (key) metadata[key] = value.join(': ');
            }
        }

        return {
            type: agentType,
            name: metadata.name || agentType,
            model: metadata.model || 'haiku',
            description: metadata.description || '',
            content: lines.slice(bodyStart).join('\n'),
            metadata: {
                frontmatter: metadata,
                tools: (metadata.tools || '').split(', '),
            },
        };
    }
}

module.exports = CCOAgentClient;
```

## 7. Deployment Checklist

- [ ] Build CCO with embedded agents: `cargo build --release`
- [ ] Verify agents are embedded: Check binary size increase
- [ ] Test all API endpoints manually
- [ ] Run integration tests: `cargo test`
- [ ] Update environment variables in production
- [ ] Deploy binary to server
- [ ] Test API from Claude Code
- [ ] Monitor error rates and performance
- [ ] Document API in README