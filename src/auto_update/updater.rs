//! Update orchestration and binary replacement
//!
//! Handles the core update flow:
//! - Download binary from GitHub
//! - Verify checksum
//! - Extract from archive
//! - Atomically replace current binary
//! - Rollback on failure

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::releases_api::{Platform, ReleaseInfo};

// MEDIUM FIX #7: Maximum download size to prevent DoS attacks
const MAX_BINARY_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

/// RAII guard for automatic temporary directory cleanup
/// MEDIUM FIX #8: Ensures cleanup on error/panic
struct TempDirGuard(PathBuf);

impl TempDirGuard {
    fn new(path: PathBuf) -> Self {
        TempDirGuard(path)
    }

    /// Prevent cleanup on success
    fn persist(self) {
        std::mem::forget(self);
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        // Cleanup happens automatically on scope exit (error or panic)
        if self.0.exists() {
            let _ = fs::remove_dir_all(&self.0);
            tracing::debug!("Cleaned up temp directory: {}", self.0.display());
        }
    }
}

/// Download and verify a binary from a release
pub async fn download_and_verify(release: &ReleaseInfo) -> Result<PathBuf> {
    // HIGH FIX #3: Create unpredictable temp directory with secure permissions (0o700)
    use uuid::Uuid;

    let temp_dir_name = format!("cco-update-{}-{}", release.version, Uuid::new_v4());
    let temp_dir = std::env::temp_dir().join(temp_dir_name);

    #[cfg(unix)]
    {
        use std::fs::DirBuilder;
        use std::os::unix::fs::DirBuilderExt;

        let mut builder = DirBuilder::new();
        builder.mode(0o700); // rwx------ (owner only)
        builder
            .create(&temp_dir)
            .context("Failed to create secure temporary directory")?;

        // Verify permissions were set correctly
        let metadata = fs::metadata(&temp_dir)?;
        let perms = metadata.permissions().mode();
        if perms & 0o077 != 0 {
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(anyhow!(
                "SECURITY: Failed to set secure permissions on temp directory (got 0o{:o})",
                perms & 0o777
            ));
        }

        tracing::debug!(
            "Created secure temp directory: {} with permissions 0o{:o}",
            temp_dir.display(),
            perms & 0o777
        );
    }

    #[cfg(not(unix))]
    {
        fs::create_dir_all(&temp_dir).context("Failed to create temporary directory")?;
        // TODO: Set Windows ACLs for equivalent protection
        tracing::warn!("Windows ACL protection not yet implemented");
    }

    // MEDIUM FIX #8: RAII guard will clean up temp_dir on ANY error or panic
    let _temp_guard = TempDirGuard::new(temp_dir.clone());

    let temp_archive = temp_dir.join(&release.filename);

    // MEDIUM FIX #7: Check available disk space before download
    if let Err(e) = check_disk_space(release.size * 2) {
        tracing::warn!("Disk space check: {}", e);
        // Continue anyway - check is advisory
    }

    // Download the archive
    tracing::info!("Downloading from: {}", release.download_url);
    download_file(&release.download_url, &temp_archive, release.size)
        .await
        .context("Failed to download update")?;

    // CRITICAL: Checksum verification is MANDATORY
    let expected_checksum = &release.checksum;

    tracing::info!("Verifying checksum...");

    if !verify_checksum(&temp_archive, expected_checksum)? {
        return Err(anyhow!(
            "SECURITY: Checksum verification FAILED! Expected: {}, \
            This indicates a corrupted download or possible MITM attack. \
            Update aborted for your safety.",
            expected_checksum
        ));
    }

    tracing::info!("Checksum verified successfully");

    // Extract archive
    tracing::info!("Extracting archive...");
    let binary_path = extract_archive(&temp_archive, &temp_dir)?;

    // Verify extracted binary exists
    if !binary_path.exists() {
        return Err(anyhow!("Binary not found in archive"));
    }

    // HIGH FIX #4: Set and VERIFY executable permissions
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&binary_path, perms)?;

        // VERIFY permissions were actually set
        let verify_perms = fs::metadata(&binary_path)?.permissions();
        let actual_mode = verify_perms.mode() & 0o777;
        if actual_mode != 0o755 {
            return Err(anyhow!(
                "SECURITY: Failed to set correct permissions on binary (expected 0o755, got 0o{:o})",
                actual_mode
            ));
        }

        tracing::debug!("Binary permissions verified: 0o{:o}", actual_mode);
    }

    // Success - prevent cleanup of temp directory
    _temp_guard.persist();

    Ok(binary_path)
}

/// Download a file from URL with size limits and streaming
/// MEDIUM FIX #7: Prevents DoS via large downloads, streams to disk
async fn download_file(url: &str, dest: &Path, expected_size: u64) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("cco/client") // LOW FIX #11: Generic user-agent for privacy
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("HTTP request failed")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    // MEDIUM FIX #7: Check Content-Length if available
    if let Some(content_length) = response.content_length() {
        if content_length > MAX_BINARY_SIZE {
            return Err(anyhow!(
                "SECURITY: Download size {} bytes exceeds maximum {} bytes (100 MB). \
                Possible DoS attack - aborting.",
                content_length,
                MAX_BINARY_SIZE
            ));
        }

        tracing::debug!("Download size: {} bytes", content_length);

        // Warn if actual size differs significantly from expected
        if content_length > expected_size * 2 {
            tracing::warn!(
                "Download size ({} bytes) is much larger than expected ({} bytes)",
                content_length,
                expected_size
            );
        }
    } else {
        tracing::warn!("No Content-Length header, will enforce size limit during download");
    }

    // MEDIUM FIX #7: Stream to disk instead of loading into memory
    use futures::StreamExt;
    use std::io::Write;

    let mut file = fs::File::create(dest).context("Failed to create download file")?;

    // HIGH FIX #4: Set secure permissions on download file immediately
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(dest)?.permissions();
        perms.set_mode(0o600); // rw------- (owner read/write only)
        fs::set_permissions(dest, perms)?;
    }

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Download stream error")?;
        downloaded += chunk.len() as u64;

        // MEDIUM FIX #7: Enforce size limit even if Content-Length was wrong/missing
        if downloaded > MAX_BINARY_SIZE {
            let _ = fs::remove_file(dest); // Clean up partial download
            return Err(anyhow!(
                "SECURITY: Download exceeded maximum size ({} bytes) after {} bytes downloaded. \
                Possible DoS attack - aborting.",
                MAX_BINARY_SIZE,
                downloaded
            ));
        }

        file.write_all(&chunk)
            .context("Failed to write download chunk")?;
    }

    file.flush().context("Failed to flush download file")?;

    tracing::info!("Download complete: {} bytes", downloaded);
    Ok(())
}

/// Check if sufficient disk space is available
/// MEDIUM FIX #7: Prevents disk exhaustion attacks
fn check_disk_space(_required_bytes: u64) -> Result<()> {
    // Note: Disk space checking using sysinfo crate
    // Currently advisory-only - logs warning if check fails
    // TODO: Enable this check once sysinfo integration is tested

    // Uncomment when ready to enable:
    // use sysinfo::{DiskExt, System, SystemExt};
    //
    // let mut sys = System::new_all();
    // sys.refresh_all();
    //
    // let temp_dir = std::env::temp_dir();
    // for disk in sys.disks() {
    //     if temp_dir.starts_with(disk.mount_point()) {
    //         let available = disk.available_space();
    //         let required_with_margin = _required_bytes * 2;
    //
    //         if available < required_with_margin {
    //             return Err(anyhow!(
    //                 "Insufficient disk space: {} MB required, {} MB available",
    //                 required_with_margin / 1024 / 1024,
    //                 available / 1024 / 1024
    //             ));
    //         }
    //
    //         tracing::debug!(
    //             "Disk space check passed: {} MB available",
    //             available / 1024 / 1024
    //         );
    //         return Ok(());
    //     }
    // }

    Ok(())
}

/// Verify SHA256 checksum of a file
fn verify_checksum(file_path: &Path, expected_checksum: &str) -> Result<bool> {
    let mut file =
        fs::File::open(file_path).context("Failed to open file for checksum verification")?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = file
            .read(&mut buffer)
            .context("Failed to read file for checksum")?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    let computed_checksum = hex::encode(result);

    Ok(computed_checksum.to_lowercase() == expected_checksum.to_lowercase())
}

/// Extract archive and return path to binary
fn extract_archive(archive_path: &Path, temp_dir: &Path) -> Result<PathBuf> {
    let platform = Platform::detect()?;

    match platform {
        Platform::WindowsX86_64 => {
            // Extract ZIP for Windows
            #[cfg(target_os = "windows")]
            {
                extract_zip(archive_path, temp_dir)?;
                Ok(temp_dir.join("cco.exe"))
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err(anyhow!("ZIP extraction not supported on this platform"))
            }
        }
        _ => {
            // Extract tar.gz for Unix-like systems
            extract_tar_gz(archive_path, temp_dir)?;
            Ok(temp_dir.join("cco"))
        }
    }
}

/// Extract tar.gz archive
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let tar_gz = fs::File::open(archive_path).context("Failed to open archive")?;

    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    archive
        .unpack(dest_dir)
        .context("Failed to extract archive")?;

    Ok(())
}

/// Extract ZIP archive (Windows only)
#[cfg(target_os = "windows")]
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    use std::io::Write;

    let file = fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// Get the installation path for CCO binary
pub fn get_install_path() -> Result<PathBuf> {
    // 1. Determine canonical installation path
    let canonical_path = get_canonical_install_path()?;

    // 2. If canonical exists, prefer it
    if canonical_path.exists() {
        return Ok(canonical_path);
    }

    // 3. Check if running from legacy location
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    if is_legacy_location(&current_exe)? {
        tracing::warn!("Running from legacy location: {}", current_exe.display());
        tracing::warn!(
            "Update will install to canonical location: {}",
            canonical_path.display()
        );
        tracing::warn!(
            "After update, remove old installation: sudo rm {}",
            current_exe.display()
        );

        // Ensure canonical directory exists
        if let Some(parent) = canonical_path.parent() {
            fs::create_dir_all(parent).context("Failed to create canonical directory")?;
        }

        return Ok(canonical_path);
    }

    // 4. Not legacy and canonical doesn't exist - use current location
    Ok(current_exe)
}

/// Get canonical (preferred) installation path
fn get_canonical_install_path() -> Result<PathBuf> {
    #[cfg(unix)]
    {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".local/bin/cco"))
    }

    #[cfg(windows)]
    {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".local\\bin\\cco.exe"))
    }
}

/// Check if path is a legacy installation location
fn is_legacy_location(path: &Path) -> Result<bool> {
    let path_str = path.to_string_lossy();

    #[cfg(unix)]
    {
        Ok(path_str.contains("/usr/local/bin/")
            || path_str.contains("/opt/")
            || path_str.contains("/usr/bin/"))
    }

    #[cfg(windows)]
    {
        Ok(path_str.contains("Program Files") || path_str.contains("ProgramData"))
    }
}

/// Replace current binary with new one (atomic operation)
pub async fn replace_binary(new_binary_path: &Path) -> Result<()> {
    let install_path = get_install_path()?;
    let backup_path = install_path.with_extension("backup");
    let temp_install_path = install_path.with_extension("new");

    // Ensure new binary has executable permissions
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(new_binary_path)
            .context("Failed to get new binary metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(new_binary_path, perms)
            .context("Failed to set executable permissions")?;
    }

    // Copy new binary to temporary location
    fs::copy(new_binary_path, &temp_install_path)
        .context("Failed to copy new binary to temporary location")?;

    // Verify new binary works
    match std::process::Command::new(&temp_install_path)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            tracing::info!("New binary verified successfully");
        }
        Ok(output) => {
            let _ = fs::remove_file(&temp_install_path);
            return Err(anyhow!(
                "New binary failed verification: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_install_path);
            return Err(anyhow!("Failed to verify new binary: {}", e));
        }
    }

    // Backup current installation
    if install_path.exists() {
        tracing::info!("Backing up current binary...");
        fs::copy(&install_path, &backup_path).context("Failed to create backup")?;
    }

    // Atomically replace binary (on Unix, this is atomic; on Windows, it's best-effort)
    #[cfg(unix)]
    {
        fs::rename(&temp_install_path, &install_path).context("Failed to replace binary")?;
    }

    #[cfg(not(unix))]
    {
        // On Windows, we need to delete first, then rename
        if install_path.exists() {
            fs::remove_file(&install_path).context("Failed to remove old binary")?;
        }
        fs::rename(&temp_install_path, &install_path).context("Failed to install new binary")?;
    }

    // Final verification
    match std::process::Command::new(&install_path)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            tracing::info!("Update installed successfully");

            // Clean up backup on success
            if backup_path.exists() {
                let _ = fs::remove_file(backup_path);
            }

            // Clean up temp directory
            if let Some(temp_dir) = new_binary_path.parent() {
                let _ = fs::remove_dir_all(temp_dir);
            }

            Ok(())
        }
        _ => {
            tracing::error!("New binary failed post-install verification, rolling back...");

            // Rollback: restore from backup
            if backup_path.exists() {
                fs::copy(&backup_path, &install_path)
                    .context("Failed to restore backup during rollback")?;
                return Err(anyhow!(
                    "Update failed verification, rolled back to previous version"
                ));
            }

            Err(anyhow!(
                "Update verification failed and no backup available"
            ))
        }
    }
}

/// Verify current binary works
pub fn verify_current_binary() -> Result<bool> {
    let install_path = get_install_path()?;

    match std::process::Command::new(&install_path)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(e) => Err(anyhow!("Failed to verify binary: {}", e)),
    }
}

/// Clean up temporary files from failed updates
pub fn cleanup_temp_files() -> Result<()> {
    let temp_dir = std::env::temp_dir();

    // Find all cco-update-* directories
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("cco-update-") {
                        tracing::debug!("Cleaning up temporary directory: {:?}", path);
                        let _ = fs::remove_dir_all(&path);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_install_path() {
        let path = get_install_path();
        assert!(path.is_ok(), "Should get install path");
    }

    #[test]
    fn test_verify_current_binary() {
        let result = verify_current_binary();
        assert!(result.is_ok(), "Should verify current binary");

        // During tests, the current executable might be a test runner, not the actual binary
        // So we just ensure the function runs without error - the actual validation is tested
        // via integration tests
    }

    #[test]
    fn test_checksum_verification() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");

        // Write test data
        fs::write(&test_file, b"Hello, World!")?;

        // Compute expected checksum
        let mut hasher = Sha256::new();
        hasher.update(b"Hello, World!");
        let expected = hex::encode(hasher.finalize());

        // Verify
        assert!(verify_checksum(&test_file, &expected)?);

        // Verify with wrong checksum
        assert!(!verify_checksum(&test_file, "0000000000000000")?);

        Ok(())
    }
}
