#![allow(dead_code)]
#![allow(unused_variables)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Rerun build if config changes
    println!("cargo:rerun-if-changed=../config/");
    println!("cargo:rerun-if-changed=../config/orchestra-config.json");
    println!("cargo:rerun-if-changed=config/agents");

    // Get git commit hash
    let git_hash = get_git_hash();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Get build date
    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Validate that config files exist
    let config_path = Path::new("../config/orchestra-config.json");
    if !config_path.exists() {
        eprintln!(
            "Warning: Config file not found at {:?}",
            config_path.canonicalize().unwrap_or_default()
        );
    }

    // Set version - use VERSION_DATE environment variable (required) with git hash
    // VERSION_DATE must be in format: YYYY.MM.DD (e.g., 2025.11.18)
    let base_version = env::var("VERSION_DATE").unwrap_or_else(|_| {
        eprintln!("ERROR: VERSION_DATE environment variable is required!");
        eprintln!("Usage: VERSION_DATE=2025.11.18 cargo build --release");
        eprintln!("Format: YYYY.MM.DD (e.g., 2025.11.18)");
        std::process::exit(1);
    });

    // Validate VERSION_DATE format (basic check: should contain dots and be reasonably long)
    if !base_version.contains('.') || base_version.len() < 8 {
        eprintln!("ERROR: VERSION_DATE format invalid: {}", base_version);
        eprintln!("Expected format: YYYY.MM.DD (e.g., 2025.11.18)");
        std::process::exit(1);
    }

    // Append git hash to version for traceability (format: YYYY.MM.DD+<git-hash>)
    let version = if git_hash != "unknown" && !git_hash.is_empty() {
        format!("{}+{}", base_version, git_hash)
    } else {
        format!("{}+unknown", base_version)
    };

    println!("cargo:rustc-env=CCO_VERSION={}", version);

    // Enable debug info in release builds for crash diagnostics
    println!("cargo:rustc-link-arg=-fPIC");

    // Embed config validation at compile time
    validate_configs();

    // Generate embedded agents code
    generate_embedded_agents();
}

fn get_git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}

fn validate_configs() {
    let config_paths = vec!["../config/orchestra-config.json"];

    for config_file in config_paths {
        let path = Path::new(config_file);
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Validate JSON structure
                    if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                        eprintln!("Invalid JSON in {}: {}", config_file, e);
                        panic!("Config validation failed for {}", config_file);
                    }
                    println!("cargo:warning=Validated config: {}", config_file);
                }
                Err(e) => {
                    eprintln!("Failed to read {}: {}", config_file, e);
                    panic!("Config file read failed: {}", config_file);
                }
            }
        }
    }
}

/// Agent data structure for build time
#[derive(Debug, Clone)]
struct AgentData {
    name: String,
    type_name: String,
    model: String,
    capabilities: Vec<String>,
}

/// Generate embedded agents code at compile time
fn generate_embedded_agents() {
    // Load agents from ~/.claude/agents/ (source of truth with all 117 agents)
    let home_agents = load_agents_from_home_directory();

    // Fallback to local config files if home directory is empty
    let local_agents = if home_agents.is_empty() {
        let orchestra_agents = load_agents_from_orchestra_config();
        if orchestra_agents.is_empty() {
            load_agents_from_markdown()
        } else {
            orchestra_agents
        }
    } else {
        home_agents
    };

    let agents = local_agents;

    if agents.is_empty() {
        println!("cargo:warning=⚠ No agents embedded - check agent configuration");
        return;
    }

    // Generate Rust code
    let rust_code = generate_agents_code(&agents);

    // Write generated file to OUT_DIR
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = PathBuf::from(&out_dir).join("agents.rs");

    if let Err(e) = fs::write(&dest_path, rust_code) {
        eprintln!("Failed to write embedded agents code: {}", e);
        println!("cargo:warning=⚠ Failed to write agents.rs: {}", e);
        return;
    }

    println!(
        "cargo:warning=✓ Embedded {} agents into binary",
        agents.len()
    );
}

/// Load agents from ~/.claude/agents/ (source of truth with all agent definitions)
fn load_agents_from_home_directory() -> Vec<AgentData> {
    let agents_dir = if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".claude/agents")
    } else {
        return Vec::new();
    };

    if !agents_dir.exists() {
        return Vec::new();
    }

    let mut agents = Vec::new();

    // Read all .md files from the agents directory
    if let Ok(entries) = fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .md files
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(agent) = parse_agent_from_markdown(&content) {
                    agents.push(agent);
                }
            }
        }
    }

    agents
}

/// Load agents from markdown files in cco/config/agents/
fn load_agents_from_markdown() -> Vec<AgentData> {
    let agents_dir = PathBuf::from("config/agents");

    if !agents_dir.exists() {
        return Vec::new();
    }

    let mut agents = Vec::new();

    // Read all .md files from the agents directory
    if let Ok(entries) = fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .md files
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(agent) = parse_agent_from_markdown(&content) {
                    agents.push(agent);
                } else {
                    println!(
                        "cargo:warning=⚠ Failed to parse agent from: {}",
                        path.display()
                    );
                }
            } else {
                println!(
                    "cargo:warning=⚠ Failed to read agent file: {}",
                    path.display()
                );
            }
        }
    }

    agents
}

/// Parse agent from markdown file with YAML frontmatter
fn parse_agent_from_markdown(content: &str) -> Option<AgentData> {
    // Check if file starts with ---
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing --- marker
    let rest = &content[3..]; // Skip opening ---
    let closing_pos = rest.find("---")?;

    // Extract the YAML content between the markers
    let yaml_content = &rest[..closing_pos];

    // Simple line-by-line YAML parser
    let mut name: Option<String> = None;
    let mut type_name: Option<String> = None;
    let mut model: Option<String> = None;
    let mut capabilities: Vec<String> = Vec::new();

    for line in yaml_content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key: value pairs
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();

            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                &value[1..value.len() - 1]
            } else {
                value
            };

            match key {
                "name" => name = Some(value.to_string()),
                "type" | "type_name" => type_name = Some(value.to_string()),
                "model" => model = Some(value.to_string()),
                "tools" | "capabilities" => {
                    // Parse comma-separated tools/capabilities
                    capabilities = value
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }

    // Validate required fields
    let name = name?;
    // If type_name is not set, use name as the type_name (agents use same value for both)
    let type_name = type_name.or_else(|| Some(name.clone()))?;
    let model = model?;

    // Validate model
    if !["opus", "sonnet", "haiku"].contains(&model.as_str()) {
        println!(
            "cargo:warning=⚠ Invalid model '{}' for agent '{}', must be opus/sonnet/haiku",
            model, name
        );
        return None;
    }

    Some(AgentData {
        name,
        type_name,
        model,
        capabilities,
    })
}

/// Load agents from orchestra config JSON (fallback)
fn load_agents_from_orchestra_config() -> Vec<AgentData> {
    let config_path = PathBuf::from("../config/orchestra-config.json");

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            println!(
                "cargo:warning=⚠ Failed to read orchestra config: {}",
                e
            );
            return Vec::new();
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(e) => {
            println!(
                "cargo:warning=⚠ Failed to parse orchestra config: {}",
                e
            );
            return Vec::new();
        }
    };

    let mut agents = Vec::new();

    // Extract Chief Architect
    if let Some(architect) = json.get("architect") {
        if let Some(agent) = extract_agent_from_json(architect, "chief-architect") {
            agents.push(agent);
        }
    }

    // Extract coding agents
    if let Some(coding_agents) = json.get("codingAgents").and_then(|v| v.as_array()) {
        for agent_obj in coding_agents {
            if let Some(type_name) = agent_obj.get("type").and_then(|v| v.as_str()) {
                if let Some(agent) = extract_agent_from_json(agent_obj, type_name) {
                    agents.push(agent);
                }
            }
        }
    }

    // Extract integration agents
    if let Some(int_agents) = json.get("integrationAgents").and_then(|v| v.as_array()) {
        for agent_obj in int_agents {
            if let Some(type_name) = agent_obj.get("type").and_then(|v| v.as_str()) {
                if let Some(agent) = extract_agent_from_json(agent_obj, type_name) {
                    agents.push(agent);
                }
            }
        }
    }

    // Extract development agents
    if let Some(dev_agents) = json.get("developmentAgents").and_then(|v| v.as_array()) {
        for agent_obj in dev_agents {
            if let Some(type_name) = agent_obj.get("type").and_then(|v| v.as_str()) {
                if let Some(agent) = extract_agent_from_json(agent_obj, type_name) {
                    agents.push(agent);
                }
            }
        }
    }

    // Extract support agents (if present)
    if let Some(support_agents) = json.get("supportAgents").and_then(|v| v.as_array()) {
        for agent_obj in support_agents {
            if let Some(type_name) = agent_obj.get("type").and_then(|v| v.as_str()) {
                if let Some(agent) = extract_agent_from_json(agent_obj, type_name) {
                    agents.push(agent);
                }
            }
        }
    }

    agents
}

/// Extract agent data from JSON object
fn extract_agent_from_json(
    json_obj: &serde_json::Value,
    type_name: &str,
) -> Option<AgentData> {
    let name = json_obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(type_name)
        .to_string();

    let model = json_obj
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("haiku")
        .to_string();

    // Extract capabilities from capabilities or specialties arrays
    let mut capabilities = Vec::new();

    // Try capabilities array
    if let Some(caps) = json_obj.get("capabilities").and_then(|v| v.as_array()) {
        for cap in caps {
            if let Some(cap_str) = cap.as_str() {
                capabilities.push(cap_str.to_string());
            }
        }
    }

    // Try specialties array (if no capabilities)
    if capabilities.is_empty() {
        if let Some(specs) = json_obj.get("specialties").and_then(|v| v.as_array()) {
            for spec in specs {
                if let Some(spec_str) = spec.as_str() {
                    capabilities.push(spec_str.to_string());
                }
            }
        }
    }

    // Add standard capabilities if empty
    if capabilities.is_empty() {
        capabilities = vec![
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "Bash".to_string(),
        ];
    }

    Some(AgentData {
        name,
        type_name: type_name.to_string(),
        model,
        capabilities,
    })
}

/// Generate Rust code for embedded agents
fn generate_agents_code(agents: &[AgentData]) -> String {
    let mut code = String::new();

    // Header
    code.push_str("// This file is auto-generated by build.rs\n");
    code.push_str("// DO NOT EDIT MANUALLY\n\n");

    code.push_str("use crate::agents_config::Agent;\n");
    code.push_str("use std::collections::HashMap;\n\n");

    // Generate agent creation functions
    code.push_str("/// Initialize embedded agents from compile-time data\n");
    code.push_str("pub fn create_embedded_agents() -> HashMap<String, Agent> {\n");
    code.push_str("    let mut agents = HashMap::new();\n\n");

    for agent in agents {
        let agent_name = escape_string(&agent.name);
        let agent_type_name = escape_string(&agent.type_name);
        let agent_model = escape_string(&agent.model);

        // Generate capabilities array
        let capabilities_array = generate_capabilities_array(&agent.capabilities);

        code.push_str(&format!(
            "    // {}\n",
            agent_name
        ));
        code.push_str(&format!(
            "    agents.insert(\n"
        ));
        code.push_str(&format!(
            "        \"{}\".to_string(),\n",
            agent_name
        ));
        code.push_str(&format!(
            "        Agent {{\n"
        ));
        code.push_str(&format!(
            "            name: \"{}\".to_string(),\n",
            agent_name
        ));
        code.push_str(&format!(
            "            type_name: \"{}\".to_string(),\n",
            agent_type_name
        ));
        code.push_str(&format!(
            "            model: \"{}\".to_string(),\n",
            agent_model
        ));
        code.push_str(&format!(
            "            capabilities: vec![{}],\n",
            capabilities_array
        ));
        code.push_str("        },\n");
        code.push_str("    );\n\n");
    }

    code.push_str("    agents\n");
    code.push_str("}\n\n");

    // Generate static data for quick access
    code.push_str("/// Static embedded agents data\n");
    code.push_str("pub static EMBEDDED_AGENTS_COUNT: usize = ");
    code.push_str(&agents.len().to_string());
    code.push_str(";\n\n");

    code.push_str("/// List of embedded agent names\n");
    code.push_str("pub static EMBEDDED_AGENT_NAMES: &[&str] = &[\n");
    for agent in agents {
        code.push_str(&format!("    \"{}\",\n", escape_string(&agent.name)));
    }
    code.push_str("];\n\n");

    // Generate a lookup table for models
    code.push_str("/// Agent name to model mapping\n");
    code.push_str("pub static AGENT_MODELS: &[(&str, &str)] = &[\n");
    for agent in agents {
        code.push_str(&format!(
            "    (\"{}\", \"{}\"),\n",
            escape_string(&agent.name),
            escape_string(&agent.model)
        ));
    }
    code.push_str("];\n\n");

    // Summary statistics
    let opus_count = agents.iter().filter(|a| a.model == "opus").count();
    let sonnet_count = agents.iter().filter(|a| a.model == "sonnet").count();
    let haiku_count = agents.iter().filter(|a| a.model == "haiku").count();

    code.push_str("/// Build-time statistics\n");
    code.push_str("pub static BUILD_STATS: &str = r#\"\n");
    code.push_str(&format!("Embedded Agents: {}\n", agents.len()));
    code.push_str(&format!("  - Opus agents: {}\n", opus_count));
    code.push_str(&format!("  - Sonnet agents: {}\n", sonnet_count));
    code.push_str(&format!("  - Haiku agents: {}\n", haiku_count));
    code.push_str("\"#;\n");

    code
}

/// Escape special characters in strings for Rust code
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Generate Rust array of capability strings
fn generate_capabilities_array(capabilities: &[String]) -> String {
    if capabilities.is_empty() {
        return String::new();
    }

    let capability_strings: Vec<String> = capabilities
        .iter()
        .map(|c| format!("\"{}\".to_string()", escape_string(c)))
        .collect();

    capability_strings.join(", ")
}
