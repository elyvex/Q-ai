//! Security primitives for Q-ai phase 0.
//!
//! Implements the D0.15 — Security Baseline as reusable guard libraries.
//! Functions and types are pure Rust (no async, no I/O, no provider deps).
//! Fail-closed: deny on any error.

use std::net::IpAddr;
use std::path::{Path, PathBuf};

/// Security error codes (QAI-SEC-0xxx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    /// Path traversal attempt (QAI-SEC-0001)
    PathTraversal,
    /// Absolute or symlink escape attempt (QAI-SEC-0002)
    SymlinkEscape,
    /// Archive expansion ratio exceeded (QAI-SEC-0003)
    ExpansionRatio,
    /// Archive entry count exceeded (QAI-SEC-0004)
    EntryCount,
    /// Download size exceeds configured limit (QAI-SEC-0005)
    DownloadTooLarge,
    /// Redirect target resolves to a blocked/private address (QAI-SEC-0006)
    PrivateAddressBlocked,
    /// Input field failed validation (QAI-SEC-0007)
    InvalidInput,
    /// Input field exceeds maximum length (QAI-SEC-0008)
    FieldTooLong,
    /// Input JSON exceeds maximum nesting depth (QAI-SEC-0009)
    JsonTooDeep,
    /// HTML/content contains disallowed markup (QAI-SEC-0010)
    DisallowedContent,
    /// Nested archive depth exceeds the configured cap (QAI-SEC-0011)
    ArchiveTooDeep,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SecurityError {}

/// Canonicalize and contain: reject traversal, symlink escapes, absolute paths.
pub fn canonicalize_and_contain(root: &Path, rel: &Path) -> Result<PathBuf, SecurityError> {
    // Reject empty or special paths (just ".") that don't make sense.
    if rel.as_os_str().is_empty() {
        return Err(SecurityError::PathTraversal);
    }

    // Ensure rel is not absolute.
    if rel.is_absolute() {
        return Err(SecurityError::PathTraversal);
    }

    // Join without simplification.
    let full = root.join(rel);

    // Resolve symlinks for canonical path (fail-closed on any error).
    let canonical_full = std::fs::canonicalize(&full).map_err(|_| SecurityError::SymlinkEscape)?;
    let canonical_root = std::fs::canonicalize(root).map_err(|_| SecurityError::SymlinkEscape)?;

    // Ensure canonical_full starts with canonical_root.
    if !canonical_full.starts_with(&canonical_root) {
        return Err(SecurityError::PathTraversal);
    }

    // Ensure no ".." components after canonicalization (extra defense).
    if canonical_full.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(SecurityError::PathTraversal);
    }

    Ok(canonical_full)
}

/// IP address private range detection (no DNS).
pub fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // RFC 1918: 10.0.0.0/8
            if octets[0] == 10 {
                return true;
            }
            // "This network" / unspecified 0.0.0.0/8
            if octets[0] == 0 {
                return true;
            }
            // RFC 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 {
                return true;
            }
            // RFC 172.16.0.0/12
            if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
                return true;
            }
            // RFC 127.x.x.x loopback
            if octets[0] == 127 {
                return true;
            }
            // RFC 169.254.x.x link-local
            if octets[0] == 169 && octets[1] == 254 {
                return true;
            }
            // Multicast
            if octets[0] & 0xF0 == 0xE0 {
                return true;
            }
            // IPv4 mcast ff00::/8
            false
        }
        IpAddr::V6(ipv6) => {
            // IPv6 ULA fc00::/7 (unique local addressing)
            if let Some(segment) = ipv6.segments().first()
                && segment & 0xfe00 == 0xfc00
            {
                return true;
            }
            // IPv6 loopback ::1
            if ipv6.is_loopback() || ipv6.is_unspecified() {
                return true;
            }
            // IPv6 link-local fe80::/10
            if let Some(segment) = ipv6.segments().first()
                && segment & 0xffc0 == 0xfe80
            {
                return true;
            }
            // Unique Local Addresses fc00::/7 (excludes global unicast)
            false
        }
    }
}

/// Security limits for archive and download operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub max_download_bytes: u64,
    pub max_archive_entries: usize,
    pub max_archive_expansion_ratio: f64,
    /// Maximum nesting depth for archives-within-archives.
    pub max_archive_depth: u32,
}

impl Limits {
    /// Check if a download length is within limits.
    pub fn check_download(&self, len: u64) -> Result<(), SecurityError> {
        if len > self.max_download_bytes {
            return Err(SecurityError::DownloadTooLarge);
        }
        Ok(())
    }

    /// Check if an archive expansion ratio is within limits.
    pub fn check_expansion(&self, compressed: u64, expanded: u64) -> Result<(), SecurityError> {
        if compressed == 0 {
            return Ok(());
        }
        let ratio = expanded as f64 / compressed as f64;
        if ratio > self.max_archive_expansion_ratio {
            return Err(SecurityError::ExpansionRatio);
        }
        Ok(())
    }

    /// Check if an entry count is within limits.
    pub fn check_entry_count(&self, entries: usize) -> Result<(), SecurityError> {
        if entries > self.max_archive_entries {
            return Err(SecurityError::EntryCount);
        }
        Ok(())
    }
}

/// Untrusted wrapper for untrusted data (sanitize only via module-specific implementation).
#[derive(Debug)]
pub struct Untrusted<T>(T);

impl<T> Untrusted<T> {
    /// Create from raw data (must be wrapped by sanitizer).
    pub fn from_raw(data: T) -> Self {
        Untrusted(data)
    }

    /// Extract inner value (unsafe; use only by sanitizer modules).
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Policy decisions for security actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
    RequireApproval { reason: String },
}

/// Deny-by-default baseline decision engine.
pub fn default_denying(action: &str) -> PolicyDecision {
    match action {
        "download" | "read" | "extract" => {
            PolicyDecision::RequireApproval { reason: "new file".to_string() }
        }
        "write" | "create" | "overwrite" => PolicyDecision::Deny {
            reason: "write operations require explicit approval".to_string(),
        },
        "list" => PolicyDecision::Deny { reason: "directory listing blocked".to_string() },
        _ => PolicyDecision::Deny { reason: "unknown action".to_string() },
    }
}

/// Check that a redirect host IP is not in private ranges.
pub fn check_url_redirect(redirect_host_resolved: IpAddr) -> Result<(), SecurityError> {
    if is_private_ip(redirect_host_resolved) {
        return Err(SecurityError::PrivateAddressBlocked);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::path::Path;
    use std::str::FromStr;

    fn limits() -> Limits {
        Limits {
            max_download_bytes: 10,
            max_archive_entries: 3,
            max_archive_expansion_ratio: 2.0,
            max_archive_depth: 2,
        }
    }

    fn ip(s: &str) -> IpAddr {
        IpAddr::from_str(s).unwrap()
    }

    #[test]
    fn limits_reject_oversized_downloads() {
        assert!(limits().check_download(10).is_ok());
        assert!(matches!(limits().check_download(11), Err(SecurityError::DownloadTooLarge)));
    }

    #[test]
    fn limits_reject_excess_entries_and_expansion() {
        assert!(limits().check_entry_count(3).is_ok());
        assert!(matches!(limits().check_entry_count(4), Err(SecurityError::EntryCount)));
        assert!(limits().check_expansion(0, 100).is_ok());
        assert!(limits().check_expansion(10, 20).is_ok());
        assert!(matches!(limits().check_expansion(10, 21), Err(SecurityError::ExpansionRatio)));
    }

    #[test]
    fn private_ip_matrix_is_fully_classified() {
        for blocked in [
            "127.0.0.1",
            "10.0.0.1",
            "192.168.1.1",
            "172.16.0.1",
            "172.31.255.255",
            "169.254.1.1",
            "0.0.0.0",
            "224.0.0.1",
            "::1",
            "fe80::1",
            "fc00::1",
            "::",
        ] {
            assert!(is_private_ip(ip(blocked)), "{blocked} must be private");
        }
        for public in ["8.8.8.8", "1.1.1.1", "2001:4860:4860::8888"] {
            assert!(!is_private_ip(ip(public)), "{public} must be public");
        }
    }

    #[test]
    fn untrusted_round_trips() {
        let wrapped = Untrusted::from_raw(5);
        assert_eq!(wrapped.into_inner(), 5);
    }

    #[test]
    fn policy_defaults_deny_by_default() {
        assert!(matches!(default_denying("write"), PolicyDecision::Deny { .. }));
        assert!(matches!(default_denying("download"), PolicyDecision::RequireApproval { .. }));
        assert!(matches!(default_denying("list"), PolicyDecision::Deny { .. }));
        assert!(matches!(default_denying("something-unknown"), PolicyDecision::Deny { .. }));
    }

    #[test]
    fn redirect_guard_blocks_private_targets() {
        assert!(matches!(
            check_url_redirect(ip("10.0.0.1")),
            Err(SecurityError::PrivateAddressBlocked)
        ));
        assert!(check_url_redirect(ip("8.8.8.8")).is_ok());
    }

    #[test]
    fn canonicalize_rejects_empty_and_absolute_paths() {
        let root = std::env::temp_dir();
        assert!(canonicalize_and_contain(&root, Path::new("")).is_err());
        assert!(canonicalize_and_contain(&root, Path::new("/etc/passwd")).is_err());
    }

    #[test]
    fn canonicalize_accepts_an_in_root_file() {
        let root = std::env::temp_dir();
        let name = format!("qai-sec-test-{}.txt", std::process::id());
        std::fs::write(root.join(&name), b"x").unwrap();
        let resolved = canonicalize_and_contain(&root, Path::new(&name));
        assert!(resolved.is_ok());
        let _ = std::fs::remove_file(root.join(&name));
    }

    #[test]
    fn security_error_is_a_std_error_and_displays() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<SecurityError>();
        assert_eq!(SecurityError::PathTraversal.to_string(), "PathTraversal");
    }
}
