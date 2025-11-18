//! CCO CLI - Claude Code Orchestra Command Line Interface

use clap::{Parser, Subcommand};
use std::io::{BufRead, Seek};

// Binary-specific modules
mod commands;
mod install;
mod update;

// Import from library
use cco::auto_update;
use cco::server::run_server;
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
}

#[derive(Subcommand)]
enum Commands {
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
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Manage the CCO HTTP server (install, run, uninstall)
    Server {
        #[command(subcommand)]
        action: ServerAction,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon
    Start {
        /// Port to listen on (default: 3000)
        #[arg(short, long, default_value = "3000")]
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
    Restart {
        /// Port to listen on (default: 3000)
        #[arg(short, long, default_value = "3000")]
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
    /// List all credentials
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Check for updates synchronously BEFORE launching anything
    // This ensures we're always running the latest version
    // Note: Uses config settings (enabled, auto_install, check_interval)
    // To disable: Set CCO_AUTO_UPDATE=false environment variable
    auto_update::check_for_updates_blocking().await;

    // If no command specified, default to install + run server + launch TUI
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // Initialize tracing early for default command
            tracing_subscriber::fmt::init();

            let version = DateVersion::current();
            println!("🚀 Starting Claude Code Orchestra {}...", version);

            // Step 1: Install server (idempotent)
            println!("📦 Installing server...");
            if let Err(e) = commands::server::install(false).await {
                eprintln!("⚠️  Server install failed: {}", e);
                eprintln!("   Attempting to continue anyway...");
            }

            // Step 2: Start server (idempotent)
            println!("🔌 Starting server on 127.0.0.1:3000...");
            if let Err(e) = commands::server::run("127.0.0.1", 3000).await {
                eprintln!("❌ Server start failed: {}", e);
                eprintln!("   Cannot proceed without server.");
                std::process::exit(1);
            }

            // Step 3: Launch TUI
            println!("🎯 Launching TUI dashboard...");
            println!();

            match cco::TuiApp::new().await {
                Ok(mut app) => {
                    return app.run().await;
                }
                Err(e) => {
                    eprintln!("❌ TUI failed to start: {}", e);
                    eprintln!("   Server is still running at http://127.0.0.1:3000");
                    eprintln!("   You can access it via browser or run 'cco dashboard'");
                    std::process::exit(1);
                }
            }
        }
    };

    match command {
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

            println!("Checking health of {}:{}", host, port);

            let url = format!("http://{}:{}/health", host, port);
            match reqwest::Client::new().get(&url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                println!("✅ Health check passed:");
                                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
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
                    eprintln!("❌ Failed to connect to {}:{}: {}", host, port, e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Credentials { action } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();

            match action {
                CredentialAction::Store { key, value: _ } => {
                    println!("Storing credential: {}", key);
                    // In a real implementation, we would encrypt and store the credential
                    Ok(())
                }
                CredentialAction::Retrieve { key } => {
                    println!("Retrieving credential: {}", key);
                    // In a real implementation, we would retrieve and decrypt the credential
                    Ok(())
                }
                CredentialAction::List => {
                    println!("Listing credentials");
                    // In a real implementation, we would list all stored credentials
                    Ok(())
                }
            }
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

        Commands::Dashboard { database: _, refresh_ms: _ } => {
            // Initialize tracing for dashboard
            tracing_subscriber::fmt::init();

            match cco::TuiApp::new().await {
                Ok(mut app) => app.run().await,
                Err(e) => {
                    eprintln!("❌ Failed to start TUI dashboard: {}", e);
                    eprintln!("   Please use the web-based dashboard instead at http://localhost:3000");
                    std::process::exit(1);
                }
            }
        }

        Commands::Daemon { action } => {
            // Initialize tracing for other commands
            tracing_subscriber::fmt::init();
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
                                    tokio::time::sleep(std::time::Duration::from_millis(100))
                                        .await;
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
                        let all_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

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

                    let url = "http://127.0.0.1:3000";
                    println!("Dashboard: {}", url);

                    // Run the actual HTTP server
                    run_server("127.0.0.1", 3000, 1073741824, 3600, false).await
                }
            }
        }

        Commands::Server { action } => {
            // Initialize tracing for server commands
            tracing_subscriber::fmt::init();

            match action {
                ServerAction::Install { force } => {
                    commands::server::install(force).await
                }

                ServerAction::Run { host, port } => {
                    commands::server::run(&host, port).await
                }

                ServerAction::Uninstall => {
                    commands::server::uninstall().await
                }
            }
        }
    }
}
