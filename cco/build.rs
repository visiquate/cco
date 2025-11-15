use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Rerun build if config changes
    println!("cargo:rerun-if-changed=../config/");
    println!("cargo:rerun-if-changed=../config/orchestra-config.json");

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

    // Set version - check environment variable first, otherwise use default
    let version = env::var("CCO_VERSION")
        .unwrap_or_else(|_| "2025.11.2".to_string());
    println!("cargo:rustc-env=CCO_VERSION={}", version);

    // Enable debug info in release builds for crash diagnostics
    println!("cargo:rustc-link-arg=-fPIC");

    // Embed config validation at compile time
    validate_configs();
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
