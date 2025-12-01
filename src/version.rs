//! Version management module for CCO
//!
//! Handles date-based versioning format (YYYY.MM.N) and version comparisons.
//! N is the release counter for that month (starts at 1 each month).

use anyhow::Result;
use std::cmp::Ordering;

/// Date-based version structure: YYYY.MM.N+<git-hash>
/// - YYYY: Year (e.g., 2025)
/// - MM: Month (1-12)
/// - N: Release counter for that month (starts at 1)
/// - git-hash: Optional short git commit hash (7 characters)
///
/// Note: PartialEq and Eq compare only the semantic version (year, month, release),
/// not the git hash. This allows versions to be equal even if built from different commits.
#[derive(Debug, Clone)]
pub struct DateVersion {
    year: u32,
    month: u32,
    release: u32,
    git_hash: Option<String>,
}

impl DateVersion {
    /// Parse version string in format "YYYY.MM.N" or "YYYY.MM.N+<git-hash>"
    ///
    /// # Examples
    ///
    /// ```
    /// use cco::DateVersion;
    ///
    /// let v = DateVersion::parse("2025.11.1").unwrap();
    /// assert_eq!(v.year(), 2025);
    /// assert_eq!(v.month(), 11);
    /// assert_eq!(v.release(), 1);
    ///
    /// let v2 = DateVersion::parse("2025.11.1+abc123d").unwrap();
    /// assert_eq!(v2.year(), 2025);
    /// assert_eq!(v2.git_hash(), Some("abc123d"));
    /// ```
    pub fn parse(version_str: &str) -> Result<Self> {
        // Split on '+' to separate version from optional git hash
        let parts: Vec<&str> = version_str.split('+').collect();
        let base_version = parts[0];
        let git_hash = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };

        // Parse "2025.11.1" format
        let version_components: Vec<&str> = base_version.split('.').collect();
        if version_components.len() != 3 {
            anyhow::bail!(
                "Invalid version format: {} (expected YYYY.MM.N or YYYY.MM.N+<git-hash>)",
                version_str
            );
        }

        let year: u32 = version_components[0]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid year: {}", version_components[0]))?;
        let month: u32 = version_components[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid month: {}", version_components[1]))?;
        let release: u32 = version_components[2]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid release number: {}", version_components[2]))?;

        if month < 1 || month > 12 {
            anyhow::bail!("Invalid month: {} (must be 1-12)", month);
        }

        Ok(Self {
            year,
            month,
            release,
            git_hash,
        })
    }

    /// Get current version from environment or default
    pub fn current() -> &'static str {
        env!("CCO_VERSION")
    }

    /// Get year component
    pub fn year(&self) -> u32 {
        self.year
    }

    /// Get month component
    pub fn month(&self) -> u32 {
        self.month
    }

    /// Get release number component (release counter for the month)
    pub fn release(&self) -> u32 {
        self.release
    }

    /// Get git hash (if present)
    pub fn git_hash(&self) -> Option<&str> {
        self.git_hash.as_deref()
    }
}

impl PartialEq for DateVersion {
    fn eq(&self, other: &Self) -> bool {
        // Compare only semantic version parts, ignore git hash
        self.year == other.year && self.month == other.month && self.release == other.release
    }
}

impl Eq for DateVersion {}

impl PartialOrd for DateVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare year first
        match self.year.cmp(&other.year) {
            Ordering::Equal => {
                // Then month
                match self.month.cmp(&other.month) {
                    Ordering::Equal => {
                        // Finally release number
                        // Note: git_hash is NOT compared - it's metadata, not version ordering
                        self.release.cmp(&other.release)
                    }
                    other => other,
                }
            }
            other => other,
        }
    }
}

impl std::fmt::Display for DateVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(hash) = &self.git_hash {
            write!(f, "{}.{}.{}+{}", self.year, self.month, self.release, hash)
        } else {
            write!(f, "{}.{}.{}", self.year, self.month, self.release)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = DateVersion::parse("2025.11.1").unwrap();
        assert_eq!(v.year(), 2025);
        assert_eq!(v.month(), 11);
        assert_eq!(v.release(), 1);
        assert_eq!(v.git_hash(), None);

        // Test with git hash
        let v2 = DateVersion::parse("2025.11.1+abc123d").unwrap();
        assert_eq!(v2.year(), 2025);
        assert_eq!(v2.month(), 11);
        assert_eq!(v2.release(), 1);
        assert_eq!(v2.git_hash(), Some("abc123d"));
    }

    #[test]
    fn test_version_parsing_errors() {
        assert!(DateVersion::parse("2025.11.1.99").is_err()); // Too many components
        assert!(DateVersion::parse("2025.11.1-99").is_err()); // Old format with hyphen
        assert!(DateVersion::parse("2025.11").is_err()); // Missing release number
        assert!(DateVersion::parse("2025.13.1").is_err()); // Invalid month
        assert!(DateVersion::parse("2025.0.1").is_err()); // Invalid month (0)
        assert!(DateVersion::parse("invalid.11.1").is_err()); // Not a number
    }

    #[test]
    fn test_version_comparison() {
        let v1 = DateVersion::parse("2025.11.1").unwrap();
        let v2 = DateVersion::parse("2025.11.2").unwrap();
        let v3 = DateVersion::parse("2025.11.10").unwrap();
        let v4 = DateVersion::parse("2025.12.1").unwrap();
        let v5 = DateVersion::parse("2026.1.1").unwrap();

        // Same month, different release numbers
        assert!(v1 < v2);
        assert!(v2 > v1);

        // Different release numbers, same month
        assert!(v2 < v3);
        assert!(v3 > v2);

        // Different months, same year
        assert!(v3 < v4);
        assert!(v4 > v3);

        // Different years
        assert!(v4 < v5);
        assert!(v5 > v4);

        // Transitivity
        assert!(v1 < v3);
        assert!(v1 < v5);
    }

    #[test]
    fn test_version_equality() {
        let v1 = DateVersion::parse("2025.11.1").unwrap();
        let v2 = DateVersion::parse("2025.11.1").unwrap();

        assert_eq!(v1, v2);
        assert!(!(v1 < v2));
        assert!(!(v1 > v2));
    }

    #[test]
    fn test_version_to_string() {
        let v = DateVersion::parse("2025.11.1").unwrap();
        assert_eq!(v.to_string(), "2025.11.1");

        let v = DateVersion::parse("2025.1.10").unwrap();
        assert_eq!(v.to_string(), "2025.1.10");

        // Test with git hash
        let v = DateVersion::parse("2025.11.1+abc123d").unwrap();
        assert_eq!(v.to_string(), "2025.11.1+abc123d");
    }

    #[test]
    fn test_version_comparison_ignores_git_hash() {
        // Versions with different git hashes but same semver should be equal
        let v1 = DateVersion::parse("2025.11.1+abc123").unwrap();
        let v2 = DateVersion::parse("2025.11.1+def456").unwrap();
        let v3 = DateVersion::parse("2025.11.1").unwrap();

        assert_eq!(v1, v2); // Same version, different git hash
        assert_eq!(v1, v3); // Same version, one has git hash
        assert!(!(v1 < v2)); // Not less than
        assert!(!(v1 > v2)); // Not greater than
    }
}
