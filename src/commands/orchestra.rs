//! Orchestra CLI commands
//!
//! Provides CLI wrapper for orchestra conductor functionality.
//! Reads orchestra-config.json directly and generates workflow instructions.

use anyhow::Result;
use cco::orchestra::{OrchestraConfig, Agent};

use crate::OrchestraAction;

/// Run orchestra command
pub async fn run(action: OrchestraAction) -> Result<()> {
    match action {
        OrchestraAction::Stats { format } => stats(format).await,
        OrchestraAction::Generate { requirement, format } => {
            generate(requirement, format).await
        }
        OrchestraAction::Workflow { requirement, format } => {
            workflow(requirement, format).await
        }
    }
}

/// Show orchestra statistics
async fn stats(format: String) -> Result<()> {
    let config = load_config()?;

    let data = serde_json::json!({
        "total_agents": config.agent_count(),
        "architect": {
            "name": config.architect.name,
            "type": config.architect.agent_type,
            "model": config.architect.model
        },
        "agent_counts": {
            "coding": config.coding_agents.len(),
            "integration": config.integration_agents.len(),
            "development": config.development_agents.len(),
            "data": config.data_agents.len(),
            "infrastructure": config.infrastructure_agents.len(),
            "security": config.security_agents.len(),
            "ai_ml": config.ai_ml_agents.len(),
            "mcp": config.mcp_agents.len(),
            "documentation": config.documentation_agents.len(),
            "research": config.research_agents.len(),
            "support": config.support_agents.len(),
            "business": config.business_agents.len()
        },
        "model_distribution": count_agents_by_model(&config)
    });

    format_output(&data, &format)
}

/// Generate agent spawn instructions for a requirement
async fn generate(requirement: String, format: String) -> Result<()> {
    let config = load_config()?;
    let selected_agents = select_agents_for_requirement(&config, &requirement);

    let mut instructions = Vec::new();

    // Add architect first
    instructions.push(generate_task_instruction(&config.architect, &requirement));

    // Add selected agents
    for agent in &selected_agents {
        instructions.push(generate_task_instruction(agent, &requirement));
    }

    let data = serde_json::json!({
        "requirement": requirement,
        "selected_agents": selected_agents.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
        "instructions": instructions,
        "usage_note": "Spawn all agents in ONE message using Claude Code's Task tool"
    });

    format_output(&data, &format)
}

/// Generate complete workflow with todos
async fn workflow(requirement: String, format: String) -> Result<()> {
    let config = load_config()?;
    let selected_agents = select_agents_for_requirement(&config, &requirement);

    let mut instructions = Vec::new();
    let mut todos = Vec::new();

    // Add architect
    instructions.push(generate_task_instruction(&config.architect, &requirement));
    todos.push(serde_json::json!({
        "content": format!("Analyze requirements: {}", requirement),
        "status": "in_progress",
        "activeForm": format!("Analyzing requirements: {}", requirement)
    }));

    // Add selected agents
    for agent in &selected_agents {
        instructions.push(generate_task_instruction(agent, &requirement));

        // Generate relevant todo based on agent type
        let todo_action = match agent.agent_type.as_str() {
            t if t.contains("test") => "Write tests",
            t if t.contains("security") => "Security audit",
            t if t.contains("doc") => "Document code",
            t if t.contains("deploy") || t.contains("devops") => "Configure deployment",
            _ => "Implement feature"
        };

        todos.push(serde_json::json!({
            "content": format!("{} ({})", todo_action, agent.name),
            "status": "pending",
            "activeForm": format!("{}ing ({})", todo_action, agent.name)
        }));
    }

    // Add quality check todos
    todos.push(serde_json::json!({
        "content": "Run all tests",
        "status": "pending",
        "activeForm": "Running all tests"
    }));

    todos.push(serde_json::json!({
        "content": "Review and approve",
        "status": "pending",
        "activeForm": "Reviewing and approving"
    }));

    let data = serde_json::json!({
        "requirement": requirement,
        "selected_agents": selected_agents.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
        "workflow": {
            "phase_1_spawn": {
                "description": "Spawn all agents in ONE message",
                "instructions": instructions
            },
            "phase_2_todos": {
                "description": "Include TodoWrite with all todos",
                "todos": todos,
                "total_todos": todos.len()
            },
            "phase_3_coordination": {
                "description": "Each agent follows coordination protocol",
                "steps": [
                    "Review knowledge base for context",
                    "Perform assigned work",
                    "Store progress in knowledge base",
                    "Document completion"
                ]
            }
        },
        "estimated_duration": estimate_duration(selected_agents.len()),
        "parallel_execution": true
    });

    format_output(&data, &format)
}

/// Load orchestra configuration from embedded config
fn load_config() -> Result<OrchestraConfig> {
    // Use embedded config - no filesystem access needed
    OrchestraConfig::load_embedded()
}

/// Select agents based on requirement keywords
fn select_agents_for_requirement<'a>(
    config: &'a OrchestraConfig,
    requirement: &str,
) -> Vec<&'a Agent> {
    let req_lower = requirement.to_lowercase();
    let mut selected = Vec::new();

    // Always include TDD agent
    for agent in &config.coding_agents {
        if agent.agent_type == "tdd-coding-agent" {
            selected.push(agent);
            break;
        }
    }

    // Language-specific agents
    if req_lower.contains("python") || req_lower.contains("fastapi") || req_lower.contains("django") {
        add_agent_by_type(&mut selected, &config.coding_agents, "python-specialist");
    }
    if req_lower.contains("go") || req_lower.contains("golang") {
        add_agent_by_type(&mut selected, &config.coding_agents, "go-specialist");
    }
    if req_lower.contains("rust") {
        add_agent_by_type(&mut selected, &config.coding_agents, "rust-specialist");
    }
    if req_lower.contains("swift") || req_lower.contains("ios") {
        add_agent_by_type(&mut selected, &config.coding_agents, "swift-specialist");
    }
    if req_lower.contains("flutter") || req_lower.contains("dart") {
        add_agent_by_type(&mut selected, &config.coding_agents, "flutter-specialist");
    }
    if req_lower.contains("typescript") || req_lower.contains("javascript") || req_lower.contains("react") {
        add_agent_by_type(&mut selected, &config.development_agents, "frontend-developer");
    }

    // Integration agents
    if req_lower.contains("salesforce") {
        add_agent_by_type(&mut selected, &config.integration_agents, "salesforce-api-specialist");
    }
    if req_lower.contains("authentik") || req_lower.contains("oauth") || req_lower.contains("oidc") {
        add_agent_by_type(&mut selected, &config.integration_agents, "authentik-api-specialist");
    }
    if req_lower.contains("api") && !req_lower.contains("salesforce") && !req_lower.contains("authentik") {
        add_agent_by_type(&mut selected, &config.integration_agents, "api-explorer");
    }

    // Infrastructure agents
    if req_lower.contains("docker") || req_lower.contains("kubernetes") || req_lower.contains("k8s") || req_lower.contains("deploy") {
        add_agent_by_type(&mut selected, &config.infrastructure_agents, "devops-engineer");
    }

    // Security agents
    if req_lower.contains("auth") || req_lower.contains("security") || req_lower.contains("credential") {
        add_agent_by_type(&mut selected, &config.security_agents, "security-auditor");
        add_agent_by_type(&mut selected, &config.security_agents, "security-engineer");
    }

    // Always include QA and documentation
    add_agent_by_type(&mut selected, &config.development_agents, "test-engineer");
    add_agent_by_type(&mut selected, &config.documentation_agents, "documentation-expert");

    // If nothing matched, provide basic web stack
    if selected.len() <= 3 {
        add_agent_by_type(&mut selected, &config.coding_agents, "python-specialist");
        add_agent_by_type(&mut selected, &config.development_agents, "frontend-developer");
    }

    selected
}

/// Helper to add agent by type if not already selected
fn add_agent_by_type<'a>(
    selected: &mut Vec<&'a Agent>,
    agents: &'a [Agent],
    agent_type: &str,
) {
    if !selected.iter().any(|a| a.agent_type == agent_type) {
        if let Some(agent) = agents.iter().find(|a| a.agent_type == agent_type) {
            selected.push(agent);
        }
    }
}

/// Generate Task instruction for an agent
fn generate_task_instruction(agent: &Agent, requirement: &str) -> serde_json::Value {
    let task_description = format!(
        "{} - {}. IMPORTANT: Read all files FULLY without limits or offsets.",
        agent.name,
        shorten_for_task(requirement)
    );

    serde_json::json!({
        "agent_name": agent.name,
        "agent_type": agent.agent_type,
        "model": agent.model,
        "task": task_description,
        "instruction": format!(
            "Task(\"{}\", \"{}\", \"{}\", \"{}\")",
            agent.name,
            task_description,
            agent.agent_type,
            agent.model
        )
    })
}

/// Shorten requirement for task description
fn shorten_for_task(requirement: &str) -> String {
    if requirement.len() > 100 {
        format!("{}...", &requirement[..97])
    } else {
        requirement.to_string()
    }
}

/// Count agents by model
fn count_agents_by_model(config: &OrchestraConfig) -> serde_json::Value {
    let mut counts = std::collections::HashMap::new();

    for agent in config.all_agents() {
        *counts.entry(&agent.model).or_insert(0) += 1;
    }

    serde_json::json!(counts)
}

/// Estimate duration based on agent count
fn estimate_duration(agent_count: usize) -> String {
    let hours = match agent_count {
        0..=3 => "30 minutes",
        4..=6 => "1 hour",
        7..=10 => "2 hours",
        _ => "2-4 hours",
    };
    format!("{} (parallel execution)", hours)
}

/// Format output based on format type
fn format_output(data: &serde_json::Value, format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        "human" => {
            format_human_readable(data)?;
        }
        _ => anyhow::bail!("Unknown format: {}. Use 'json' or 'human'", format),
    }
    Ok(())
}

/// Format output in human-readable format
fn format_human_readable(data: &serde_json::Value) -> Result<()> {
    // Handle stats output
    if let Some(total) = data["total_agents"].as_u64() {
        println!("\n=== Claude Orchestra Statistics ===\n");
        println!("Total Agents: {}", total);

        if let Some(arch) = data["architect"].as_object() {
            println!("\nLeadership:");
            println!("  {} ({}) - model: {}",
                arch["name"].as_str().unwrap_or("Unknown"),
                arch["type"].as_str().unwrap_or("Unknown"),
                arch["model"].as_str().unwrap_or("Unknown")
            );
        }

        if let Some(counts) = data["agent_counts"].as_object() {
            println!("\nAgent Categories:");
            for (category, count) in counts {
                if let Some(num) = count.as_u64() {
                    if num > 0 {
                        println!("  {}: {}", category, num);
                    }
                }
            }
        }

        if let Some(models) = data["model_distribution"].as_object() {
            println!("\nModel Distribution:");
            for (model, count) in models {
                println!("  {}: {} agents", model, count);
            }
        }

        println!();
        return Ok(());
    }

    // Handle generate/workflow output
    if let Some(req) = data["requirement"].as_str() {
        println!("\n=== Orchestra Workflow ===\n");
        println!("Requirement: {}\n", req);

        if let Some(agents) = data["selected_agents"].as_array() {
            println!("Selected Agents: {} agents", agents.len());
            for agent in agents {
                println!("  - {}", agent.as_str().unwrap_or("Unknown"));
            }
            println!();
        }

        // Show workflow phases
        if let Some(workflow) = data["workflow"].as_object() {
            println!("Workflow Phases:\n");

            // Phase 1
            if let Some(phase1) = workflow["phase_1_spawn"].as_object() {
                println!("Phase 1: {}", phase1["description"].as_str().unwrap_or("Spawn agents"));
                if let Some(instructions) = phase1["instructions"].as_array() {
                    println!("  {} Task() calls to make", instructions.len());
                }
                println!();
            }

            // Phase 2
            if let Some(phase2) = workflow["phase_2_todos"].as_object() {
                println!("Phase 2: {}", phase2["description"].as_str().unwrap_or("Create todos"));
                if let Some(total) = phase2["total_todos"].as_u64() {
                    println!("  {} todo items", total);
                }
                println!();
            }

            // Phase 3
            if let Some(phase3) = workflow["phase_3_coordination"].as_object() {
                println!("Phase 3: {}", phase3["description"].as_str().unwrap_or("Coordination"));
                if let Some(steps) = phase3["steps"].as_array() {
                    for step in steps {
                        println!("  - {}", step.as_str().unwrap_or("Unknown"));
                    }
                }
                println!();
            }
        }

        // Show instructions if present
        if let Some(instructions) = data["instructions"].as_array() {
            println!("Generated Instructions:\n");
            for instr in instructions.iter().take(3) {
                if let Some(cmd) = instr["instruction"].as_str() {
                    println!("{}", cmd);
                }
            }
            if instructions.len() > 3 {
                println!("... and {} more", instructions.len() - 3);
            }
            println!();
        }

        // Show duration estimate
        if let Some(duration) = data["estimated_duration"].as_str() {
            println!("Estimated Duration: {}", duration);
        }

        println!();
        return Ok(());
    }

    // Fallback: just print pretty JSON
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}
