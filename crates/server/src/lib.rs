//! HTTP server: health endpoints + Quran read API v1 (D1.7).
//!
//! axum over `tokio::net::TcpListener`. The Quran routes are defined in
//! [`api`] against a backend trait, so contract tests run with fakes.

pub mod api;

use std::net::SocketAddr;
use thiserror::Error;

/// Errors originating from the server.
#[derive(Debug, Error)]
pub enum ServerError {
    /// The TCP listener could not bind to the requested address.
    #[error("failed to bind listener on {addr}: {source}")]
    Bind {
        /// Address.
        addr: String,
        /// I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The address string could not be resolved to a socket address.
    #[error("invalid listen address `{0}`")]
    InvalidAddr(String),
    #[error("server bind must be loopback; use 127.0.0.1 or [::1] instead of `{0}`")]
    NonLoopback(String),
    /// I/O failure while serving.
    #[error("serve error: {0}")]
    Serve(#[from] std::io::Error),
}

/// Whether the given `host:port` address binds to a loopback interface.
pub fn is_loopback(addr: &str) -> bool {
    loopback_addr(addr).is_ok()
}

pub fn loopback_addr(addr: &str) -> Result<SocketAddr, ServerError> {
    let parsed = if let Some(port) = addr.strip_prefix("localhost:") {
        let port = port.parse::<u16>().map_err(|_| ServerError::InvalidAddr(addr.to_owned()))?;
        SocketAddr::from(([127, 0, 0, 1], port))
    } else {
        addr.parse::<SocketAddr>().map_err(|_| ServerError::InvalidAddr(addr.to_owned()))?
    };
    if !parsed.ip().is_loopback() {
        return Err(ServerError::NonLoopback(addr.to_owned()));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_detection() {
        assert!(is_loopback("127.0.0.1:8737"));
        assert!(is_loopback("[::1]:8737"));
        assert!(is_loopback("localhost:8737"));
        assert!(!is_loopback("0.0.0.0:8737"));
        assert!(!is_loopback("192.168.1.10:8737"));
        assert!(is_loopback("127.0.0.2:0"));
        for invalid in [
            "localhost",
            "localhost:",
            "localhost:abc",
            "localhost:65536",
            "127.0.0.1.example:8737",
            "[::1]evil:8737",
        ] {
            assert!(!is_loopback(invalid), "{invalid}");
        }
        assert_eq!(
            loopback_addr("localhost:8737").unwrap(),
            "127.0.0.1:8737".parse::<SocketAddr>().unwrap()
        );
    }
}
