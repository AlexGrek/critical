use std::io::Read;
use std::path::Path;

use anyhow::{bail, Result};
use serde::de::Deserialize;
use serde_json::Value;

use crate::{api, context};

/// Pluralize a singular kind name to get the API collection name.
/// e.g. "group" → "groups", "user" → "users", "project" → "projects"
fn to_api_kind(kind: &str) -> String {
    format!("{}s", kind)
}

/// Parse a YAML string (potentially multi-document) into a list of `(kind, id, body)` tuples.
/// `kind` is stripped from `body` since it's only used for routing, not stored in the DB.
fn parse_documents(content: &str) -> Result<Vec<(String, String, Value)>> {
    let mut docs = Vec::new();

    for document in serde_yaml::Deserializer::from_str(content) {
        let mut value: Value = Value::deserialize(document)
            .map_err(|e| anyhow::anyhow!("failed to parse YAML document: {}", e))?;

        // Skip null documents — these appear for empty input or trailing `---` separators.
        if value.is_null() {
            continue;
        }

        let kind = value
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("document is missing required field 'kind'"))?
            .to_string();

        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("{}: document is missing required field 'id'", kind))?
            .to_string();

        // Strip 'kind' — not a DB field, only used for routing
        if let Some(obj) = value.as_object_mut() {
            obj.remove("kind");
        }

        docs.push((kind, id, value));
    }

    Ok(docs)
}

pub async fn run(filename: Option<&Path>) -> Result<()> {
    let ctx = context::require_current()?;

    let content = match filename {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path.display(), e))?,
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| anyhow::anyhow!("failed to read stdin: {}", e))?;
            buf
        }
    };

    let documents = parse_documents(&content)?;

    if documents.is_empty() {
        bail!("no valid YAML documents found in input");
    }

    for (kind, id, mut body) in documents {
        let api_kind = to_api_kind(&kind);

        // Fetch the existing resource to obtain its hash_code. If the resource
        // does not exist yet this is a create, and no hash is injected. Any
        // other error (auth, network) is surfaced immediately.
        if let Some(existing) = api::try_get_kind(&ctx.url, &ctx.token, &api_kind, &id).await? {
            if let Some(hash) = existing.get("hash_code").and_then(|v| v.as_str()) {
                if let Some(obj) = body.as_object_mut() {
                    obj.insert("hash_code".to_string(), serde_json::Value::String(hash.to_string()));
                }
            }
        }

        api::apply_object(&ctx.url, &ctx.token, &api_kind, &id, body).await
            .map_err(|e| {
                // api.rs formats errors as "{message} ({status})" — detect 409 by suffix.
                if e.to_string().contains("(409 Conflict)") {
                    anyhow::anyhow!("{}/{} was modified since last read — re-run apply to retry", kind, id)
                } else {
                    e
                }
            })?;
        println!("{}/{} applied", kind, id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- to_api_kind ---

    #[test]
    fn kind_is_pluralized() {
        assert_eq!(to_api_kind("group"), "groups");
        assert_eq!(to_api_kind("user"), "users");
        assert_eq!(to_api_kind("project"), "projects");
        assert_eq!(to_api_kind("membership"), "memberships");
        assert_eq!(to_api_kind("ticket"), "tickets");
    }

    // --- parse_documents: happy paths ---

    #[test]
    fn parse_single_document() {
        let yaml = "kind: group\nid: g_test\nname: Test Group\n";
        let docs = parse_documents(yaml).unwrap();

        assert_eq!(docs.len(), 1);
        let (kind, id, body) = &docs[0];
        assert_eq!(kind, "group");
        assert_eq!(id, "g_test");
        assert_eq!(body["name"].as_str().unwrap(), "Test Group");
        // 'kind' must be stripped from body
        assert!(body.get("kind").is_none(), "kind should be removed from body");
        // 'id' is preserved in body (backend upsert re-injects it anyway)
        assert_eq!(body["id"].as_str().unwrap(), "g_test");
    }

    #[test]
    fn parse_multi_document() {
        let yaml = "kind: group\nid: g_a\nname: Alpha\n---\nkind: group\nid: g_b\nname: Beta\n";
        let docs = parse_documents(yaml).unwrap();

        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].0, "group");
        assert_eq!(docs[0].1, "g_a");
        assert_eq!(docs[1].0, "group");
        assert_eq!(docs[1].1, "g_b");
    }

    #[test]
    fn parse_mixed_kinds_multi_document() {
        let yaml = "kind: group\nid: g_eng\nname: Engineering\n---\nkind: user\nid: u_alice\n";
        let docs = parse_documents(yaml).unwrap();

        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].0, "group");
        assert_eq!(docs[1].0, "user");
        assert_eq!(docs[1].1, "u_alice");
    }

    #[test]
    fn parse_nested_fields_are_preserved() {
        let yaml =
            "kind: user\nid: u_bob\npersonal:\n  name: Bob Smith\n  job_title: Engineer\n";
        let docs = parse_documents(yaml).unwrap();

        assert_eq!(docs.len(), 1);
        let body = &docs[0].2;
        assert_eq!(body["personal"]["name"].as_str().unwrap(), "Bob Smith");
        assert_eq!(body["personal"]["job_title"].as_str().unwrap(), "Engineer");
    }

    #[test]
    fn parse_empty_input_returns_empty_vec() {
        // serde_yaml yields a null document for "" — we skip it → empty vec.
        // run() turns an empty vec into the "no valid YAML documents" error.
        let docs = parse_documents("").unwrap();
        assert!(docs.is_empty());
    }

    #[test]
    fn parse_trailing_separator_skipped() {
        // A trailing `---` produces a null document that should be silently skipped.
        let yaml = "kind: group\nid: g_x\nname: X\n---\n";
        let docs = parse_documents(yaml).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].1, "g_x");
    }

    // --- parse_documents: error paths ---

    #[test]
    fn parse_missing_kind_returns_error() {
        let yaml = "id: g_test\nname: No Kind\n";
        let err = parse_documents(yaml).unwrap_err();
        assert!(
            err.to_string().contains("kind"),
            "error should mention 'kind', got: {}",
            err
        );
    }

    #[test]
    fn parse_missing_id_returns_error() {
        let yaml = "kind: group\nname: No ID\n";
        let err = parse_documents(yaml).unwrap_err();
        assert!(
            err.to_string().contains("id"),
            "error should mention 'id', got: {}",
            err
        );
    }

    #[test]
    fn parse_invalid_yaml_returns_error() {
        let yaml = "kind: group\n  bad_indent: [\n";
        assert!(parse_documents(yaml).is_err(), "invalid YAML should fail");
    }

    #[test]
    fn parse_error_on_second_document_fails() {
        // First doc is valid, second is missing 'id'
        let yaml = "kind: group\nid: g_ok\n---\nkind: group\nname: missing-id\n";
        let err = parse_documents(yaml).unwrap_err();
        assert!(err.to_string().contains("id"));
    }

    #[test]
    fn kind_field_not_in_body_after_parse() {
        let yaml = "kind: project\nid: p_alpha\ndescription: A project\n";
        let docs = parse_documents(yaml).unwrap();
        let body = &docs[0].2;
        assert!(body.get("kind").is_none());
        assert_eq!(body["description"].as_str().unwrap(), "A project");
    }
}
