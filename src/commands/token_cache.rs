use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Ensure the cache directory exists and is restricted to the current user.
fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create token cache directory")?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(parent)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(parent, perms)
                .context("Failed to secure token cache directory permissions")?;
        }
    }

    Ok(())
}

/// Tighten permissions on an existing cache file (best-effort on non-Unix).
pub fn tighten_permissions_if_exists(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        if path.exists() {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)
                .context("Failed to set token cache permissions to 600")?;
        }
    }

    Ok(())
}

/// Write cache content with 0600 permissions to avoid credential leakage.
pub fn write_secure_cache(path: &Path, contents: &str) -> Result<()> {
    ensure_parent_dir(path)?;

    fs::write(path, contents).context("Failed to write token cache")?;
    tighten_permissions_if_exists(path)?;

    Ok(())
}
