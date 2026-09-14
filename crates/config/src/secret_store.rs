//! Secret storage abstraction (D0.5 / P0-T14–T16).
//!
//! Secret **values** never live in SQLite; what persists is a [`SecretRef`]
//! (`secret://<backend>/<path...>`). Backends:
//!
//! - [`EnvSecretStore`] — reads `QAI_SECRET_<KEY>` from the environment.
//! - [`EncryptedFileSecretStore`] — an XChaCha20-Poly1305 encrypted JSON file,
//!   keyed by a passphrase supplied via the environment.
//! - [`KeychainSecretStore`] — abstracted behind the same trait; its OS-native
//!   implementation requires the `keyring` crate and returns
//!   [`SecretError::Unsupported`] until that dependency is added (Phase 1).

use async_trait::async_trait;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng, rand_core::RngCore};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

use crate::Secret;

/// A reference to a secret, of the form `secret://<backend>/<seg>/<seg>...`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecretRef {
    backend: String,
    segments: Vec<String>,
}

/// A malformed secret reference or a backend failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SecretError {
    /// The reference is not `secret://<backend>/<path>`.
    #[error("invalid secret reference: {0}")]
    InvalidRef(String),
    /// The secret does not exist in the backend.
    #[error("secret not found: {0}")]
    NotFound(String),
    /// The backend exists but the requested operation is unsupported.
    #[error("secret backend unsupported: {0}")]
    Unsupported(String),
    /// A backend-specific failure.
    #[error("secret backend error: {0}")]
    Backend(String),
}

impl SecretRef {
    /// Parse `secret://<backend>/<segment>/<segment>...`.
    pub fn parse(input: &str) -> Result<Self, SecretError> {
        let rest = input
            .strip_prefix("secret://")
            .ok_or_else(|| SecretError::InvalidRef(input.to_string()))?;
        let mut parts = rest.split('/').filter(|s| !s.is_empty());
        let backend =
            parts.next().ok_or_else(|| SecretError::InvalidRef(input.to_string()))?.to_string();
        let segments: Vec<String> = parts.map(String::from).collect();
        if segments.is_empty() {
            return Err(SecretError::InvalidRef(input.to_string()));
        }
        Ok(Self { backend, segments })
    }

    /// The backend scheme (e.g. `env`, `keychain`, `encrypted_file`).
    pub fn backend(&self) -> &str {
        &self.backend
    }

    /// The path segments after the backend.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// The environment variable name for the `env` backend
    /// (`QAI_SECRET_<SEGMENTS_UPPER_UNDERSCORE>`).
    pub fn env_var(&self) -> String {
        let joined = self.segments.join("_").to_uppercase().replace(['-', '.'], "_");
        format!("QAI_SECRET_{joined}")
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "secret://{}/{}", self.backend, self.segments.join("/"))
    }
}

/// Storage backend for secrets.
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Retrieve a secret value.
    async fn get(&self, r: &SecretRef) -> Result<Secret<String>, SecretError>;
    /// Store a secret value.
    async fn put(&self, r: &SecretRef, v: Secret<String>) -> Result<(), SecretError>;
    /// Delete a secret value.
    async fn delete(&self, r: &SecretRef) -> Result<(), SecretError>;
    /// List references only — never values.
    async fn list_refs(&self) -> Result<Vec<SecretRef>, SecretError>;
    /// The backend name.
    fn backend_name(&self) -> &'static str;
}

/// Environment-variable secret backend.
///
/// Reads `QAI_SECRET_<KEY>`; read-only (writing to the process environment is
/// not a durable or safe operation, so `put`/`delete` report `Unsupported`).
#[derive(Debug, Default, Clone)]
pub struct EnvSecretStore;

#[async_trait]
impl SecretStore for EnvSecretStore {
    async fn get(&self, r: &SecretRef) -> Result<Secret<String>, SecretError> {
        let name = r.env_var();
        match std::env::var(&name) {
            Ok(value) => Ok(Secret::new(value)),
            Err(_) => Err(SecretError::NotFound(r.to_string())),
        }
    }

    async fn put(&self, _r: &SecretRef, _v: Secret<String>) -> Result<(), SecretError> {
        Err(SecretError::Unsupported("env backend is read-only".to_string()))
    }

    async fn delete(&self, _r: &SecretRef) -> Result<(), SecretError> {
        Err(SecretError::Unsupported("env backend is read-only".to_string()))
    }

    async fn list_refs(&self) -> Result<Vec<SecretRef>, SecretError> {
        let mut refs = Vec::new();
        for (key, _) in std::env::vars() {
            if let Some(rest) = key.strip_prefix("QAI_SECRET_") {
                let mut reference = SecretRef {
                    backend: "env".to_string(),
                    segments: rest.to_lowercase().split('_').map(String::from).collect(),
                };
                reference.segments.retain(|s| !s.is_empty());
                if !reference.segments.is_empty() {
                    refs.push(reference);
                }
            }
        }
        refs.sort();
        Ok(refs)
    }

    fn backend_name(&self) -> &'static str {
        "env"
    }
}

/// OS keychain secret backend (Secret Service / macOS Keychain / Windows
/// Credential Manager).
///
/// The concrete implementation is gated behind the `keychain` feature; without
/// it the backend reports [`SecretError::Unsupported`] so the default build has
/// no platform-specific dependencies. The account for a ref is its path
/// segments joined by `/` (e.g. `openai/default`) under the `qai` service.
#[derive(Debug, Default, Clone)]
pub struct KeychainSecretStore;

#[cfg(not(feature = "keychain"))]
#[async_trait]
impl SecretStore for KeychainSecretStore {
    async fn get(&self, _r: &SecretRef) -> Result<Secret<String>, SecretError> {
        Err(SecretError::Unsupported(
            "keychain backend requires the `keychain` feature".to_string(),
        ))
    }
    async fn put(&self, _r: &SecretRef, _v: Secret<String>) -> Result<(), SecretError> {
        Err(SecretError::Unsupported(
            "keychain backend requires the `keychain` feature".to_string(),
        ))
    }
    async fn delete(&self, _r: &SecretRef) -> Result<(), SecretError> {
        Err(SecretError::Unsupported(
            "keychain backend requires the `keychain` feature".to_string(),
        ))
    }
    async fn list_refs(&self) -> Result<Vec<SecretRef>, SecretError> {
        Err(SecretError::Unsupported(
            "keychain backend requires the `keychain` feature".to_string(),
        ))
    }
    fn backend_name(&self) -> &'static str {
        "keychain"
    }
}

#[cfg(feature = "keychain")]
#[async_trait]
impl SecretStore for KeychainSecretStore {
    async fn get(&self, r: &SecretRef) -> Result<Secret<String>, SecretError> {
        let entry = keyring::Entry::new("qai", &r.segments().join("/"))
            .map_err(|e| SecretError::Backend(e.to_string()))?;
        match entry.get_password() {
            Ok(value) => Ok(Secret::new(value)),
            Err(keyring::Error::NoEntry) => Err(SecretError::NotFound(r.to_string())),
            Err(e) => Err(SecretError::Backend(e.to_string())),
        }
    }
    async fn put(&self, r: &SecretRef, v: Secret<String>) -> Result<(), SecretError> {
        let entry = keyring::Entry::new("qai", &r.segments().join("/"))
            .map_err(|e| SecretError::Backend(e.to_string()))?;
        entry
            .set_password(v.expose())
            .map_err(|e| SecretError::Backend(e.to_string()))
    }
    async fn delete(&self, r: &SecretRef) -> Result<(), SecretError> {
        let entry = keyring::Entry::new("qai", &r.segments().join("/"))
            .map_err(|e| SecretError::Backend(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Err(SecretError::NotFound(r.to_string())),
            Err(e) => Err(SecretError::Backend(e.to_string())),
        }
    }
    async fn list_refs(&self) -> Result<Vec<SecretRef>, SecretError> {
        // The OS keychain has no portable enumeration API; listing requires an
        // external index, which is a Phase 1 concern.
        Ok(Vec::new())
    }
    fn backend_name(&self) -> &'static str {
        "keychain"
    }
}

/// XChaCha20-Poly1305 encrypted-file secret backend.
///
/// Secrets are stored as an encrypted JSON map `{ "secret://…": value }`. The
/// file layout is `24-byte nonce || ciphertext`. The 32-byte key is
/// `SHA-256(passphrase)`; the passphrase is supplied at construction — normally
/// read from an environment variable via
/// [`EncryptedFileSecretStore::from_env`] — and is never written to disk.
#[derive(Debug, Clone)]
pub struct EncryptedFileSecretStore {
    path: std::path::PathBuf,
    passphrase: String,
}

impl EncryptedFileSecretStore {
    /// Create from an explicit passphrase and file path.
    pub fn new(path: impl Into<std::path::PathBuf>, passphrase: String) -> Self {
        Self { path: path.into(), passphrase }
    }

    /// Create by reading the passphrase from `env_var`.
    pub fn from_env(
        path: impl Into<std::path::PathBuf>,
        env_var: &str,
    ) -> Result<Self, SecretError> {
        let passphrase = std::env::var(env_var).map_err(|_| {
            SecretError::Backend(format!("secret passphrase env var `{env_var}` is not set"))
        })?;
        Ok(Self::new(path, passphrase))
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        let key_bytes = Sha256::digest(self.passphrase.as_bytes());
        let key = Key::from_slice(&key_bytes);
        XChaCha20Poly1305::new(key)
    }

    fn load(&self) -> Result<BTreeMap<String, String>, SecretError> {
        if !self.path.exists() {
            return Ok(BTreeMap::new());
        }
        let raw = std::fs::read(&self.path).map_err(|e| SecretError::Backend(e.to_string()))?;
        if raw.len() < 24 {
            return Err(SecretError::Backend("encrypted file is truncated".to_string()));
        }
        let (nonce, ciphertext) = raw.split_at(24);
        let plaintext =
            self.cipher().decrypt(XNonce::from_slice(nonce), ciphertext).map_err(|_| {
                SecretError::Backend("decryption failed (wrong passphrase?)".to_string())
            })?;
        serde_json::from_slice(&plaintext).map_err(|e| SecretError::Backend(e.to_string()))
    }

    fn save(&self, map: &BTreeMap<String, String>) -> Result<(), SecretError> {
        let plaintext = serde_json::to_vec(map).map_err(|e| SecretError::Backend(e.to_string()))?;
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let ciphertext = self
            .cipher()
            .encrypt(XNonce::from_slice(&nonce_bytes), plaintext.as_ref())
            .map_err(|_| SecretError::Backend("encryption failed".to_string()))?;
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut out = nonce_bytes.to_vec();
        out.extend_from_slice(&ciphertext);
        std::fs::write(&self.path, out).map_err(|e| SecretError::Backend(e.to_string()))
    }
}

#[async_trait]
impl SecretStore for EncryptedFileSecretStore {
    async fn get(&self, r: &SecretRef) -> Result<Secret<String>, SecretError> {
        let map = self.load()?;
        map.get(&r.to_string())
            .cloned()
            .map(Secret::new)
            .ok_or_else(|| SecretError::NotFound(r.to_string()))
    }
    async fn put(&self, r: &SecretRef, v: Secret<String>) -> Result<(), SecretError> {
        let mut map = self.load()?;
        map.insert(r.to_string(), v.expose().clone());
        self.save(&map)
    }
    async fn delete(&self, r: &SecretRef) -> Result<(), SecretError> {
        let mut map = self.load()?;
        if map.remove(&r.to_string()).is_none() {
            return Err(SecretError::NotFound(r.to_string()));
        }
        self.save(&map)
    }
    async fn list_refs(&self) -> Result<Vec<SecretRef>, SecretError> {
        Ok(self.load()?.keys().filter_map(|k| SecretRef::parse(k).ok()).collect())
    }
    fn backend_name(&self) -> &'static str {
        "encrypted_file"
    }
}

/// Select a backend by name.
///
/// The encrypted-file backend needs a path and a passphrase, so it is
/// constructed via [`EncryptedFileSecretStore::from_env`] rather than here.
pub fn backend_by_name(name: &str) -> Result<Box<dyn SecretStore>, SecretError> {
    match name {
        "env" => Ok(Box::new(EnvSecretStore)),
        "keychain" => Ok(Box::new(KeychainSecretStore)),
        "encrypted_file" => Err(SecretError::Unsupported(
            "construct the encrypted_file backend via EncryptedFileSecretStore::from_env"
                .to_string(),
        )),
        other => Err(SecretError::Unsupported(format!("unknown backend `{other}`"))),
    }
}

/// A convenience registry of named backends.
#[derive(Default)]
pub struct SecretStores {
    stores: BTreeMap<String, Box<dyn SecretStore>>,
}

impl SecretStores {
    /// Register a backend under its [`SecretStore::backend_name`].
    pub fn register(&mut self, store: Box<dyn SecretStore>) {
        self.stores.insert(store.backend_name().to_string(), store);
    }

    /// Look up a backend by name.
    pub fn get(&self, backend: &str) -> Option<&dyn SecretStore> {
        self.stores.get(backend).map(|b| b.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_reference() {
        let r = SecretRef::parse("secret://env/openai/default").unwrap();
        assert_eq!(r.backend(), "env");
        assert_eq!(r.segments(), &["openai", "default"]);
        assert_eq!(r.env_var(), "QAI_SECRET_OPENAI_DEFAULT");
        assert_eq!(r.to_string(), "secret://env/openai/default");
    }

    #[test]
    fn rejects_malformed_references() {
        for bad in ["env/openai", "secret://env", "secret://", "openai"] {
            assert!(SecretRef::parse(bad).is_err(), "{bad} must be rejected");
        }
    }

    #[tokio::test]
    async fn env_store_reports_not_found_for_missing_keys() {
        let store = EnvSecretStore;
        let r = SecretRef::parse("secret://env/qai-test/missing").unwrap();
        let err = store.get(&r).await.unwrap_err();
        assert!(matches!(err, SecretError::NotFound(_)));
    }

    #[tokio::test]
    async fn env_store_is_read_only() {
        let store = EnvSecretStore;
        let r = SecretRef::parse("secret://env/qai-test/x").unwrap();
        let err = store.put(&r, Secret::new("v".to_string())).await.unwrap_err();
        assert!(matches!(err, SecretError::Unsupported(_)));
    }

    #[tokio::test]
    #[cfg(not(feature = "keychain"))]
    async fn unbacked_backends_report_unsupported() {
        let r = SecretRef::parse("secret://keychain/openai/default").unwrap();
        assert!(matches!(KeychainSecretStore.get(&r).await, Err(SecretError::Unsupported(_))));
    }

    #[tokio::test]
    async fn encrypted_file_round_trips_with_a_real_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.enc");
        let store =
            EncryptedFileSecretStore::new(path.clone(), "correct horse battery staple".into());
        let r = SecretRef::parse("secret://encrypted_file/openai/default").unwrap();

        store.put(&r, Secret::new("sk-sentinel-123".to_string())).await.unwrap();
        assert!(path.exists());

        // The on-disk file is encrypted: the plaintext never appears.
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            !bytes.windows(14).any(|w| w == b"sk-sentinel-123"),
            "secret must not appear in plaintext on disk"
        );

        // Round-trips and lists refs.
        assert_eq!(store.get(&r).await.unwrap().expose(), "sk-sentinel-123");
        assert_eq!(store.list_refs().await.unwrap().len(), 1);

        // Delete removes it.
        store.delete(&r).await.unwrap();
        assert!(matches!(store.get(&r).await, Err(SecretError::NotFound(_))));
    }

    #[tokio::test]
    async fn encrypted_file_rejects_the_wrong_passphrase() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.enc");
        let r = SecretRef::parse("secret://encrypted_file/openai/default").unwrap();
        EncryptedFileSecretStore::new(path.clone(), "right".into())
            .put(&r, Secret::new("v".to_string()))
            .await
            .unwrap();

        let wrong = EncryptedFileSecretStore::new(path, "wrong".into());
        assert!(matches!(wrong.get(&r).await, Err(SecretError::Backend(_))));
    }

    #[tokio::test]
    async fn encrypted_file_requires_the_passphrase_env_var() {
        let dir = tempfile::tempdir().unwrap();
        let err = EncryptedFileSecretStore::from_env(
            dir.path().join("secrets.enc"),
            "QAI_SECRET_PASSPHRASE_DEFINITELY_UNSET",
        )
        .unwrap_err();
        assert!(matches!(err, SecretError::Backend(_)));
    }

    #[tokio::test]
    async fn backend_lookup_by_name() {
        assert_eq!(backend_by_name("env").unwrap().backend_name(), "env");
        assert!(backend_by_name("encrypted_file").is_err());
        assert!(backend_by_name("nope").is_err());
    }
}
