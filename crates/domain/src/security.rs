//! Security primitives for Q-ai phase 0.
//! 
//! Implements the D0.15 — Security Baseline as reusable guard libraries.
//! Functions and types are pure Rust (no async, no I/O, no provider deps).
//! Fail-closed: deny on any error.

use std::path::{Path, PathBuf};
use std::net::IpAddr;

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
            if octets[0] == 10 { return true; }
            // RFC 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 { return true; }
            // RFC 172.16.0.0/12
            if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 { return true; }
            // RFC 127.x.x.x loopback
            if octets[0] == 127 { return true; }
            // RFC 169.254.x.x link-local
            if octets[0] == 169 && octets[1] == 254 { return true; }
            // Multicast
            if octets[0] & 0xF0 == 0xE0 { return true; }
            // IPv4 mcast ff00::/8
            false
        }
        IpAddr::V6(ipv6) => {
            // IPv6 ULA fc00::/7 (unique local addressing)
            if let Some(segment) = ipv6.segments().get(0) {
                if segment & 0xfe00 == 0xfc00 { return true; }
            }
            // IPv6 loopback ::1
            if ipv6.is_loopback() || ipv6.is_unspecified() { return true; }
            // IPv6 link-local fe80::/10
            if let Some(segment) = ipv6.segments().get(0) {
                if segment & 0xffc0 == 0xfe80 { return true; }
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
        if compressed == 0 { return Ok(()); }
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
        "download" | "read" | "extract" => PolicyDecision::RequireApproval { reason: "new file".to_string() },
        "write" | "create" | "overwrite" => PolicyDecision::Deny { reason: "write operations require explicit approval".to_string() },
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