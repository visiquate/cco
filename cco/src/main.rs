//! CCO CLI - Claude Code Orchestra Command Line Interface

use clap::{Parser, Subcommand};

mod install;
mod update;
mod auto_update;
mod commands;

// Import server module and version
use cco::server::run_server;
use cco::version::DateVersion;

#[derive(Parser)]
#[command(name = "cco")]
#[command(about = "Claude Code Orchestra - Multi-agent development system with intelligent caching")]
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

        /// Auto-confirm installation
        #[arg(long)]
        yes: bool,

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
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Start background update check (non-blocking)
    auto_update::check_for_updates_async();

    // If no command specified, default to run
    let command = cli.command.unwrap_or(Commands::Run {
        port: 3000,
        host: "127.0.0.1".to_string(),
        database_url: "sqlite://analytics.db".to_string(),
        cache_size: 1073741824,
        cache_ttl: 3600,
    });

    match command {
        Commands::Install { force } => {
            install::run(force).await
        }

        Commands::Update { check, yes, channel } => {
            update::run(check, yes, channel).await
        }

        Commands::Config { action } => match action {
            ConfigAction::Set { key, value } => {
                auto_update::set_config(&key, &value)
            }
            ConfigAction::Get { key } => {
                auto_update::get_config(&key)?;
                Ok(())
            }
            ConfigAction::Show => {
                auto_update::show_config()
            }
        },

        Commands::Run {
            port,
            host,
            database_url: _database_url,
            cache_size,
            cache_ttl,
        } => {
            let version = DateVersion::current();
            println!("🚀 Starting Claude Code Orchestra {}...", version);

            // Background check for updates (non-blocking)
            tokio::spawn(async {
                if let Ok(Some(latest)) = update::check_latest_version().await {
                    let current = DateVersion::current();
                    if latest != current {
                        println!("\nℹ️  New version available: {} (current: {})", latest, current);
                        println!("   Run 'cco update' to upgrade");
                    }
                }
            });

            // Auto-open browser after a short delay
            let url = format!("http://{}:{}", host, port);
            let url_clone = url.clone();
            tokio::spawn(async move {
                // Wait a moment for the server to start
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                println!("🌐 Opening browser at {}...", url_clone);
                if let Err(e) = webbrowser::open(&url_clone) {
                    eprintln!("⚠️  Failed to open browser: {}", e);
                    eprintln!("   Please manually navigate to: {}", url_clone);
                }
            });

            // Run the actual HTTP server
            run_server(&host, port, cache_size, cache_ttl).await
        }

        Commands::Version => {
            let version = DateVersion::current();
            println!("CCO version {}", version);
            println!("Build: Production");
            println!("Rust: 1.75+");

            // Background check for updates (non-blocking)
            tokio::spawn(async move {
                match update::check_latest_version().await {
                    Ok(Some(latest)) if latest != version => {
                        println!("\n⚠️  New version available: {} (current: {})", latest, version);
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
            println!("Checking health of {}:{}", host, port);
            // In a real implementation, we would make an HTTP request to /health
            println!("Health check would go to http://{}:{}/health", host, port);
            Ok(())
        }

        Commands::Credentials { action } => match action {
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
        },

        Commands::Status => {
            commands::status::run().await
        }

        Commands::Shutdown { port, all } => {
            commands::shutdown::run(port, all).await
        }

        Commands::Logs { port, follow, lines } => {
            commands::logs::run(port, follow, lines).await
        }
    }
}
