use std::net::IpAddr;

use crate::security::{self, SecurityError};

/// SSRF guard: validate a resolved IP against private/blocked ranges (D0.15 / T44).
///
/// This is the "resolve-then-check" half of the SSRF guard. The caller performs
/// DNS resolution (network I/O lives outside `domain`), then hands the resolved
/// IPs here. Any IP in a blocked range fails closed.
pub fn check_resolved_ip(ip: IpAddr) -> Result<(), SecurityError> {
    if security::is_private_ip(ip) {
        return Err(SecurityError::PrivateAddressBlocked);
    }
    Ok(())
}

/// Validate a redirect target's resolved IP (D0.15 / T44).
///
/// Mirrors [`check_resolved_ip`] with a distinct name for call-site clarity.
pub fn check_redirect_target(ip: IpAddr) -> Result<(), SecurityError> {
    check_resolved_ip(ip)
}

/// Validate a domain allowlist membership (D0.15 / T44).
///
/// The allowlist is a set of exact host suffixes. An empty allowlist denies
/// everything (fail-closed). Matching is case-insensitive.
pub fn check_domain_allowlist(host: &str, allowlist: &[String]) -> bool {
    if allowlist.is_empty() {
        return false;
    }
    let host = host.trim().to_lowercase();
    allowlist.iter().any(|allowed| {
        host == allowed.to_lowercase() || host.ends_with(&format!(".{}", allowed.to_lowercase()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn private_ipv4_matrix() {
        let blocked = ["127.0.0.1", "10.0.0.1", "172.16.0.1", "192.168.0.1", "169.254.0.1"];
        for ip in blocked {
            let ip = IpAddr::from_str(ip).unwrap();
            assert!(check_resolved_ip(ip).is_err(), "expected {ip} to be blocked");
        }
    }

    #[test]
    fn private_ipv6_matrix() {
        let blocked = ["fe80::1", "fc00::1", "::1"];
        for ip in blocked {
            let ip = IpAddr::from_str(ip).unwrap();
            assert!(check_resolved_ip(ip).is_err(), "expected {ip} to be blocked");
        }
    }

    #[test]
    fn public_ip_is_allowed() {
        let ip = IpAddr::from_str("8.8.8.8").unwrap();
        assert!(check_resolved_ip(ip).is_ok());
    }

    #[test]
    fn domain_allowlist_exact_and_suffix() {
        let allowlist = vec!["example.com".to_string(), "trusted.org".to_string()];
        assert!(check_domain_allowlist("example.com", &allowlist));
        assert!(check_domain_allowlist("api.example.com", &allowlist));
        assert!(!check_domain_allowlist("evil.com", &allowlist));
    }

    #[test]
    fn empty_allowlist_denies_everything() {
        let allowlist: Vec<String> = vec![];
        assert!(!check_domain_allowlist("example.com", &allowlist));
    }
}
