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

/// Validate a JSON instance against a JSON Schema subset.
///
/// Supports `type` (string or array of strings), `enum`, `required`,
/// `properties`, `items`, and `additionalProperties: false` — enough for the
/// Phase-0 schemas. Returns one message per violation.
pub fn validate_instance(instance: &serde_json::Value, schema: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    validate_at(instance, schema, "$", &mut errors);
    errors
}

fn type_matches(value: &serde_json::Value, expected: &str) -> bool {
    match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn validate_at(
    value: &serde_json::Value,
    schema: &serde_json::Value,
    path: &str,
    errors: &mut Vec<String>,
) {
    // type
    match schema.get("type") {
        Some(serde_json::Value::String(t)) => {
            if !type_matches(value, t) {
                errors.push(format!("{path}: expected type {t}"));
            }
        }
        Some(serde_json::Value::Array(types)) => {
            let ok = types.iter().filter_map(|t| t.as_str()).any(|t| type_matches(value, t));
            if !ok {
                errors.push(format!("{path}: does not match any of {types:?}"));
            }
        }
        _ => {}
    }

    // enum
    if let Some(allowed) = schema.get("enum").and_then(|e| e.as_array())
        && !allowed.iter().any(|a| a == value)
    {
        errors.push(format!("{path}: {value} is not one of {allowed:?}"));
    }

    // required + properties
    if let Some(obj) = value.as_object() {
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            for key in required.iter().filter_map(|k| k.as_str()) {
                if !obj.contains_key(key) {
                    errors.push(format!("{path}: missing required property `{key}`"));
                }
            }
        }
        if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
            for (key, subschema) in props {
                if let Some(child) = obj.get(key) {
                    validate_at(child, subschema, &format!("{path}.{key}"), errors);
                }
            }
            if schema.get("additionalProperties") == Some(&serde_json::Value::Bool(false)) {
                for key in obj.keys() {
                    if !props.contains_key(key) {
                        errors.push(format!("{path}: unexpected property `{key}`"));
                    }
                }
            }
        }
    }

    // items
    if let (Some(items), Some(arr)) = (schema.get("items"), value.as_array()) {
        for (i, child) in arr.iter().enumerate() {
            validate_at(child, items, &format!("{path}[{i}]"), errors);
        }
    }
}

/// `cargo xtask validate <instance.json> <schema.json>`.
pub fn validate_cmd(instance_path: &std::path::Path, schema_path: &std::path::Path) -> Result<()> {
    let instance: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(instance_path)
            .with_context(|| format!("read {}", instance_path.display()))?,
    )
    .context("instance is not valid JSON")?;
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(schema_path)
            .with_context(|| format!("read {}", schema_path.display()))?,
    )
    .context("schema is not valid JSON")?;

    let errors = validate_instance(&instance, &schema);
    if errors.is_empty() {
        println!("validate: OK — {} satisfies {}", instance_path.display(), schema_path.display());
        Ok(())
    } else {
        for e in &errors {
            eprintln!("validate: {e}");
        }
        anyhow::bail!("validate: {} violation(s)", errors.len())
    }
}

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

    // Ensure each schema file exists (placeholder owned by its crate).
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

    fn doctor_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["checks"],
            "additionalProperties": false,
            "properties": {
                "checks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "status", "summary", "remedy", "next_command"],
                        "additionalProperties": false,
                        "properties": {
                            "id": {"type": "string"},
                            "status": {"enum": ["pass", "warn", "fail", "skipped"]},
                            "summary": {"type": "string"},
                            "remedy": {"type": ["string", "null"]},
                            "next_command": {"type": ["string", "null"]}
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn accepts_a_conforming_instance() {
        let instance = serde_json::json!({
            "checks": [
                {"id": "database.reachable", "status": "pass", "summary": "ok",
                 "remedy": null, "next_command": null}
            ]
        });
        assert!(validate_instance(&instance, &doctor_schema()).is_empty());
    }

    #[test]
    fn reports_bad_enum_and_missing_and_extra_properties() {
        let instance = serde_json::json!({
            "checks": [
                {"id": "x", "status": "exploded", "summary": "s", "remedy": null, "next_command": null, "extra": 1}
            ]
        });
        let errors = validate_instance(&instance, &doctor_schema());
        assert!(errors.iter().any(|e| e.contains("status")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("unexpected property `extra`")), "{errors:?}");
    }

    #[test]
    fn reports_a_missing_required_top_level_key() {
        let instance = serde_json::json!({});
        let errors = validate_instance(&instance, &doctor_schema());
        assert!(
            errors.iter().any(|e| e.contains("missing required property `checks`")),
            "{errors:?}"
        );
    }

    #[test]
    fn accepts_nullable_fields() {
        let instance = serde_json::json!({
            "checks": [
                {"id": "x", "status": "skipped", "summary": "s",
                 "remedy": "do a thing", "next_command": "qai doctor"}
            ]
        });
        assert!(validate_instance(&instance, &doctor_schema()).is_empty());
    }
}
