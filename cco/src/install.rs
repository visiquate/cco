//! Installation module for CCO
//!
//! Handles self-installation of the CCO binary to ~/.local/bin/
//! and updates shell rc files for PATH configuration.

use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Detect the user's current shell
fn detect_shell() -> Result<String> {
    let shell =
        env::var("SHELL").context("Could not detect shell from $SHELL environment variable")?;

    let shell_name = Path::new(&shell)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Could not parse shell name"))?
        .to_string();

    Ok(shell_name)
}

/// Get the appropriate shell RC file path
fn get_shell_rc_path(shell: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    let rc_file = match shell {
        "zsh" => home.join(".zshrc"),
        "bash" => {
            // macOS typically uses .bash_profile, Linux uses .bashrc
            if cfg!(target_os = "macos") {
                home.join(".bash_profile")
            } else {
                home.join(".bashrc")
            }
        }
        "fish" => home.join(".config/fish/config.fish"),
        _ => return Err(anyhow!("Unsupported shell: {}", shell)),
    };

    Ok(rc_file)
}

/// Check if ~/.local/bin is already in PATH
fn is_in_path() -> bool {
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local/bin");
        if let Ok(path) = env::var("PATH") {
            return path.split(':').any(|p| Path::new(p) == local_bin);
        }
    }
    false
}

/// Update shell RC file to add ~/.local/bin to PATH
fn update_shell_rc(shell: &str) -> Result<()> {
    let rc_path = get_shell_rc_path(shell)?;

    // Create parent directory if needed (for fish)
    if let Some(parent) = rc_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read existing content
    let existing_content = if rc_path.exists() {
        fs::read_to_string(&rc_path)?
    } else {
        String::new()
    };

    // Check if PATH export already exists
    let path_exports = vec![
        r#"export PATH="$HOME/.local/bin:$PATH""#,
        r#"export PATH="~/.local/bin:$PATH""#,
        "set -gx PATH $HOME/.local/bin $PATH", // fish syntax
    ];

    for export in &path_exports {
        if existing_content.contains(export) {
            println!("  PATH already configured in {}", rc_path.display());
            return Ok(());
        }
    }

    // Append PATH export to RC file
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_path)?;

    let export_line = if shell == "fish" {
        "\n# Added by CCO installer\nset -gx PATH $HOME/.local/bin $PATH\n"
    } else {
        "\n# Added by CCO installer\nexport PATH=\"$HOME/.local/bin:$PATH\"\n"
    };

    file.write_all(export_line.as_bytes())?;

    println!("  Updated {} with PATH configuration", rc_path.display());

    Ok(())
}

/// Install CCO binary to ~/.local/bin
pub async fn run(force: bool) -> Result<()> {
    println!("→ Installing CCO v{}...", env!("CARGO_PKG_VERSION"));

    // Get the current executable path
    let current_exe = env::current_exe().context("Could not determine current executable path")?;

    // Determine installation directory
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let install_dir = home.join(".local/bin");
    let install_path = install_dir.join("cco");

    // Create installation directory
    println!("→ Creating {}/", install_dir.display());
    fs::create_dir_all(&install_dir).context("Could not create installation directory")?;

    // Check if binary already exists
    if install_path.exists() && !force {
        println!("⚠️  CCO is already installed at {}", install_path.display());
        println!("   Use --force to reinstall");
        return Ok(());
    }

    // Copy binary to installation directory
    println!("→ Copying binary to {}", install_path.display());
    fs::copy(&current_exe, &install_path)
        .context("Could not copy binary to installation directory")?;

    // Set executable permissions (Unix only)
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&install_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_path, perms)?;
    }

    // Detect shell and update PATH
    match detect_shell() {
        Ok(shell) => {
            println!("→ Detected shell: {}", shell);

            // Check if already in PATH at runtime
            if is_in_path() {
                println!("  ~/.local/bin is already in PATH");
            } else {
                match update_shell_rc(&shell) {
                    Ok(_) => {
                        println!("\n✅ Installation complete!");
                        println!("\nTo start using CCO:");
                        println!("  1. Restart your terminal, or");
                        println!("  2. Run: source {}", get_shell_rc_path(&shell)?.display());
                        println!("\nVerify with: cco version");
                    }
                    Err(e) => {
                        println!(
                            "\n⚠️  Could not automatically update shell configuration: {}",
                            e
                        );
                        println!("\nManually add this to your shell RC file:");
                        println!(r#"  export PATH="$HOME/.local/bin:$PATH""#);
                    }
                }
            }
        }
        Err(_) => {
            println!("\n⚠️  Could not detect shell");
            println!("\nManually add this to your shell RC file:");
            println!(r#"  export PATH="$HOME/.local/bin:$PATH""#);
        }
    }

    if is_in_path() {
        println!("\n✅ Installation complete!");
        println!("\nVerify with: cco version");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell() {
        // This will succeed if SHELL is set
        if env::var("SHELL").is_ok() {
            assert!(detect_shell().is_ok());
        }
    }

    #[test]
    fn test_shell_rc_paths() {
        if let Ok(_) = get_shell_rc_path("zsh") {
            // Path should end with .zshrc
        }
        if let Ok(_) = get_shell_rc_path("bash") {
            // Path should end with .bashrc or .bash_profile
        }
    }
}
