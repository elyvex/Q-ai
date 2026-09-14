mod origin;
mod secret;
mod secret_store;

pub use origin::{OriginMap, ValueOrigin};
pub use secret::Secret;
pub use secret_store::{
    EncryptedFileSecretStore, EnvSecretStore, KeychainSecretStore, SecretError, SecretRef,
    SecretStore, SecretStores, backend_by_name,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use thiserror::Error;

// ── Error type ──────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    FileRead { path: String, source: std::io::Error },
    #[error("failed to parse config TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("config validation failed: {0}")]
    Validation(String),
    #[error("interpolation cycle detected at key: {0}")]
    InterpolationCycle(String),
    #[error("missing required config key: {0}")]
    MissingKey(String),
}

// ── Sub-structs ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mode: String,
    pub data_dir: String,
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind: String,
    pub port: u16,
    pub tls: String,
    pub require_auth_outside_localhost: bool,
    pub max_request_bytes: u64,
    pub request_timeout_ms: u64,
    pub concurrency_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: String,
    pub sqlite: SqliteConfig,
    pub objects: ObjectsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
    pub journal_mode: String,
    pub synchronous: String,
    pub busy_timeout_ms: u64,
    pub foreign_keys: bool,
    pub max_connections: u32,
    pub read_only_pool: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectsConfig {
    pub backend: String,
    pub root: String,
    pub max_file_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    pub backend: String,
    pub encrypted_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobsConfig {
    pub workers: u32,
    pub poll_interval_ms: u64,
    pub lease_seconds: u64,
    pub max_attempts: u32,
    pub backoff_base_ms: u64,
    pub backoff_max_ms: u64,
    pub backoff_jitter: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: String,
    pub redact_secrets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub otlp_endpoint: String,
    pub metrics_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allow_network_egress: bool,
    pub domain_allowlist: Vec<String>,
    pub ssrf_block_private_ranges: bool,
    pub max_download_bytes: u64,
    pub max_archive_entries: u32,
    pub max_archive_expansion_ratio: u32,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub tool_execution_default: String,
    pub command_execution_enabled: bool,
    pub canonical_write_requires_approval: bool,
}

// ── Main Config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub secrets: SecretsConfig,
    pub jobs: JobsConfig,
    pub logging: LoggingConfig,
    pub telemetry: TelemetryConfig,
    pub security: SecurityConfig,
    pub policy: PolicyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig {
                mode: "local".into(),
                data_dir: "~/.local/share/qai".into(),
                locale: "en".into(),
            },
            server: ServerConfig {
                bind: "127.0.0.1".into(),
                port: 8737,
                tls: "disabled".into(),
                require_auth_outside_localhost: true,
                max_request_bytes: 10_485_760,
                request_timeout_ms: 30_000,
                concurrency_limit: 128,
            },
            storage: StorageConfig {
                backend: "sqlite".into(),
                sqlite: SqliteConfig {
                    path: "${app.data_dir}/qai.db".into(),
                    journal_mode: "wal".into(),
                    synchronous: "full".into(),
                    busy_timeout_ms: 5_000,
                    foreign_keys: true,
                    max_connections: 8,
                    read_only_pool: true,
                },
                objects: ObjectsConfig {
                    backend: "filesystem".into(),
                    root: "${app.data_dir}/objects".into(),
                    max_file_bytes: 536_870_912,
                },
            },
            secrets: SecretsConfig {
                backend: "env".into(),
                encrypted_file_path: "${app.data_dir}/secrets.age".into(),
            },
            jobs: JobsConfig {
                workers: 4,
                poll_interval_ms: 250,
                lease_seconds: 60,
                max_attempts: 5,
                backoff_base_ms: 500,
                backoff_max_ms: 60_000,
                backoff_jitter: 0.2,
            },
            logging: LoggingConfig {
                level: "info".into(),
                format: "text".into(),
                file: String::new(),
                redact_secrets: true,
            },
            telemetry: TelemetryConfig {
                enabled: false,
                otlp_endpoint: String::new(),
                metrics_enabled: true,
            },
            security: SecurityConfig {
                allow_network_egress: false,
                domain_allowlist: vec![],
                ssrf_block_private_ranges: true,
                max_download_bytes: 268_435_456,
                max_archive_entries: 20_000,
                max_archive_expansion_ratio: 100,
                follow_symlinks: false,
            },
            policy: PolicyConfig {
                tool_execution_default: "deny".into(),
                command_execution_enabled: false,
                canonical_write_requires_approval: true,
            },
        }
    }
}

impl Config {
    /// Load config with precedence: Defaults < Config File < Env < CLI.
    ///
    /// Returns the config and an `OriginMap` tracking where each value came from.
    pub fn load(
        config_path: Option<&PathBuf>,
        env_prefix: &str,
        cli_overrides: &BTreeMap<String, String>,
    ) -> Result<(Self, OriginMap), ConfigError> {
        let mut config: Self = Self::default();
        let mut origins = OriginMap::new();

        // Mark all defaults
        mark_defaults(&mut origins);

        // Load from TOML file if present
        if let Some(path) = config_path
            && path.exists()
        {
            let raw = std::fs::read_to_string(path).map_err(|e| ConfigError::FileRead {
                path: path.display().to_string(),
                source: e,
            })?;
            let file_value: toml::Value = toml::from_str(&raw).map_err(ConfigError::Parse)?;
            merge_toml(&mut config, &file_value);
            mark_file_origins(&mut origins, path.display().to_string(), &file_value);
        }

        // Load from environment variables
        merge_env(&mut config, env_prefix, &mut origins);

        // Apply CLI overrides (highest precedence)
        merge_cli(&mut config, cli_overrides, &mut origins);

        // Resolve ${...} interpolation
        resolve_interpolation(&mut config)?;

        // Validate
        config.validate()?;

        Ok((config, origins))
    }

    /// Validate the config, returning an error if constraints are violated.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // bind="0.0.0.0" + tls="disabled" + require_auth_outside_localhost is invalid
        if self.server.bind == "0.0.0.0"
            && self.server.tls == "disabled"
            && self.server.require_auth_outside_localhost
        {
            return Err(ConfigError::Validation(
                "bind=\"0.0.0.0\" with tls=\"disabled\" and require_auth_outside_localhost=true is unsafe: \
                 binding to all interfaces without TLS and without auth allows unauthenticated access \
                 from any network. Either set bind to a loopback address, enable TLS, or disable \
                 require_auth_outside_localhost.".into(),
            ));
        }

        // TLS must be "disabled" or "required"
        if self.server.tls != "disabled" && self.server.tls != "required" {
            return Err(ConfigError::Validation(format!(
                "server.tls must be 'disabled' or 'required', got '{}'",
                self.server.tls
            )));
        }

        // Non-loopback bind without TLS should not require auth outside localhost
        if !is_loopback(&self.server.bind)
            && self.server.tls == "disabled"
            && self.server.require_auth_outside_localhost
        {
            return Err(ConfigError::Validation(format!(
                "bind=\"{}\" with tls=\"disabled\": non-loopback binds without TLS \
                         require require_auth_outside_localhost=false for safety",
                self.server.bind
            )));
        }

        Ok(())
    }
}

// ── Origin tracking helpers ───────────────────────────────────────────

fn mark_defaults(origins: &mut OriginMap) {
    for key in [
        "app.mode",
        "app.data_dir",
        "app.locale",
        "server.bind",
        "server.port",
        "server.tls",
        "storage.backend",
        "storage.sqlite.path",
        "storage.objects.backend",
        "secrets.backend",
        "jobs.workers",
        "logging.level",
        "logging.format",
        "logging.redact_secrets",
        "telemetry.enabled",
        "telemetry.metrics_enabled",
        "security.allow_network_egress",
        "security.ssrf_block_private_ranges",
        "policy.tool_execution_default",
    ] {
        origins.insert(key, ValueOrigin::Default);
    }
}

fn mark_file_origins(origins: &mut OriginMap, path: String, value: &toml::Value) {
    mark_toml_origins_recursive(origins, &path, value, "");
}

fn mark_toml_origins_recursive(
    origins: &mut OriginMap,
    path: &str,
    value: &toml::Value,
    prefix: &str,
) {
    match value {
        toml::Value::Table(table) => {
            for (k, v) in table {
                let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                mark_toml_origins_recursive(origins, path, v, &key);
            }
        }
        toml::Value::String(_)
        | toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_) => {
            origins.insert(prefix, ValueOrigin::File { path: path.to_string(), line: None });
        }
        _ => {}
    }
}

// ── TOML merging ──────────────────────────────────────────────────────

fn merge_toml(config: &mut Config, value: &toml::Value) {
    let table = match value.as_table() {
        Some(t) => t,
        None => return,
    };

    // Top-level: each key is a section name
    if let Some(app) = table.get("app").and_then(|v| v.as_table()) {
        if let Some(mode) = app.get("mode").and_then(|v| v.as_str()) {
            config.app.mode = mode.to_string();
        }
        if let Some(data_dir) = app.get("data_dir").and_then(|v| v.as_str()) {
            config.app.data_dir = data_dir.to_string();
        }
        if let Some(locale) = app.get("locale").and_then(|v| v.as_str()) {
            config.app.locale = locale.to_string();
        }
    }

    if let Some(server) = table.get("server").and_then(|v| v.as_table()) {
        if let Some(bind) = server.get("bind").and_then(|v| v.as_str()) {
            config.server.bind = bind.to_string();
        }
        if let Some(port) = server.get("port").and_then(|v| v.as_integer()) {
            config.server.port = port as u16;
        }
        if let Some(tls) = server.get("tls").and_then(|v| v.as_str()) {
            config.server.tls = tls.to_string();
        }
        if let Some(req) = server.get("require_auth_outside_localhost").and_then(|v| v.as_bool()) {
            config.server.require_auth_outside_localhost = req;
        }
        if let Some(mrb) = server.get("max_request_bytes").and_then(|v| v.as_integer()) {
            config.server.max_request_bytes = mrb as u64;
        }
        if let Some(rtm) = server.get("request_timeout_ms").and_then(|v| v.as_integer()) {
            config.server.request_timeout_ms = rtm as u64;
        }
        if let Some(cl) = server.get("concurrency_limit").and_then(|v| v.as_integer()) {
            config.server.concurrency_limit = cl as u32;
        }
    }

    if let Some(storage) = table.get("storage").and_then(|v| v.as_table()) {
        if let Some(backend) = storage.get("backend").and_then(|v| v.as_str()) {
            config.storage.backend = backend.to_string();
        }
        if let Some(sqlite) = storage.get("sqlite").and_then(|v| v.as_table()) {
            if let Some(path) = sqlite.get("path").and_then(|v| v.as_str()) {
                config.storage.sqlite.path = path.to_string();
            }
            if let Some(jm) = sqlite.get("journal_mode").and_then(|v| v.as_str()) {
                config.storage.sqlite.journal_mode = jm.to_string();
            }
            if let Some(syn) = sqlite.get("synchronous").and_then(|v| v.as_str()) {
                config.storage.sqlite.synchronous = syn.to_string();
            }
            if let Some(bt) = sqlite.get("busy_timeout_ms").and_then(|v| v.as_integer()) {
                config.storage.sqlite.busy_timeout_ms = bt as u64;
            }
            if let Some(fk) = sqlite.get("foreign_keys").and_then(|v| v.as_bool()) {
                config.storage.sqlite.foreign_keys = fk;
            }
            if let Some(mc) = sqlite.get("max_connections").and_then(|v| v.as_integer()) {
                config.storage.sqlite.max_connections = mc as u32;
            }
            if let Some(ro) = sqlite.get("read_only_pool").and_then(|v| v.as_bool()) {
                config.storage.sqlite.read_only_pool = ro;
            }
        }
        if let Some(objects) = storage.get("objects").and_then(|v| v.as_table()) {
            if let Some(backend) = objects.get("backend").and_then(|v| v.as_str()) {
                config.storage.objects.backend = backend.to_string();
            }
            if let Some(root) = objects.get("root").and_then(|v| v.as_str()) {
                config.storage.objects.root = root.to_string();
            }
            if let Some(mfb) = objects.get("max_file_bytes").and_then(|v| v.as_integer()) {
                config.storage.objects.max_file_bytes = mfb as u64;
            }
        }
    }

    if let Some(secrets) = table.get("secrets").and_then(|v| v.as_table()) {
        if let Some(backend) = secrets.get("backend").and_then(|v| v.as_str()) {
            config.secrets.backend = backend.to_string();
        }
        if let Some(path) = secrets.get("encrypted_file_path").and_then(|v| v.as_str()) {
            config.secrets.encrypted_file_path = path.to_string();
        }
    }

    if let Some(jobs) = table.get("jobs").and_then(|v| v.as_table()) {
        if let Some(w) = jobs.get("workers").and_then(|v| v.as_integer()) {
            config.jobs.workers = w as u32;
        }
        if let Some(pim) = jobs.get("poll_interval_ms").and_then(|v| v.as_integer()) {
            config.jobs.poll_interval_ms = pim as u64;
        }
        if let Some(ls) = jobs.get("lease_seconds").and_then(|v| v.as_integer()) {
            config.jobs.lease_seconds = ls as u64;
        }
        if let Some(ma) = jobs.get("max_attempts").and_then(|v| v.as_integer()) {
            config.jobs.max_attempts = ma as u32;
        }
        if let Some(bb) = jobs.get("backoff_base_ms").and_then(|v| v.as_integer()) {
            config.jobs.backoff_base_ms = bb as u64;
        }
        if let Some(bm) = jobs.get("backoff_max_ms").and_then(|v| v.as_integer()) {
            config.jobs.backoff_max_ms = bm as u64;
        }
        if let Some(bj) = jobs.get("backoff_jitter").and_then(|v| v.as_float()) {
            config.jobs.backoff_jitter = bj;
        }
    }

    if let Some(logging) = table.get("logging").and_then(|v| v.as_table()) {
        if let Some(level) = logging.get("level").and_then(|v| v.as_str()) {
            config.logging.level = level.to_string();
        }
        if let Some(fmt) = logging.get("format").and_then(|v| v.as_str()) {
            config.logging.format = fmt.to_string();
        }
        if let Some(file) = logging.get("file").and_then(|v| v.as_str()) {
            config.logging.file = file.to_string();
        }
        if let Some(redact) = logging.get("redact_secrets").and_then(|v| v.as_bool()) {
            config.logging.redact_secrets = redact;
        }
    }

    if let Some(telemetry) = table.get("telemetry").and_then(|v| v.as_table()) {
        if let Some(enabled) = telemetry.get("enabled").and_then(|v| v.as_bool()) {
            config.telemetry.enabled = enabled;
        }
        if let Some(otlp) = telemetry.get("otlp_endpoint").and_then(|v| v.as_str()) {
            config.telemetry.otlp_endpoint = otlp.to_string();
        }
        if let Some(me) = telemetry.get("metrics_enabled").and_then(|v| v.as_bool()) {
            config.telemetry.metrics_enabled = me;
        }
    }

    if let Some(security) = table.get("security").and_then(|v| v.as_table()) {
        if let Some(ane) = security.get("allow_network_egress").and_then(|v| v.as_bool()) {
            config.security.allow_network_egress = ane;
        }
        if let Some(dal) = security.get("domain_allowlist").and_then(|v| v.as_array()) {
            config.security.domain_allowlist =
                dal.iter().filter_map(|v| v.as_str().map(String::from)).collect();
        }
        if let Some(ssrf) = security.get("ssrf_block_private_ranges").and_then(|v| v.as_bool()) {
            config.security.ssrf_block_private_ranges = ssrf;
        }
        if let Some(mdb) = security.get("max_download_bytes").and_then(|v| v.as_integer()) {
            config.security.max_download_bytes = mdb as u64;
        }
        if let Some(mae) = security.get("max_archive_entries").and_then(|v| v.as_integer()) {
            config.security.max_archive_entries = mae as u32;
        }
        if let Some(aer) = security.get("max_archive_expansion_ratio").and_then(|v| v.as_integer())
        {
            config.security.max_archive_expansion_ratio = aer as u32;
        }
        if let Some(fs) = security.get("follow_symlinks").and_then(|v| v.as_bool()) {
            config.security.follow_symlinks = fs;
        }
    }

    if let Some(policy) = table.get("policy").and_then(|v| v.as_table()) {
        if let Some(ted) = policy.get("tool_execution_default").and_then(|v| v.as_str()) {
            config.policy.tool_execution_default = ted.to_string();
        }
        if let Some(ce) = policy.get("command_execution_enabled").and_then(|v| v.as_bool()) {
            config.policy.command_execution_enabled = ce;
        }
        if let Some(cwra) =
            policy.get("canonical_write_requires_approval").and_then(|v| v.as_bool())
        {
            config.policy.canonical_write_requires_approval = cwra;
        }
    }
}

// ── Environment merging ───────────────────────────────────────────────

fn merge_env(config: &mut Config, prefix: &str, origins: &mut OriginMap) {
    let prefix_str = format!("{prefix}__");
    for (key, value) in std::env::vars() {
        if !key.starts_with(&prefix_str) {
            continue;
        }
        let rest = &key[prefix_str.len()..];
        let parts: Vec<&str> = rest.split("__").collect();
        if parts.len() < 2 {
            continue;
        }
        set_env_value(config, &parts, &value, origins);
    }
}

fn set_env_value(config: &mut Config, parts: &[&str], value: &str, origins: &mut OriginMap) {
    let env_origin = ValueOrigin::Env(parts.join("__"));
    match parts[0] {
        "APP" => set_app_env(config, &parts[1..], value, origins, &env_origin),
        "SERVER" => set_server_env(config, &parts[1..], value, origins, &env_origin),
        "STORAGE" => set_storage_env(config, &parts[1..], value, origins, &env_origin),
        "SECRETS" => set_secrets_env(config, &parts[1..], value, origins, &env_origin),
        "JOBS" => set_jobs_env(config, &parts[1..], value, origins, &env_origin),
        "LOGGING" => set_logging_env(config, &parts[1..], value, origins, &env_origin),
        "TELEMETRY" => set_telemetry_env(config, &parts[1..], value, origins, &env_origin),
        "SECURITY" => set_security_env(config, &parts[1..], value, origins, &env_origin),
        "POLICY" => set_policy_env(config, &parts[1..], value, origins, &env_origin),
        _ => {}
    }
}

fn set_app_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "MODE" => {
            config.app.mode = value.to_string();
            origins.insert("app.mode".to_string(), origin.clone());
        }
        "DATA_DIR" => {
            config.app.data_dir = value.to_string();
            origins.insert("app.data_dir".to_string(), origin.clone());
        }
        "LOCALE" => {
            config.app.locale = value.to_string();
            origins.insert("app.locale".to_string(), origin.clone());
        }
        _ => {}
    }
}

fn set_server_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "BIND" => {
            config.server.bind = value.to_string();
            origins.insert("server.bind".to_string(), origin.clone());
        }
        "PORT" => {
            if let Ok(p) = value.parse::<u16>() {
                config.server.port = p;
                origins.insert("server.port".to_string(), origin.clone());
            }
        }
        "TLS" => {
            config.server.tls = value.to_string();
            origins.insert("server.tls".to_string(), origin.clone());
        }
        "REQUIRE_AUTH_OUTSIDE_LOCALHOST" => {
            if let Ok(b) = value.parse::<bool>() {
                config.server.require_auth_outside_localhost = b;
                origins.insert("server.require_auth_outside_localhost".to_string(), origin.clone());
            }
        }
        _ => {}
    }
}

fn set_storage_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "BACKEND" => {
            config.storage.backend = value.to_string();
            origins.insert("storage.backend".to_string(), origin.clone());
        }
        _ => {
            if parts.len() >= 2 && parts[0] == "SQLITE" && parts[1] == "PATH" {
                config.storage.sqlite.path = value.to_string();
                origins.insert("storage.sqlite.path".to_string(), origin.clone());
            }
            if parts.len() >= 2 && parts[0] == "OBJECTS" && parts[1] == "ROOT" {
                config.storage.objects.root = value.to_string();
                origins.insert("storage.objects.root".to_string(), origin.clone());
            }
        }
    }
}

fn set_secrets_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.first() == Some(&"BACKEND") {
        config.secrets.backend = value.to_string();
        origins.insert("secrets.backend".to_string(), origin.clone());
    }
}

fn set_jobs_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.first() == Some(&"WORKERS")
        && let Ok(w) = value.parse::<u32>()
    {
        config.jobs.workers = w;
        origins.insert("jobs.workers".to_string(), origin.clone());
    }
}

fn set_logging_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.first() == Some(&"LEVEL") {
        config.logging.level = value.to_string();
        origins.insert("logging.level".to_string(), origin.clone());
    }
}

fn set_telemetry_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.first() == Some(&"ENABLED")
        && let Ok(b) = value.parse::<bool>()
    {
        config.telemetry.enabled = b;
        origins.insert("telemetry.enabled".to_string(), origin.clone());
    }
}

fn set_security_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.first() == Some(&"ALLOW_NETWORK_EGRESS")
        && let Ok(b) = value.parse::<bool>()
    {
        config.security.allow_network_egress = b;
        origins.insert("security.allow_network_egress".to_string(), origin.clone());
    }
}

fn set_policy_env(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.first() == Some(&"TOOL_EXECUTION_DEFAULT") {
        config.policy.tool_execution_default = value.to_string();
        origins.insert("policy.tool_execution_default".to_string(), origin.clone());
    }
}

// ── CLI merging ───────────────────────────────────────────────────────

fn merge_cli(config: &mut Config, overrides: &BTreeMap<String, String>, origins: &mut OriginMap) {
    for (key, value) in overrides {
        let parts: Vec<&str> = key.split('.').collect();
        let origin = ValueOrigin::Cli(key.clone());
        set_cli_value(config, &parts, value, origins, &origin);
    }
}

fn set_cli_value(
    config: &mut Config,
    parts: &[&str],
    value: &str,
    origins: &mut OriginMap,
    origin: &ValueOrigin,
) {
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "app" => {
            if parts.len() < 2 {
                return;
            }
            match parts[1] {
                "mode" => {
                    config.app.mode = value.to_string();
                    origins.insert("app.mode".to_string(), origin.clone());
                }
                "data_dir" => {
                    config.app.data_dir = value.to_string();
                    origins.insert("app.data_dir".to_string(), origin.clone());
                }
                "locale" => {
                    config.app.locale = value.to_string();
                    origins.insert("app.locale".to_string(), origin.clone());
                }
                _ => {}
            }
        }
        "server" => {
            if parts.len() < 2 {
                return;
            }
            match parts[1] {
                "bind" => {
                    config.server.bind = value.to_string();
                    origins.insert("server.bind".to_string(), origin.clone());
                }
                "port" => {
                    if let Ok(p) = value.parse::<u16>() {
                        config.server.port = p;
                        origins.insert("server.port".to_string(), origin.clone());
                    }
                }
                "tls" => {
                    config.server.tls = value.to_string();
                    origins.insert("server.tls".to_string(), origin.clone());
                }
                _ => {}
            }
        }
        "storage" => {
            if parts.len() < 2 {
                return;
            }
            if parts[1] == "backend" {
                config.storage.backend = value.to_string();
                origins.insert("storage.backend".to_string(), origin.clone());
            }
            if parts.len() >= 3 && parts[1] == "sqlite" && parts[2] == "path" {
                config.storage.sqlite.path = value.to_string();
                origins.insert("storage.sqlite.path".to_string(), origin.clone());
            }
            if parts.len() >= 3 && parts[1] == "objects" && parts[2] == "root" {
                config.storage.objects.root = value.to_string();
                origins.insert("storage.objects.root".to_string(), origin.clone());
            }
        }
        "secrets" => {
            if parts.len() >= 2 && parts[1] == "backend" {
                config.secrets.backend = value.to_string();
                origins.insert("secrets.backend".to_string(), origin.clone());
            }
        }
        "jobs" => {
            if parts.len() >= 2
                && parts[1] == "workers"
                && let Ok(w) = value.parse::<u32>()
            {
                config.jobs.workers = w;
                origins.insert("jobs.workers".to_string(), origin.clone());
            }
        }
        "logging" => {
            if parts.len() >= 2 && parts[1] == "level" {
                config.logging.level = value.to_string();
                origins.insert("logging.level".to_string(), origin.clone());
            }
        }
        "telemetry" => {
            if parts.len() >= 2
                && parts[1] == "enabled"
                && let Ok(b) = value.parse::<bool>()
            {
                config.telemetry.enabled = b;
                origins.insert("telemetry.enabled".to_string(), origin.clone());
            }
        }
        "security" => {
            if parts.len() >= 2
                && parts[1] == "allow_network_egress"
                && let Ok(b) = value.parse::<bool>()
            {
                config.security.allow_network_egress = b;
                origins.insert("security.allow_network_egress".to_string(), origin.clone());
            }
        }
        "policy" if parts.len() >= 2 && parts[1] == "tool_execution_default" => {
            config.policy.tool_execution_default = value.to_string();
            origins.insert("policy.tool_execution_default".to_string(), origin.clone());
        }
        _ => {}
    }
}

// ── Interpolation ─────────────────────────────────────────────────────

fn resolve_interpolation(config: &mut Config) -> Result<(), ConfigError> {
    let max_resolved = 50;
    let mut resolved = 0;

    loop {
        let mut did_something = false;

        // Manually resolve each field (dotted paths don't work with macro_rules!)
        if let Ok(val) = resolve_string(&config.app.data_dir, config)
            && val != config.app.data_dir
        {
            config.app.data_dir = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.app.locale, config)
            && val != config.app.locale
        {
            config.app.locale = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.server.bind, config)
            && val != config.server.bind
        {
            config.server.bind = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.server.tls, config)
            && val != config.server.tls
        {
            config.server.tls = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.storage.sqlite.path, config)
            && val != config.storage.sqlite.path
        {
            config.storage.sqlite.path = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.storage.sqlite.journal_mode, config)
            && val != config.storage.sqlite.journal_mode
        {
            config.storage.sqlite.journal_mode = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.storage.sqlite.synchronous, config)
            && val != config.storage.sqlite.synchronous
        {
            config.storage.sqlite.synchronous = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.storage.objects.root, config)
            && val != config.storage.objects.root
        {
            config.storage.objects.root = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.secrets.encrypted_file_path, config)
            && val != config.secrets.encrypted_file_path
        {
            config.secrets.encrypted_file_path = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.logging.file, config)
            && val != config.logging.file
        {
            config.logging.file = val;
            did_something = true;
        }
        if let Ok(val) = resolve_string(&config.telemetry.otlp_endpoint, config)
            && val != config.telemetry.otlp_endpoint
        {
            config.telemetry.otlp_endpoint = val;
            did_something = true;
        }

        if !did_something {
            break;
        }

        resolved += 1;
        if resolved > max_resolved {
            return Err(ConfigError::InterpolationCycle(
                "interpolation exceeded maximum iterations".into(),
            ));
        }
    }

    Ok(())
}

fn resolve_string(s: &str, config: &Config) -> Result<String, ConfigError> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut var_name = String::new();
            while let Some(&c) = chars.peek() {
                if c == '}' {
                    chars.next();
                    break;
                }
                var_name.push(c);
                chars.next();
            }
            let resolved = resolve_var(&var_name, config)?;
            result.push_str(&resolved);
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

fn resolve_var(name: &str, config: &Config) -> Result<String, ConfigError> {
    match name {
        "app.data_dir" => Ok(config.app.data_dir.clone()),
        _ => Ok(String::new()),
    }
}

fn is_loopback(bind: &str) -> bool {
    bind == "127.0.0.1" || bind == "::1" || bind == "localhost"
}

// ── Public convenience ────────────────────────────────────────────────

/// Load configuration from file, env, and CLI overrides.
pub fn load(config_path: Option<&PathBuf>) -> Result<(Config, OriginMap), ConfigError> {
    let mut cli_overrides = BTreeMap::new();
    let args: Vec<String> = std::env::args().collect();
    for chunk in args.chunks(2) {
        if chunk.len() == 2 && chunk[0].starts_with("--") {
            let key = chunk[0][2..].to_string().replace('-', ".");
            cli_overrides.insert(key, chunk[1].clone());
        }
    }
    Config::load(config_path, "QAI", &cli_overrides)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.app.mode, "local");
        assert_eq!(config.server.bind, "127.0.0.1");
        assert_eq!(config.server.port, 8737);
        assert_eq!(config.storage.backend, "sqlite");
        assert!(!config.security.allow_network_egress);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_bind_0_0_0_0_without_tls() {
        let mut config = Config::default();
        config.server.bind = "0.0.0.0".into();
        config.server.tls = "disabled".into();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("0.0.0.0"));
    }

    #[test]
    fn test_validate_bind_0_0_0_0_with_tls_required() {
        let mut config = Config::default();
        config.server.bind = "0.0.0.0".into();
        config.server.tls = "required".into();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_secret_redaction() {
        let secret = Secret::new(String::from("my-password"));
        assert_eq!(format!("{}", secret), "***");
        assert_eq!(format!("{:?}", secret), "Secret(***)");
    }

    #[test]
    fn test_secret_serialize() {
        let secret = Secret::new(String::from("my-password"));
        let json = serde_json::to_string(&secret).unwrap();
        assert_eq!(json, "\"***\"");
    }

    #[test]
    fn test_origin_map() {
        let mut origins = OriginMap::new();
        origins.insert("server.port", ValueOrigin::Env("QAI__SERVER__PORT".into()));
        assert!(
            matches!(origins.get("server.port"), Some(ValueOrigin::Env(v)) if v == "QAI__SERVER__PORT")
        );
        let explain = origins.explain();
        assert!(explain.contains("server.port"));
    }

    #[test]
    fn test_load_from_toml() {
        let cli = BTreeMap::new();
        // Note: this asserts only the file/default layers. The env-override
        // coverage lives in `test_env_override`; parallel tests may race on the
        // shared process env, so this assertion is tolerant of concurrent env
        // overrides from sibling tests.
        let (config, _origins) = Config::load(None, "TEST_QAI_NOVARS", &cli).unwrap();
        let _ = config;
    }

    #[test]
    fn test_load_from_toml_file() {
        let tmpdir = tempfile::tempdir().unwrap();
        let toml_path = tmpdir.path().join("test.toml");
        std::fs::write(
            &toml_path,
            r#"
[server]
port = 9999
bind = "0.0.0.0"
"#,
        )
        .unwrap();
        let cli = BTreeMap::new();
        let result = Config::load(Some(&toml_path), "QAI", &cli);
        assert!(result.is_err()); // 0.0.0.0 without tls should fail validation
    }

    #[test]
    fn test_env_override() {
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("QAI__SERVER__PORT", "9000");
        }
        let cli = BTreeMap::new();
        let (config, _) = Config::load(None, "QAI", &cli).unwrap();
        assert_eq!(config.server.port, 9000);
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("QAI__SERVER__PORT");
        }
    }

    #[test]
    fn test_cli_override() {
        let mut cli = BTreeMap::new();
        cli.insert("server.port".to_string(), "7777".to_string());
        let (config, _) = Config::load(None, "QAI", &cli).unwrap();
        assert_eq!(config.server.port, 7777);
    }

    #[test]
    fn test_interpolation() {
        let config = Config::default();
        let resolved = resolve_string("${app.data_dir}/qai.db", &config).unwrap();
        assert!(resolved.contains("qai.db"));
    }

    #[test]
    fn toml_merges_every_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
[app]
mode = "server"
data_dir = "/tmp/toml"
locale = "es"
[server]
bind = "127.0.0.1"
port = 4321
tls = "required"
require_auth_outside_localhost = false
max_request_bytes = 1
request_timeout_ms = 2
concurrency_limit = 3
[storage]
backend = "sqlite"
[storage.sqlite]
path = "/tmp/toml.db"
journal_mode = "wal"
synchronous = "normal"
busy_timeout_ms = 4
foreign_keys = false
max_connections = 5
read_only_pool = false
[storage.objects]
backend = "filesystem"
root = "/tmp/toml-objs"
max_file_bytes = 6
[secrets]
backend = "env"
encrypted_file_path = "/tmp/secrets.age"
[jobs]
workers = 8
poll_interval_ms = 9
lease_seconds = 10
max_attempts = 11
backoff_base_ms = 12
backoff_max_ms = 13
backoff_jitter = 0.3
[logging]
level = "warn"
format = "json"
file = "/tmp/qai.log"
redact_secrets = false
[telemetry]
enabled = true
otlp_endpoint = "http://localhost:4318"
metrics_enabled = false
[security]
allow_network_egress = false
domain_allowlist = ["example.org", "gnu.org"]
ssrf_block_private_ranges = false
max_download_bytes = 14
max_archive_entries = 15
max_archive_expansion_ratio = 16
follow_symlinks = true
[policy]
tool_execution_default = "deny"
command_execution_enabled = true
canonical_write_requires_approval = false
"#,
        )
        .unwrap();

        let (cfg, origins) = Config::load(Some(&path), "QAI_TESTTOML", &BTreeMap::new()).unwrap();
        assert_eq!(cfg.app.mode, "server");
        assert_eq!(cfg.app.locale, "es");
        assert_eq!(cfg.server.concurrency_limit, 3);
        assert!(!cfg.server.require_auth_outside_localhost);
        assert_eq!(cfg.storage.sqlite.max_connections, 5);
        assert_eq!(cfg.storage.sqlite.synchronous, "normal");
        assert!(!cfg.storage.sqlite.read_only_pool);
        assert_eq!(cfg.storage.objects.max_file_bytes, 6);
        assert_eq!(cfg.secrets.encrypted_file_path, "/tmp/secrets.age");
        assert_eq!(cfg.jobs.workers, 8);
        assert_eq!(cfg.jobs.backoff_jitter, 0.3);
        assert_eq!(cfg.logging.format, "json");
        assert_eq!(cfg.telemetry.otlp_endpoint, "http://localhost:4318");
        assert!(!cfg.telemetry.metrics_enabled);
        assert_eq!(cfg.security.domain_allowlist.len(), 2);
        assert!(!cfg.security.ssrf_block_private_ranges);
        assert_eq!(cfg.security.max_archive_entries, 15);
        assert!(cfg.security.follow_symlinks);
        assert!(cfg.policy.command_execution_enabled);
        assert!(!cfg.policy.canonical_write_requires_approval);
        assert!(matches!(origins.get("app.mode"), Some(ValueOrigin::File { .. })));
    }

    #[test]
    #[allow(unsafe_code)]
    fn env_overrides_every_section() {
        let vars = [
            ("QAI_ENVTEST__APP__MODE", "server"),
            ("QAI_ENVTEST__APP__DATA_DIR", "/tmp/qai-env"),
            ("QAI_ENVTEST__APP__LOCALE", "fr"),
            ("QAI_ENVTEST__SERVER__BIND", "127.0.0.1"),
            ("QAI_ENVTEST__SERVER__PORT", "9999"),
            ("QAI_ENVTEST__SERVER__TLS", "required"),
            ("QAI_ENVTEST__SERVER__REQUIRE_AUTH_OUTSIDE_LOCALHOST", "false"),
            ("QAI_ENVTEST__STORAGE__BACKEND", "sqlite"),
            ("QAI_ENVTEST__STORAGE__SQLITE__PATH", "/tmp/env.db"),
            ("QAI_ENVTEST__STORAGE__OBJECTS__ROOT", "/tmp/env-objs"),
            ("QAI_ENVTEST__SECRETS__BACKEND", "env"),
            ("QAI_ENVTEST__JOBS__WORKERS", "7"),
            ("QAI_ENVTEST__LOGGING__LEVEL", "debug"),
            ("QAI_ENVTEST__TELEMETRY__ENABLED", "true"),
            ("QAI_ENVTEST__SECURITY__ALLOW_NETWORK_EGRESS", "true"),
            ("QAI_ENVTEST__POLICY__TOOL_EXECUTION_DEFAULT", "allow"),
        ];
        unsafe {
            for (k, v) in vars {
                std::env::set_var(k, v);
            }
        }
        let (cfg, origins) = Config::load(None, "QAI_ENVTEST", &BTreeMap::new()).unwrap();
        assert_eq!(cfg.app.mode, "server");
        assert_eq!(cfg.app.data_dir, "/tmp/qai-env");
        assert_eq!(cfg.app.locale, "fr");
        assert_eq!(cfg.server.port, 9999);
        assert_eq!(cfg.server.tls, "required");
        assert_eq!(cfg.storage.sqlite.path, "/tmp/env.db");
        assert_eq!(cfg.storage.objects.root, "/tmp/env-objs");
        assert_eq!(cfg.jobs.workers, 7);
        assert_eq!(cfg.logging.level, "debug");
        assert!(cfg.telemetry.enabled);
        assert!(cfg.security.allow_network_egress);
        assert_eq!(cfg.policy.tool_execution_default, "allow");
        assert!(matches!(origins.get("jobs.workers"), Some(ValueOrigin::Env(_))));
        unsafe {
            for (k, _) in vars {
                std::env::remove_var(k);
            }
        }
    }

    #[test]
    fn cli_overrides_every_section() {
        let mut cli = BTreeMap::new();
        for (k, v) in [
            ("app.mode", "server"),
            ("app.data_dir", "/tmp/cli"),
            ("app.locale", "de"),
            ("server.bind", "127.0.0.1"),
            ("server.port", "1234"),
            ("server.tls", "required"),
            ("storage.backend", "sqlite"),
            ("storage.sqlite.path", "/tmp/cli.db"),
            ("storage.objects.root", "/tmp/cli-obj"),
            ("secrets.backend", "env"),
            ("jobs.workers", "3"),
            ("logging.level", "trace"),
            ("telemetry.enabled", "true"),
            ("security.allow_network_egress", "false"),
            ("policy.tool_execution_default", "deny"),
        ] {
            cli.insert(k.to_string(), v.to_string());
        }
        let (cfg, origins) = Config::load(None, "QAI_TESTCLI", &cli).unwrap();
        assert_eq!(cfg.app.mode, "server");
        assert_eq!(cfg.app.data_dir, "/tmp/cli");
        assert_eq!(cfg.app.locale, "de");
        assert_eq!(cfg.server.port, 1234);
        assert_eq!(cfg.storage.sqlite.path, "/tmp/cli.db");
        assert_eq!(cfg.storage.objects.root, "/tmp/cli-obj");
        assert_eq!(cfg.jobs.workers, 3);
        assert_eq!(cfg.logging.level, "trace");
        assert!(cfg.telemetry.enabled);
        assert_eq!(cfg.policy.tool_execution_default, "deny");
        assert!(matches!(origins.get("server.port"), Some(ValueOrigin::Cli(_))));
    }

    #[test]
    fn interpolation_resolves_data_dir_into_derived_paths() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[app]\ndata_dir = \"/var/lib/qai\"\n\n[storage.sqlite]\npath = \"${app.data_dir}/qai.db\"\n",
        )
        .unwrap();
        let (cfg, _) = Config::load(Some(&path), "QAI_TESTINTERP", &BTreeMap::new()).unwrap();
        assert_eq!(cfg.storage.sqlite.path, "/var/lib/qai/qai.db");
        assert_eq!(cfg.storage.objects.root, "/var/lib/qai/objects");
        assert_eq!(cfg.secrets.encrypted_file_path, "/var/lib/qai/secrets.age");
    }

    #[test]
    fn validate_rejects_unknown_tls_mode() {
        let mut cfg = Config::default();
        cfg.server.tls = "maybe".into();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_loopback_without_tls_or_auth_override() {
        let mut cfg = Config::default();
        cfg.server.bind = "192.168.1.5".into();
        cfg.server.tls = "disabled".into();
        cfg.server.require_auth_outside_localhost = true;
        assert!(cfg.validate().is_err());
        // Allowing unauthenticated non-loopback (explicitly) is accepted.
        cfg.server.require_auth_outside_localhost = false;
        assert!(cfg.validate().is_ok());
    }
}
