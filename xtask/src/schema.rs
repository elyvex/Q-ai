//! `cargo xtask gen-schema` — emit JSON Schemas deterministically.
//!
//! Generates a stable `docs/schemas/index.json` describing the schemas Q-ai publishes
//! during Phase 0, so `gen-schema && git diff --exit-code docs/schemas/` is quiet on a
//! clean tree (the `schemas` CI job). Individual schema documents are produced by the
//! crates that own them (config in `config`/T12, manifest in `sources`/T29, doctor in
//! `cli`/T53); this tasks owns the index and the directory contract.

use anyhow::{Context, Result};

/// The schemas we commit to publishing this phase.
const SCHEMAS: &[&str] = &["config.v1", "source-manifest.v1", "doctor.v1", "error.v1"];

pub const SCHEMAS_SUBDIR: &str = "docs/schemas";

pub fn run() -> Result<()> {
    let root = std::env::current_dir()?;
    let out_dir = root.join(SCHEMAS_SUBDIR);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("create_dir_all {}", out_dir.display()))?;

    // Deterministic index — no timestamps so the CI diff check is stable.
    let index = serde_json::json!({
        "schema": "https://json-schema.org/draft/2020-12/schema",
        "description": "Index of JSON Schemas published by qai during Phase 0.",
        "schemas": SCHEMAS.iter().map(|name| {
            serde_json::json!({ "name": name, "file": format!("{name}.schema.json") })
        }).collect::<Vec<_>>(),
    });

    let index_path = out_dir.join("index.json");
    std::fs::write(&index_path, serde_json::to_string_pretty(&index)?)
        .with_context(|| format!("write {}", index_path.display()))?;

    // Ensure each schema file exists (empty placeholder owned by its crate). This makes
    // the directory self-consistent even before the owning crate lands (T12/T29/T53).
    for name in SCHEMAS {
        let path = out_dir.join(format!("{name}.schema.json"));
        if !path.exists() {
            let empty = serde_json::json!({
                "schema": "https://json-schema.org/draft/2020-12/schema",
                "title": name,
                "description": "Populated by the owning crate during Phase 0.",
                "type": "object",
            });
            std::fs::write(&path, serde_json::to_string_pretty(&empty)?)
                .with_context(|| format!("write {}", path.display()))?;
        }
    }

    println!("gen-schema: OK — wrote schema index under {}", out_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_is_deterministic() {
        // Running twice must produce byte-identical output (CI uses `git diff --exit-code`).
        let dir = std::env::temp_dir().join(format!("qai_schema_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);

        std::env::set_current_dir(&dir).unwrap();
        run().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        run().unwrap();

        let first = std::fs::read_to_string(dir.join(SCHEMAS_SUBDIR).join("index.json")).unwrap();
        let second = std::fs::read_to_string(dir.join(SCHEMAS_SUBDIR).join("index.json")).unwrap();
        assert_eq!(first, second);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
