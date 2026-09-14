use std::collections::BTreeMap;
use std::fmt;

/// Where a config value came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueOrigin {
    Default,
    File { path: String, line: Option<usize> },
    Env(String),
    Cli(String),
}

impl fmt::Display for ValueOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::File { path, line } => {
                if let Some(l) = line {
                    write!(f, "file {path}:{l}")
                } else {
                    write!(f, "file {path}")
                }
            }
            Self::Env(var) => write!(f, "env {var}"),
            Self::Cli(flag) => write!(f, "cli {flag}"),
        }
    }
}

/// Tracks per-field origin metadata.
#[derive(Debug, Clone, Default)]
pub struct OriginMap {
    inner: BTreeMap<String, ValueOrigin>,
}

impl OriginMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<String>, origin: ValueOrigin) {
        self.inner.insert(key.into(), origin);
    }

    pub fn get(&self, key: &str) -> Option<&ValueOrigin> {
        self.inner.get(key)
    }

    pub fn explain(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.inner {
            out.push_str(&format!("{k} = {v}\n"));
        }
        out
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ValueOrigin)> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_every_variant() {
        assert_eq!(ValueOrigin::Default.to_string(), "default");
        assert_eq!(
            ValueOrigin::File { path: "c.toml".into(), line: Some(3) }.to_string(),
            "file c.toml:3"
        );
        assert_eq!(
            ValueOrigin::File { path: "c.toml".into(), line: None }.to_string(),
            "file c.toml"
        );
        assert_eq!(ValueOrigin::Env("QAI__X".into()).to_string(), "env QAI__X");
        assert_eq!(ValueOrigin::Cli("--port".into()).to_string(), "cli --port");
    }

    #[test]
    fn map_len_iter_and_explain() {
        let mut map = OriginMap::new();
        assert!(map.is_empty());
        map.insert("server.port", ValueOrigin::Default);
        map.insert("server.bind", ValueOrigin::Env("QAI__SERVER__BIND".into()));
        assert_eq!(map.len(), 2);
        assert_eq!(map.iter().count(), 2);
        let explain = map.explain();
        assert!(explain.contains("server.port = default"));
        assert!(explain.contains("server.bind = env QAI__SERVER__BIND"));
    }
}
