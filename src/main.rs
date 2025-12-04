//! CCO CLI - Claude Code Orchestra Command Line Interface

use clap::{Parser, Subcommand};
use std::io::{BufRead, Seek};

// Binary-specific modules
mod commands;
mod install;
mod update;

// Import from library
use cco::auto_update;
use cco::version::DateVersion;

#[derive(Parser)]
#[command(name = "cco")]
#[command(
    about = "Claude Code Orchestra - Multi-agent development system with intelligent caching"
)]
#[command(version = env!("CCO_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Arguments to pass to Claude Code (when no subcommand specified)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    claude_args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch TUI monitoring dashboard
    Tui,

    /// Run the CCO proxy server
    Run {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Database URL
        #[arg(long, default_value = "sqlite://analytics.db")]
        database_url: String,

        /// Cache size in bytes
        #[arg(long, default_value = "1073741824")]
        cache_size: u64,

        /// Cache TTL in seconds
        #[arg(long, default_value = "3600")]
        cache_ttl: u64,

        /// Enable debug logging
        #[arg(long)]
        debug: bool,
    },

    /// Install CCO to ~/.local/bin
    Install {
        /// Force reinstallation even if already installed
        #[arg(long)]
        force: bool,
    },

    /// Check for and install updates
    Update {
        /// Only check for updates, don't install
        #[arg(long)]
        check: bool,

        /// Prompt for confirmation before installing (default: auto-install)
        #[arg(long)]
        prompt: bool,

        /// Update channel (stable or beta)
        #[arg(long)]
        channel: Option<String>,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show version and build info
    Version,

    /// Check health status
    Health {
        /// Host to check
        #[arg(default_value = "127.0.0.1")]
        host: String,

        /// Port to check
        #[arg(default_value = "3000")]
        port: u16,
    },

    /// Manage credentials
    Credentials {
        #[command(subcommand)]
        action: CredentialAction,
    },

    /// Show status of running instances
    Status,

    /// Shutdown running instances
    Shutdown {
        /// Specific port to shutdown
        #[arg(short, long)]
        port: Option<u16>,

        /// Shutdown all instances
        #[arg(short, long)]
        all: bool,
    },

    /// View logs from a running instance
    Logs {
        /// Specific port to view logs from
        #[arg(short, long)]
        port: Option<u16>,

        /// Follow logs (tail -f behavior)
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show (default: 50)
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
    },

    /// Launch the TUI metrics dashboard
    Dashboard {
        /// Database path for metrics
        #[arg(long, default_value = "analytics.db")]
        database: String,

        /// Refresh rate in milliseconds
        #[arg(long, default_value = "1000")]
        refresh_ms: u64,
    },

    /// Manage daemon lifecycle (start, stop, restart, status)
    ///
    /// The daemon binds to a random OS-assigned port for improved security.
    /// Clients automatically discover the port via the PID file.
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Manage the CCO HTTP server (install, run, uninstall)
    Server {
        #[command(subcommand)]
        action: ServerAction,
    },

    /// Manage orchestration system (now integrated into daemon)
    ///
    /// Orchestration API is available at /api/orchestration/* routes on the daemon.
    /// Use `cco daemon start` to start the daemon (includes orchestration).
    Orchestration {
        #[command(subcommand)]
        action: OrchestrationAction,
    },

    /// Login to CCO releases API via OIDC device flow
    Login,

    /// Logout from CCO releases API (clear stored tokens)
    Logout,

    /// Manage knowledge store (search, store, stats)
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeAction,
    },

    /// Manage LLM routing (stats, route decisions, custom calls)
    LlmRouter {
        #[command(subcommand)]
        action: LlmRouterAction,
    },

    /// Orchestra conductor (stats, generate, workflow)
    Orchestra {
        #[command(subcommand)]
        action: OrchestraAction,
    },

    /// Re-sign binary with JIT entitlements (macOS only)
    ///
    /// Required for ML inference on some macOS systems.
    /// Run this after copying the binary to a new machine.
    #[cfg(target_os = "macos")]
    SelfSign {
        /// Path to binary (default: current executable)
        #[arg(short, long)]
        path: Option<String>,
    },
}

#[derive(Subcommand)]
enum OrchestrationAction {
    /// Get context for an agent
    ///
    /// Queries the daemon's orchestration API at /api/orchestration/context
    GetContext {
        /// Issue ID
        issue_id: String,

        /// Agent type
        agent_type: String,
    },

    /// Store agent results
    ///
    /// Posts to the daemon's orchestration API at /api/orchestration/results
    StoreResult {
        /// Issue ID
        issue_id: String,

        /// Agent type
        agent_type: String,

        /// Result JSON file
        #[arg(short, long)]
        file: String,
    },

    /// Publish an event
    ///
    /// Posts to the daemon's orchestration API at /api/orchestration/events
    PublishEvent {
        /// Event type
        event_type: String,

        /// Publisher ID
        #[arg(short, long)]
        publisher: String,

        /// Topic
        #[arg(short, long)]
        topic: String,

        /// Event data (JSON)
        #[arg(short, long)]
        data: String,
    },

    /// Subscribe to events
    ///
    /// Long-polls the daemon's orchestration API at /api/orchestration/events/wait
    Subscribe {
        /// Event type to subscribe to
        event_type: String,

        /// Timeout in milliseconds
        #[arg(short, long, default_value = "30000")]
        timeout: u64,
    },

    /// Show orchestration status
    ///
    /// Queries the daemon's orchestration API at /api/orchestration/status
    Status,

    /// Clear context cache
    ///
    /// Deletes cached context via daemon's orchestration API
    ClearCache {
        /// Issue ID to clear cache for
        issue_id: String,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon
    ///
    /// Use port 0 for random OS-assigned port (recommended for security).
    /// Clients auto-discover the port from the PID file.
    Start {
        /// Port to listen on (0 = random OS-assigned port, default: 0)
        #[arg(short, long, default_value = "0")]
        port: u16,

        /// Host to bind to (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Cache size in bytes (default: 1GB)
        #[arg(long, default_value = "1073741824")]
        cache_size: u64,

        /// Cache TTL in seconds (default: 3600)
        #[arg(long, default_value = "3600")]
        cache_ttl: u64,
    },

    /// Stop the daemon
    Stop,

    /// Restart the daemon
    ///
    /// Use port 0 for random OS-assigned port (recommended for security).
    Restart {
        /// Port to listen on (0 = random OS-assigned port, default: 0)
        #[arg(short, long, default_value = "0")]
        port: u16,

        /// Host to bind to (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Cache size in bytes (default: 1GB)
        #[arg(long, default_value = "1073741824")]
        cache_size: u64,

        /// Cache TTL in seconds (default: 3600)
        #[arg(long, default_value = "3600")]
        cache_ttl: u64,
    },

    /// Check daemon status
    Status,

    /// View daemon logs
    Logs {
        /// Follow logs (tail -f behavior)
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show (default: 50)
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
    },

    /// Install daemon as system service
    Install,

    /// Uninstall daemon from system service
    Uninstall,

    /// Enable service (start on boot)
    Enable,

    /// Disable service (don't start on boot)
    Disable,

    /// Run daemon in foreground (internal use)
    Run,
}

#[derive(Subcommand)]
enum ServerAction {
    /// Install/initialize server (idempotent)
    Install {
        /// Force reinstall even if already installed
        #[arg(long)]
        force: bool,
    },
    /// Run the server (starts in background, idempotent)
    Run {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Uninstall server (idempotent)
    Uninstall,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., updates.enabled)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., updates.enabled)
        key: String,
    },
    /// Show all configuration
    Show,
}

#[derive(Subcommand)]
enum CredentialAction {
    /// Store a credential
    Store {
        /// Credential key
        key: String,
        /// Credential value
        value: String,
    },
    /// Retrieve a credential
    Retrieve {
        /// Credential key
        key: String,
    },
    /// Delete a credential
    Delete {
        /// Credential key
        key: String,
    },
    /// List all credentials
    List,
    /// Check credential rotation status
    CheckRotation,
}

#[derive(Subcommand, Debug)]
enum KnowledgeAction {
    /// Store knowledge item
    Store {
        /// Knowledge text
        text: String,

        /// Knowledge type (decision, architecture, implementation, etc.)
        #[arg(short, long)]
        r#type: Option<String>,

        /// Agent name
        #[arg(short, long)]
        agent: Option<String>,

        /// Output format (json or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Search knowledge base
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Show knowledge statistics
    Stats {
        /// Output format
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Pre-compaction knowledge capture
    PreCompaction {
        /// Conversation text (or path to file)
        conversation: String,

        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Post-compaction knowledge retrieval
    PostCompaction {
        /// Current task description
        task: String,

        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

#[derive(Subcommand, Debug)]
enum LlmRouterAction {
    /// Show routing configuration and statistics
    Stats {
        /// Output format (json or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Show routing decision for agent and task type
    Route {
        /// Agent type (e.g., python-expert, system-architect)
        agent_type: String,

        /// Task type (e.g., implement, design, code)
        #[arg(short, long)]
        task_type: Option<String>,

        /// Output format (json or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Call custom LLM endpoint with a prompt
    Call {
        /// Prompt to send to LLM
        prompt: String,

        /// Output format (json or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
}

#[derive(Subcommand, Debug)]
enum OrchestraAction {
    /// Show orchestra statistics (agent counts, model distribution)
    Stats {
        /// Output format (json or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Generate agent spawn instructions for a requirement
    Generate {
        /// User requirement description
        requirement: String,

        /// Output format (json or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Generate complete workflow with todos for a requirement
    Workflow {
        /// User requirement description
        requirement: String,

        /// Output format (json or human)
        #[arg(short, long, default_value = "human")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Check for updates synchronously BEFORE launching anything
    // This ensures we're always running the latest version
    // Note: Uses config settings (enabled, auto_install, check_interval)
    // To disable: Set CCO_AUTO_UPDATE=false environment variable
    auto_update::check_for_updates_blocking().await;

    // If no command specified, default to launching Claude Code with orchestration
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // Initialize tracing early for default command
            tracing_subscriber::fmt::init();

            // Launch Claude Code with orchestration support
            return commands::launcher::launch_claude_code(cli.claude_args).await;
        }
    };

    match command {
        Commands::Tui => {
            // Initialize tracing for TUI command
            tracing_subscriber::fmt::init();
            commands::tui::launch_tui().await
        }

        Commands::Install { force } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();
            install::run(force).await
        }

        Commands::Update {
            check,
            prompt,
            channel,
        } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();
            // Default is auto-install (yes=true), unless --prompt is specified
            let auto_confirm = !prompt;
            update::run(check, auto_confirm, channel).await
        }

        Commands::Config { action } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();
            match action {
                ConfigAction::Set { key, value } => auto_update::set_config(&key, &value),
                ConfigAction::Get { key } => {
                    auto_update::get_config(&key)?;
                    Ok(())
                }
                ConfigAction::Show => auto_update::show_config(),
            }
        }

        Commands::Run {
            port,
            host,
            database_url: _database_url,
            cache_size: _cache_size,
            cache_ttl: _cache_ttl,
            debug,
        } => {
            // Configure logging level based on debug flag BEFORE initializing tracing
            if debug {
                std::env::set_var("RUST_LOG", "debug");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }

            // Initialize tracing with the configured log level
            tracing_subscriber::fmt::init();

            let version = DateVersion::current();
            println!("🚀 Starting Claude Code Orchestra {}...", version);

            // Check if daemon is already running on a different port
            if let Ok(daemon_port) = cco::daemon::read_daemon_port() {
                if daemon_port != port {
                    eprintln!(
                        "⚠️  Warning: Daemon already running on port {}",
                        daemon_port
                    );
                    eprintln!("   Requested port: {}", port);
                    eprintln!("   Stop daemon first with: cco daemon stop");
                    eprintln!("   Or use the running daemon: cco tui");
                    std::process::exit(1);
                }
            }

            // Background check for updates (non-blocking)
            tokio::spawn(async {
                if let Ok(Some(latest)) = update::check_latest_version().await {
                    let current = DateVersion::current();
                    if latest != current {
                        println!(
                            "\nℹ️  New version available: {} (current: {})",
                            latest, current
                        );
                        println!("   Run 'cco update' to upgrade");
                    }
                }
            });

            // Step 1: Install server (idempotent)
            println!("📦 Installing server...");
            if let Err(e) = commands::server::install(false).await {
                eprintln!("⚠️  Server install failed: {}", e);
                eprintln!("   Attempting to continue anyway...");
            }

            // Step 2: Start server (idempotent)
            println!("🔌 Starting server on {}:{}...", host, port);
            if let Err(e) = commands::server::run(&host, port).await {
                eprintln!("❌ Server start failed: {}", e);
                eprintln!("   Cannot proceed without server.");
                std::process::exit(1);
            }

            // Step 3: Launch TUI
            println!("🎯 Launching TUI dashboard...");
            println!();

            match cco::TuiApp::new().await {
                Ok(mut app) => app.run().await,
                Err(e) => {
                    eprintln!("❌ Failed to start TUI: {}", e);
                    eprintln!("   Server is still running at http://{}:{}", host, port);
                    eprintln!("   You can access it via browser or run 'cco dashboard'");
                    std::process::exit(1);
                }
            }
        }

        Commands::Version => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();

            let version = DateVersion::current();
            println!("CCO version {}", version);
            println!("Build: Production");
            println!("Rust: 1.75+");

            // Background check for updates (non-blocking)
            tokio::spawn(async move {
                match update::check_latest_version().await {
                    Ok(Some(latest)) if latest != version => {
                        println!(
                            "\n⚠️  New version available: {} (current: {})",
                            latest, version
                        );
                        println!("   Run 'cco update' to upgrade");
                    }
                    Ok(Some(_)) => {
                        println!("\n✅ You have the latest version");
                    }
                    Ok(None) => {
                        println!("\n(Unable to check for updates)");
                    }
                    Err(_) => {} // Silent failure for version check
                }
            });

            // Give a moment for the check to complete
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            Ok(())
        }

        Commands::Health { host, port } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();

            // Auto-discover daemon port if using default values
            let actual_port = if host == "127.0.0.1" && port == 3000 {
                // User didn't specify custom host/port, try to discover daemon port
                match cco::daemon::read_daemon_port() {
                    Ok(discovered_port) => {
                        println!("Discovered daemon on port {}", discovered_port);
                        discovered_port
                    }
                    Err(_) => {
                        eprintln!("❌ Daemon not running");
                        eprintln!("   Start it with: cco daemon start");
                        std::process::exit(1);
                    }
                }
            } else {
                // User specified custom host/port, use as-is
                port
            };

            println!("Checking health of {}:{}", host, actual_port);

            let url = format!("http://{}:{}/health", host, actual_port);
            match reqwest::Client::new().get(&url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                println!("✅ Health check passed:");
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&json).unwrap_or_default()
                                );
                                Ok(())
                            }
                            Err(_) => {
                                println!("✅ Health check passed (HTTP {})", status);
                                Ok(())
                            }
                        }
                    } else {
                        eprintln!("❌ Health check failed: HTTP {}", status);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect to {}:{}: {}", host, actual_port, e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Credentials { action } => {
            // Initialize tracing for credentials commands
            tracing_subscriber::fmt::init();

            commands::credentials::run(action).await
        }

        Commands::Status => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();
            commands::status::run().await
        }

        Commands::Shutdown { port, all } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();
            commands::shutdown::run(port, all).await
        }

        Commands::Logs {
            port,
            follow,
            lines,
        } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();
            commands::logs::run(port, follow, lines).await
        }

        Commands::Dashboard {
            database: _,
            refresh_ms: _,
        } => {
            // Initialize tracing for dashboard
            tracing_subscriber::fmt::init();

            match cco::TuiApp::new().await {
                Ok(mut app) => app.run().await,
                Err(e) => {
                    eprintln!("❌ Failed to start TUI dashboard: {}", e);

                    // Try to discover daemon port for helpful error message
                    if let Ok(port) = cco::daemon::read_daemon_port() {
                        eprintln!(
                            "   Please use the web-based dashboard instead at http://localhost:{}",
                            port
                        );
                    } else {
                        eprintln!("   Start the daemon first: cco daemon start");
                    }

                    std::process::exit(1);
                }
            }
        }

        Commands::Daemon { action } => {
            // For DaemonAction::Run, tracing is initialized in the handler with file output
            // For other daemon actions, initialize tracing to console
            if !matches!(action, DaemonAction::Run) {
                tracing_subscriber::fmt::init();
            }
            match action {
                DaemonAction::Start {
                    port,
                    host,
                    cache_size,
                    cache_ttl,
                } => {
                    let config = cco::daemon::DaemonConfig {
                        port,
                        host,
                        log_level: "info".to_string(),
                        log_rotation_size: 10 * 1024 * 1024,
                        log_max_files: 5,
                        database_url: "sqlite://analytics.db".to_string(),
                        cache_size,
                        cache_ttl,
                        auto_start: true,
                        health_checks: true,
                        health_check_interval: 30,
                        hooks: Default::default(),
                    };

                    let manager = cco::daemon::DaemonManager::new(config);
                    manager.start().await
                }

                DaemonAction::Stop => {
                    let config = cco::daemon::load_config().unwrap_or_default();
                    let manager = cco::daemon::DaemonManager::new(config);
                    manager.stop().await
                }

                DaemonAction::Restart {
                    port,
                    host,
                    cache_size,
                    cache_ttl,
                } => {
                    let config = cco::daemon::DaemonConfig {
                        port,
                        host,
                        log_level: "info".to_string(),
                        log_rotation_size: 10 * 1024 * 1024,
                        log_max_files: 5,
                        database_url: "sqlite://analytics.db".to_string(),
                        cache_size,
                        cache_ttl,
                        auto_start: true,
                        health_checks: true,
                        health_check_interval: 30,
                        hooks: Default::default(),
                    };

                    let manager = cco::daemon::DaemonManager::new(config);
                    manager.restart().await
                }

                DaemonAction::Status => {
                    let config = cco::daemon::load_config().unwrap_or_default();
                    let manager = cco::daemon::DaemonManager::new(config);

                    match manager.get_status().await {
                        Ok(status) => {
                            println!("\n✅ Daemon Status:");
                            println!("   PID: {}", status.pid);
                            println!("   Running: {}", status.is_running);
                            println!("   Port: {}", status.port);
                            println!("   Version: {}", status.version);
                            println!("   Started at: {}", status.started_at);
                            println!();
                            Ok(())
                        }
                        Err(e) => {
                            println!("\n⚠️  Daemon is not running: {}", e);
                            Ok(())
                        }
                    }
                }

                DaemonAction::Logs { follow, lines } => {
                    let log_file = cco::daemon::get_daemon_log_file()?;

                    if !log_file.exists() {
                        eprintln!("No daemon log file found: {}", log_file.display());
                        return Ok(());
                    }

                    if follow {
                        let mut file = std::fs::File::open(&log_file)?;
                        file.seek(std::io::SeekFrom::End(0))?;

                        println!("Following daemon logs (Ctrl+C to stop)...\n");

                        let mut reader = std::io::BufReader::new(file);
                        let mut buffer = String::new();

                        loop {
                            match reader.read_line(&mut buffer) {
                                Ok(0) => {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                }
                                Ok(_) => {
                                    print!("{}", buffer);
                                    std::io::Write::flush(&mut std::io::stdout())?;
                                    buffer.clear();
                                }
                                Err(e) => {
                                    eprintln!("Error reading log file: {}", e);
                                    break;
                                }
                            }
                        }
                    } else {
                        let file = std::fs::File::open(&log_file)?;
                        let reader = std::io::BufReader::new(file);
                        let all_lines: Vec<String> =
                            reader.lines().collect::<Result<Vec<_>, _>>()?;

                        let start = if all_lines.len() > lines {
                            all_lines.len() - lines
                        } else {
                            0
                        };

                        for line in &all_lines[start..] {
                            println!("{}", line);
                        }
                    }

                    Ok(())
                }

                DaemonAction::Install => {
                    let service_manager = cco::daemon::service::get_service_manager()?;
                    service_manager.install()
                }

                DaemonAction::Uninstall => {
                    let service_manager = cco::daemon::service::get_service_manager()?;
                    service_manager.uninstall()
                }

                DaemonAction::Enable => {
                    let service_manager = cco::daemon::service::get_service_manager()?;
                    service_manager.enable()
                }

                DaemonAction::Disable => {
                    let service_manager = cco::daemon::service::get_service_manager()?;
                    service_manager.disable()
                }

                DaemonAction::Run => {
                    // This is called by the service manager to run the daemon in foreground
                    println!("🚀 Starting CCO daemon...");

                    let version = DateVersion::current();
                    println!("Version: {}", version);

                    // Initialize file-based logging BEFORE any daemon operations
                    if let Err(e) = cco::daemon::init_daemon_logging() {
                        eprintln!("Warning: Failed to initialize file logging: {}", e);
                        eprintln!("Falling back to console logging");
                        tracing_subscriber::fmt::init();
                    } else {
                        println!("✅ Logging configured: ~/.cco/logs/daemon.log");
                    }

                    // Load daemon configuration
                    let config = cco::daemon::load_config().unwrap_or_else(|e| {
                        eprintln!("Warning: Failed to load config, using defaults: {}", e);
                        cco::daemon::DaemonConfig::default()
                    });

                    // Run the daemon HTTP server (with CRUD classifier and hooks)
                    // Server internally updates PID file with actual port after binding
                    cco::daemon::run_daemon_server(config).await?;

                    Ok(())
                }
            }
        }

        Commands::Server { action } => {
            // Initialize tracing for server commands
            tracing_subscriber::fmt::init();

            match action {
                ServerAction::Install { force } => commands::server::install(force).await,

                ServerAction::Run { host, port } => commands::server::run(&host, port).await,

                ServerAction::Uninstall => commands::server::uninstall().await,
            }
        }

        Commands::Login => {
            // Initialize tracing for auth commands
            tracing_subscriber::fmt::init();

            cco::auth::login().await
        }

        Commands::Logout => {
            // Initialize tracing for auth commands
            tracing_subscriber::fmt::init();

            cco::auth::logout().await
        }

        Commands::Knowledge { action } => {
            // Initialize tracing for knowledge commands
            tracing_subscriber::fmt::init();

            commands::knowledge::run(action).await
        }

        Commands::LlmRouter { action } => {
            // Initialize tracing for LLM router commands
            tracing_subscriber::fmt::init();

            commands::llm_router::run(action).await
        }

        Commands::Orchestra { action } => {
            // Initialize tracing for orchestra commands
            tracing_subscriber::fmt::init();

            commands::orchestra::run(action).await
        }

        #[cfg(target_os = "macos")]
        Commands::SelfSign { path } => {
            // Initialize tracing
            tracing_subscriber::fmt::init();

            // Embedded entitlements for JIT/Metal support
            const ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
</dict>
</plist>
"#;

            let binary_path = match path {
                Some(p) => std::path::PathBuf::from(p),
                None => std::env::current_exe()?,
            };

            println!("🔐 Re-signing binary with JIT entitlements...");
            println!("   Binary: {}", binary_path.display());

            // Write entitlements to temp file
            let temp_dir = std::env::temp_dir();
            let entitlements_path = temp_dir.join("cco-entitlements.plist");
            std::fs::write(&entitlements_path, ENTITLEMENTS)?;

            // Run codesign
            let output = std::process::Command::new("codesign")
                .args([
                    "--force",
                    "--sign", "-",
                    "--entitlements", entitlements_path.to_str().unwrap(),
                    "--options", "runtime",
                    binary_path.to_str().unwrap(),
                ])
                .output()?;

            // Clean up temp file
            let _ = std::fs::remove_file(&entitlements_path);

            if output.status.success() {
                println!("✅ Binary signed successfully!");
                println!("\n   Entitlements applied:");
                println!("   - com.apple.security.cs.allow-jit");
                println!("   - com.apple.security.cs.allow-unsigned-executable-memory");
                println!("   - com.apple.security.cs.disable-library-validation");

                // Clear quarantine attribute
                let _ = std::process::Command::new("xattr")
                    .args(["-c", binary_path.to_str().unwrap()])
                    .output();

                println!("\n   Run 'cco --version' to verify.");
                Ok(())
            } else {
                eprintln!("❌ Signing failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
        }

        Commands::Orchestration { action } => {
            // Initialize tracing for orchestration commands
            tracing_subscriber::fmt::init();

            // Discover daemon port - orchestration is now integrated into daemon
            let daemon_port = match cco::daemon::read_daemon_port() {
                Ok(port) => port,
                Err(_) => {
                    eprintln!("❌ Daemon not running");
                    eprintln!("   Start the daemon first: cco daemon start");
                    eprintln!("   (Orchestration is now integrated into the daemon)");
                    std::process::exit(1);
                }
            };
            let base_url = format!("http://127.0.0.1:{}", daemon_port);

            match action {
                OrchestrationAction::GetContext {
                    issue_id,
                    agent_type,
                } => {
                    println!(
                        "📥 Getting context for {} (issue: {})",
                        agent_type, issue_id
                    );

                    let client = reqwest::Client::new();
                    let url = format!(
                        "{}/api/orchestration/context/{}/{}",
                        base_url, issue_id, agent_type
                    );

                    match client.get(&url).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                let json: serde_json::Value = response.json().await?;
                                println!("{}", serde_json::to_string_pretty(&json)?);
                                Ok(())
                            } else {
                                eprintln!("❌ Error: HTTP {}", response.status());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to connect: {}", e);
                            eprintln!("   Make sure the daemon is running (cco daemon start)");
                            std::process::exit(1);
                        }
                    }
                }

                OrchestrationAction::StoreResult {
                    issue_id,
                    agent_type,
                    file,
                } => {
                    println!("💾 Storing result for {} (issue: {})", agent_type, issue_id);

                    let result_json = std::fs::read_to_string(&file)?;
                    let result: serde_json::Value = serde_json::from_str(&result_json)?;

                    let client = reqwest::Client::new();
                    let url = format!("{}/api/orchestration/results", base_url);

                    let request_body = serde_json::json!({
                        "agent_id": uuid::Uuid::new_v4().to_string(),
                        "agent_type": agent_type,
                        "issue_id": issue_id,
                        "project_id": "default",
                        "result": result,
                        "timestamp": chrono::Utc::now(),
                    });

                    match client.post(&url).json(&request_body).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                let json: serde_json::Value = response.json().await?;
                                println!("✅ Result stored");
                                println!("{}", serde_json::to_string_pretty(&json)?);
                                Ok(())
                            } else {
                                eprintln!("❌ Error: HTTP {}", response.status());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to connect: {}", e);
                            std::process::exit(1);
                        }
                    }
                }

                OrchestrationAction::PublishEvent {
                    event_type,
                    publisher,
                    topic,
                    data,
                } => {
                    println!("📢 Publishing event: {}", event_type);

                    let event_data: serde_json::Value = serde_json::from_str(&data)?;

                    let client = reqwest::Client::new();
                    let url = format!("{}/api/orchestration/events/{}", base_url, event_type);

                    let request_body = serde_json::json!({
                        "event_type": event_type,
                        "publisher": publisher,
                        "topic": topic,
                        "data": event_data,
                    });

                    match client.post(&url).json(&request_body).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                let json: serde_json::Value = response.json().await?;
                                println!("✅ Event published");
                                println!("{}", serde_json::to_string_pretty(&json)?);
                                Ok(())
                            } else {
                                eprintln!("❌ Error: HTTP {}", response.status());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to connect: {}", e);
                            std::process::exit(1);
                        }
                    }
                }

                OrchestrationAction::Subscribe {
                    event_type,
                    timeout,
                } => {
                    println!("👂 Subscribing to events: {}", event_type);

                    let client = reqwest::Client::new();
                    let url = format!(
                        "{}/api/orchestration/events/wait/{}?timeout={}",
                        base_url, event_type, timeout
                    );

                    match client.get(&url).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                let json: serde_json::Value = response.json().await?;
                                println!("{}", serde_json::to_string_pretty(&json)?);
                                Ok(())
                            } else {
                                eprintln!("❌ Error: HTTP {}", response.status());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to connect: {}", e);
                            std::process::exit(1);
                        }
                    }
                }

                OrchestrationAction::Status => {
                    println!("📊 Orchestration System Status\n");

                    let client = reqwest::Client::new();
                    let health_url = format!("{}/health", base_url);
                    let status_url = format!("{}/api/orchestration/status", base_url);

                    // Check daemon health first
                    match client.get(&health_url).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                let json: serde_json::Value = response.json().await?;
                                println!("✅ Daemon is running on port {}", daemon_port);
                                println!("{}", serde_json::to_string_pretty(&json)?);
                            }
                        }
                        Err(_) => {
                            println!("❌ Daemon is not responding");
                            println!("   Start it with: cco daemon start");
                            return Ok(());
                        }
                    }

                    // Get orchestration-specific status
                    match client.get(&status_url).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                let json: serde_json::Value = response.json().await?;
                                println!("\n📈 Orchestration Status:");
                                println!("{}", serde_json::to_string_pretty(&json)?);
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  Could not get orchestration status: {}", e);
                        }
                    }

                    Ok(())
                }

                OrchestrationAction::ClearCache { issue_id } => {
                    println!("🗑️  Clearing cache for issue: {}", issue_id);

                    let client = reqwest::Client::new();
                    let url = format!("{}/api/orchestration/cache/context/{}", base_url, issue_id);

                    match client.delete(&url).send().await {
                        Ok(response) => {
                            if response.status().is_success() {
                                let json: serde_json::Value = response.json().await?;
                                println!("✅ Cache cleared");
                                println!("{}", serde_json::to_string_pretty(&json)?);
                                Ok(())
                            } else {
                                eprintln!("❌ Error: HTTP {}", response.status());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to connect: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }
}
