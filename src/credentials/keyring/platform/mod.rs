//! Platform-specific backend implementations

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

pub mod fallback;

use crate::credentials::keyring::backend::PlatformBackend;
use tracing::{info, warn};

/// Create appropriate backend for current platform
pub fn create_backend() -> Box<dyn PlatformBackend> {
    #[cfg(target_os = "macos")]
    {
        let backend = macos::MacOSKeychainBackend::new("com.visiquate.cco");
        if backend.is_available() {
            info!("Using macOS Keychain backend");
            return Box::new(backend);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let backend = linux::LinuxSecretServiceBackend::new("com.visiquate.cco");
        if backend.is_available() {
            info!("Using Linux Secret Service backend");
            return Box::new(backend);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(backend) = windows::WindowsDpapiBackend::new("com.visiquate.cco") {
            if backend.is_available() {
                info!("Using Windows DPAPI backend");
                return Box::new(backend);
            }
        }
    }

    // Fallback to encrypted file
    warn!("OS keyring unavailable, using encrypted file fallback");
    match fallback::EncryptedFileBackend::new() {
        Ok(backend) => Box::new(backend),
        Err(e) => {
            warn!("Failed to create fallback backend: {}", e);
            // Return a no-op backend to prevent panic
            Box::new(fallback::EncryptedFileBackend::new().unwrap_or_else(|_| {
                // This should never happen in practice, but provides a safe fallback
                panic!("Failed to initialize any credential backend")
            }))
        }
    }
}
