//! Phase 0 — minimal HTTP health server.
//!
//! Implements a hand-rolled HTTP/1.1 server over `tokio::net::TcpListener`:
//! `GET /healthz` and `GET /readyz` return 200, everything else 404.
//! Authentication enforcement is deferred to Phase 11 (see `config` docs).

use std::net::SocketAddr;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Errors originating from the health server.
#[derive(Debug, Error)]
pub enum ServerError {
    /// The TCP listener could not bind to the requested address.
    #[error("failed to bind listener on {addr}: {source}")]
    Bind {
        addr: String,
        #[source]
        source: std::io::Error,
    },
    /// The address string could not be resolved to a socket address.
    #[error("invalid listen address `{0}`")]
    InvalidAddr(String),
    /// I/O failure while serving a connection.
    #[error("connection I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Start the health server on `addr` and serve until the process is killed.
///
/// The default `addr` is `127.0.0.1:<port>`; passing another interface is
/// allowed (auth enforcement is Phase 11), but logs a warning.
pub async fn start(addr: &str) -> Result<(), ServerError> {
    if !is_loopback(addr) {
        eprintln!(
            "warning: server binding to non-loopback address `{addr}`; \
             authentication enforcement is deferred until Phase 11"
        );
    }

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|source| ServerError::Bind {
            addr: addr.to_string(),
            source,
        })?;

    loop {
        let (stream, _peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream).await {
                eprintln!("connection error: {err}");
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), ServerError> {
    let mut buf = [0u8; 1024];
    let read = stream.read(&mut buf).await?;
    if read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buf[..read]);
    let response = handle_request(&request);

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

/// Parse the request line of an HTTP/1.1 request and produce a response.
///
/// Separate from the I/O loop so it can be unit-tested directly.
pub fn handle_request(request: &str) -> String {
    let Some(request_line) = request.lines().next() else {
        return status_line(400, "bad request");
    };
    let mut parts = request_line.split_whitespace();
    let (method, target) = match (parts.next(), parts.next()) {
        (Some(method), Some(target)) => (method, target),
        _ => return status_line(400, "bad request"),
    };

    if method != "GET" {
        return status_line(405, "method not allowed");
    }

    match target {
        "/healthz" => status_line(200, "ok"),
        "/readyz" => status_line(200, "ready"),
        _ => status_line(404, "not found"),
    }
}

fn status_line(status: u16, body: &str) -> String {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Unknown",
    };
    let body_len = body.len();
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/plain\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n{body}"
    )
}

/// Whether the given `host:port` address binds to a loopback interface.
fn is_loopback(addr: &str) -> bool {
    match addr.parse::<SocketAddr>() {
        Ok(addr) => addr.ip().is_loopback(),
        Err(_) => addr == "localhost" || addr.split(':').next() == Some("localhost"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check() {
        let resp = handle_request("GET /healthz HTTP/1.1");
        assert!(resp.starts_with("HTTP/1.1 200"));
        assert!(resp.ends_with("\r\n\r\nok"));
    }

    #[tokio::test]
    async fn ready_check() {
        let resp = handle_request("GET /readyz HTTP/1.1");
        assert!(resp.starts_with("HTTP/1.1 200"));
        assert!(resp.ends_with("\r\n\r\nready"));
    }

    #[tokio::test]
    async fn unknown_path_is_404() {
        let resp = handle_request("GET /nope HTTP/1.1");
        assert!(resp.starts_with("HTTP/1.1 404"));
        assert!(resp.ends_with("\r\n\r\nnot found"));
    }

    #[tokio::test]
    async fn non_get_method_is_405() {
        let resp = handle_request("POST /healthz HTTP/1.1");
        assert!(resp.starts_with("HTTP/1.1 405"));
    }

    #[test]
    fn loopback_detection() {
        assert!(is_loopback("127.0.0.1:8737"));
        assert!(is_loopback("[::1]:8737"));
        assert!(is_loopback("localhost:8737"));
        assert!(!is_loopback("0.0.0.0:8737"));
        assert!(!is_loopback("192.168.1.10:8737"));
    }
}