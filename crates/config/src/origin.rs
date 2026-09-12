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
