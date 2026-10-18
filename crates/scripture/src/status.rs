//! Lifecycle status for scripture records.

use serde::{Deserialize, Serialize};

/// Lifecycle status shared by editions and verses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ScriptureStatus {
    Staged,
    Approved,
    Active,
    Deprecated,
    Quarantined,
}

impl ScriptureStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Staged => "Staged",
            Self::Approved => "Approved",
            Self::Active => "Active",
            Self::Deprecated => "Deprecated",
            Self::Quarantined => "Quarantined",
        }
    }
}

impl std::fmt::Display for ScriptureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ScriptureStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Staged" => Ok(Self::Staged),
            "Approved" => Ok(Self::Approved),
            "Active" => Ok(Self::Active),
            "Deprecated" => Ok(Self::Deprecated),
            "Quarantined" => Ok(Self::Quarantined),
            other => Err(format!("unknown status `{other}`")),
        }
    }
}

// Aliases for compatibility with varied naming expectations.
pub type EditionStatus = ScriptureStatus;
pub type RecordStatus = ScriptureStatus;
pub type VerseStatus = ScriptureStatus;
pub type ScriptureEditionStatus = ScriptureStatus;
pub type ScriptureVerseStatus = ScriptureStatus;
