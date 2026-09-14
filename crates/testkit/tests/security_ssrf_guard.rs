//! `tests/security/ssrf_guard.rs` — private/loopback/link-local/ULA blocking.
//!
//! AC-P0-15: resolve-then-check rejects every blocked range and fails closed.

use domain::security_net::{check_domain_allowlist, check_redirect_target, check_resolved_ip};
use std::net::IpAddr;
use std::str::FromStr;

fn ip(s: &str) -> IpAddr {
    IpAddr::from_str(s).expect("valid ip")
}

#[test]
fn loopback_private_linklocal_and_ula_are_blocked() {
    let blocked = [
        // loopback
        "127.0.0.1",
        "127.1.2.3",
        "::1",
        // RFC1918
        "10.0.0.1",
        "10.255.255.255",
        "172.16.0.1",
        "172.31.255.255",
        "192.168.0.1",
        "192.168.255.255",
        // link-local
        "169.254.0.1",
        "fe80::1",
        // IPv6 unique-local (ULA)
        "fc00::1",
        "fd12:3456::1",
        // multicast
        "224.0.0.1",
        // unspecified
        "0.0.0.0",
        "::",
    ];
    for s in blocked {
        assert!(check_resolved_ip(ip(s)).is_err(), "expected {s} to be blocked");
    }
}

#[test]
fn public_addresses_are_allowed() {
    for s in ["8.8.8.8", "1.1.1.1", "93.184.216.34", "2606:4700:4700::1111"] {
        assert!(check_resolved_ip(ip(s)).is_ok(), "expected {s} to be allowed");
    }
}

#[test]
fn redirect_to_private_is_blocked() {
    assert!(check_redirect_target(ip("10.0.0.5")).is_err());
    assert!(check_redirect_target(ip("1.1.1.1")).is_ok());
}

#[test]
fn empty_allowlist_denies_everything() {
    // Fail-closed: no allowlist means no host is permitted.
    let empty: Vec<String> = vec![];
    assert!(!check_domain_allowlist("example.com", &empty));
}

#[test]
fn allowlist_exact_and_subdomain_match() {
    let list = vec!["example.org".to_string(), "gnu.org".to_string()];
    assert!(check_domain_allowlist("example.org", &list));
    assert!(check_domain_allowlist("api.example.org", &list));
    assert!(!check_domain_allowlist("evil-example.org", &list));
    assert!(!check_domain_allowlist("example.org.evil.com", &list));
}
