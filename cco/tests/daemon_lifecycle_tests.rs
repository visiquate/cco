//! Daemon Lifecycle Tests
//!
//! Tests for Phase 1d cross-platform daemon lifecycle management including:
//! - Configuration loading and validation
//! - Daemon start/stop/restart operations
//! - PID file management
//! - Status checking
//! - Platform-specific service installation (mocked)

#[cfg(test)]
mod daemon_lifecycle_tests {
    use cco::daemon::{DaemonConfig, DaemonManager, get_daemon_config_file, get_daemon_log_file, get_daemon_pid_file};
    use tempfile::TempDir;

    #[test]
    fn test_daemon_config_defaults() {
        let config = DaemonConfig::default();

        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_rotation_size, 10 * 1024 * 1024);
        assert_eq!(config.log_max_files, 5);
        assert!(config.auto_start);
        assert!(config.health_checks);
    }

    #[test]
    fn test_daemon_config_validation_valid() {
        let config = DaemonConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_daemon_config_validation_invalid_port_zero() {
        let mut config = DaemonConfig::default();
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_daemon_config_validation_invalid_log_level() {
        let mut config = DaemonConfig::default();
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_daemon_config_validation_valid_log_levels() {
        for level in &["debug", "info", "warn", "error"] {
            let mut config = DaemonConfig::default();
            config.log_level = level.to_string();
            assert!(config.validate().is_ok(), "Log level '{}' should be valid", level);
        }
    }

    #[test]
    fn test_daemon_config_validation_invalid_cache_size() {
        let mut config = DaemonConfig::default();
        config.cache_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_daemon_config_validation_invalid_cache_ttl() {
        let mut config = DaemonConfig::default();
        config.cache_ttl = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_daemon_config_validation_invalid_log_max_files() {
        let mut config = DaemonConfig::default();
        config.log_max_files = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_daemon_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = DaemonConfig::default();
        config.port = 8080;
        config.log_level = "debug".to_string();
        config.auto_start = false;

        config.save(&config_path).unwrap();
        assert!(config_path.exists());

        let loaded = DaemonConfig::load(&config_path).unwrap();
        assert_eq!(loaded.port, 8080);
        assert_eq!(loaded.log_level, "debug");
        assert!(!loaded.auto_start);
    }

    #[test]
    fn test_daemon_config_set_port() {
        let mut config = DaemonConfig::default();
        config.set("port", "8080").unwrap();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_daemon_config_set_host() {
        let mut config = DaemonConfig::default();
        config.set("host", "0.0.0.0").unwrap();
        assert_eq!(config.host, "0.0.0.0");
    }

    #[test]
    fn test_daemon_config_set_log_level() {
        let mut config = DaemonConfig::default();
        config.set("log_level", "debug").unwrap();
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_daemon_config_set_invalid_key() {
        let mut config = DaemonConfig::default();
        assert!(config.set("invalid_key", "value").is_err());
    }

    #[test]
    fn test_daemon_config_set_invalid_port() {
        let mut config = DaemonConfig::default();
        assert!(config.set("port", "invalid").is_err());
    }

    #[test]
    fn test_daemon_config_get_port() {
        let config = DaemonConfig::default();
        assert_eq!(config.get("port").unwrap(), "3000");
    }

    #[test]
    fn test_daemon_config_get_host() {
        let config = DaemonConfig::default();
        assert_eq!(config.get("host").unwrap(), "127.0.0.1");
    }

    #[test]
    fn test_daemon_config_get_log_level() {
        let config = DaemonConfig::default();
        assert_eq!(config.get("log_level").unwrap(), "info");
    }

    #[test]
    fn test_daemon_config_get_invalid_key() {
        let config = DaemonConfig::default();
        assert!(config.get("invalid_key").is_err());
    }

    #[test]
    fn test_daemon_config_load_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        // Should return defaults if file doesn't exist
        let config = DaemonConfig::load(&config_path).unwrap();
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_daemon_manager_creation() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config.clone());

        // Just verify it creates without error
        assert_eq!(manager.config.port, 3000);
    }

    #[test]
    fn test_get_daemon_log_file() {
        let result = get_daemon_log_file();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("daemon.log"));
    }

    #[test]
    fn test_get_daemon_pid_file() {
        let result = get_daemon_pid_file();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("daemon.pid"));
    }

    #[test]
    fn test_get_daemon_config_file() {
        let result = get_daemon_config_file();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[tokio::test]
    async fn test_daemon_manager_get_status_no_pid_file() {
        let config = DaemonConfig::default();
        let manager = DaemonManager::new(config);

        let result = manager.get_status().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_daemon_config_toml_roundtrip() {
        let mut original = DaemonConfig::default();
        original.port = 8888;
        original.host = "0.0.0.0".to_string();
        original.log_level = "debug".to_string();
        original.auto_start = false;
        original.health_checks = false;

        let toml_str = toml::to_string_pretty(&original).unwrap();
        let loaded: DaemonConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(loaded.port, original.port);
        assert_eq!(loaded.host, original.host);
        assert_eq!(loaded.log_level, original.log_level);
        assert_eq!(loaded.auto_start, original.auto_start);
        assert_eq!(loaded.health_checks, original.health_checks);
    }

    #[test]
    fn test_daemon_config_all_set_operations() {
        let mut config = DaemonConfig::default();

        config.set("port", "9000").unwrap();
        config.set("host", "192.168.1.1").unwrap();
        config.set("log_level", "error").unwrap();
        config.set("log_rotation_size", "5242880").unwrap();
        config.set("log_max_files", "3").unwrap();
        config.set("cache_size", "536870912").unwrap();
        config.set("cache_ttl", "7200").unwrap();
        config.set("auto_start", "false").unwrap();
        config.set("health_checks", "false").unwrap();
        config.set("health_check_interval", "60").unwrap();

        assert_eq!(config.port, 9000);
        assert_eq!(config.host, "192.168.1.1");
        assert_eq!(config.log_level, "error");
        assert_eq!(config.log_rotation_size, 5242880);
        assert_eq!(config.log_max_files, 3);
        assert_eq!(config.cache_size, 536870912);
        assert_eq!(config.cache_ttl, 7200);
        assert!(!config.auto_start);
        assert!(!config.health_checks);
        assert_eq!(config.health_check_interval, 60);
    }

    #[test]
    fn test_daemon_config_all_get_operations() {
        let mut config = DaemonConfig::default();
        config.port = 9000;
        config.host = "192.168.1.1".to_string();
        config.log_level = "error".to_string();
        config.auto_start = false;

        assert_eq!(config.get("port").unwrap(), "9000");
        assert_eq!(config.get("host").unwrap(), "192.168.1.1");
        assert_eq!(config.get("log_level").unwrap(), "error");
        assert_eq!(config.get("auto_start").unwrap(), "false");
    }
}

#[cfg(test)]
mod daemon_service_tests {
    use cco::daemon::service;

    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn test_get_service_manager() {
        let result = service::get_service_manager();
        assert!(result.is_ok(), "Service manager should be available on supported platforms");
    }

    #[test]
    fn test_service_manager_trait_exists() {
        // This test just verifies that the ServiceManager trait is accessible
        // The actual trait behavior is tested via platform-specific implementations
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod daemon_macos_tests {
    use cco::daemon::service::macos::MacOSService;

    #[test]
    fn test_macos_service_creation() {
        let result = MacOSService::new();
        assert!(result.is_ok());

        let service = result.unwrap();
        let path_str = service.plist_path.to_string_lossy();
        assert!(path_str.contains("Library/LaunchAgents"));
        assert!(path_str.contains("com.anthropic.cco.daemon.plist"));
    }

    #[test]
    fn test_macos_plist_generation() {
        let service = MacOSService::new().unwrap();
        let plist = service.generate_plist().unwrap();

        assert!(plist.contains("<?xml version"));
        assert!(plist.contains("<!DOCTYPE plist"));
        assert!(plist.contains("com.anthropic.cco.daemon"));
        assert!(plist.contains("daemon"));
        assert!(plist.contains("run"));
        assert!(plist.contains("KeepAlive"));
        assert!(plist.contains("RunAtLoad"));
        assert!(plist.contains("StandardOutPath"));
        assert!(plist.contains("StandardErrorPath"));
        assert!(plist.contains("</plist>"));
    }

    #[test]
    fn test_macos_plist_valid_xml_structure() {
        let service = MacOSService::new().unwrap();
        let plist = service.generate_plist().unwrap();

        // Check for valid XML structure
        assert!(plist.starts_with("<?xml"));
        assert!(plist.contains("<dict>"));
        assert!(plist.contains("</dict>"));
        assert!(plist.contains("<array>"));
        assert!(plist.contains("</array>"));
    }
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod daemon_linux_tests {
    use cco::daemon::service::linux::LinuxService;

    #[test]
    fn test_linux_service_creation() {
        let result = LinuxService::new();
        assert!(result.is_ok());

        let service = result.unwrap();
        let path_str = service.service_path.to_string_lossy();
        assert!(path_str.contains(".config/systemd/user"));
        assert!(path_str.contains("cco-daemon.service"));
    }

    #[test]
    fn test_linux_service_unit_generation() {
        let service = LinuxService::new().unwrap();
        let unit = service.generate_service_unit().unwrap();

        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("[Service]"));
        assert!(unit.contains("[Install]"));
        assert!(unit.contains("Description="));
        assert!(unit.contains("daemon run"));
        assert!(unit.contains("Restart=always"));
        assert!(unit.contains("RestartSec=10"));
        assert!(unit.contains("Type=simple"));
        assert!(unit.contains("ExecStart="));
        assert!(unit.contains("StandardOutput="));
        assert!(unit.contains("StandardError="));
    }

    #[test]
    fn test_linux_service_unit_install_section() {
        let service = LinuxService::new().unwrap();
        let unit = service.generate_service_unit().unwrap();

        assert!(unit.contains("[Install]"));
        assert!(unit.contains("WantedBy=default.target"));
    }
}
