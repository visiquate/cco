//! Platform-specific service management
//!
//! Provides abstraction for creating and managing system services on different platforms.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

use anyhow::Result;

/// Service management for a specific platform
pub trait ServiceManager {
    /// Install daemon as system service
    fn install(&self) -> Result<()>;

    /// Uninstall daemon from system service
    fn uninstall(&self) -> Result<()>;

    /// Check if service is installed
    fn is_installed(&self) -> Result<bool>;

    /// Enable service to start on boot
    fn enable(&self) -> Result<()>;

    /// Disable service from starting on boot
    fn disable(&self) -> Result<()>;
}

/// Platform-specific service implementation
#[cfg(target_os = "macos")]
pub type PlatformService = macos::MacOSService;

#[cfg(target_os = "linux")]
pub type PlatformService = linux::LinuxService;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub struct PlatformService;

/// Get platform service manager
#[cfg(target_os = "macos")]
pub fn get_service_manager() -> Result<Box<dyn ServiceManager>> {
    Ok(Box::new(macos::MacOSService::new()?))
}

#[cfg(target_os = "linux")]
pub fn get_service_manager() -> Result<Box<dyn ServiceManager>> {
    Ok(Box::new(linux::LinuxService::new()?))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn get_service_manager() -> Result<Box<dyn ServiceManager>> {
    anyhow::bail!("Service management is only supported on macOS and Linux")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn test_get_service_manager() {
        let result = get_service_manager();
        assert!(result.is_ok());
    }
}
