use anyhow::Result;
use serde_json::Value;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

use crate::{api, context};
use crate::api::{to_api_kind, to_singular_kind};

/// Strip noisy server-managed fields from the resource before showing in editor
fn strip_server_fields(mut value: Value) -> Value {
    if let Some(obj) = value.as_object_mut() {
        obj.remove("hash_code");
        obj.remove("state");
        obj.remove("deletion");
    }
    value
}

/// Open a resource for editing in $VISUAL / $EDITOR / vi, and save changes back to the API.
///
/// Workflow:
/// 1. Fetch the resource
/// 2. Strip noisy server fields (hash_code, state, deletion)
/// 3. Inject kind: field
/// 4. Write to a temp file with a recognizable name
/// 5. Open in user's $VISUAL / $EDITOR / vi (blocking)
/// 6. Re-read the file; if unchanged, exit with "no changes"
/// 7. Re-fetch hash_code from server (for conflict detection)
/// 8. Parse edited YAML, inject hash_code, and POST via apply_object()
/// 9. Handle 409 conflicts with a clear message
/// 10. Print success message
pub async fn run(kind: &str, id: &str) -> Result<()> {
    let ctx = context::require_current()?;
    let api_kind = to_api_kind(kind);

    // 1. Fetch the resource
    let resource = api::get_kind(&ctx.url, &ctx.token, &api_kind, id).await?;

    // 2 & 3. Strip server fields and inject kind
    let stripped = strip_server_fields(resource.clone());
    let mut for_editor = stripped;
    if let Some(obj) = for_editor.as_object_mut() {
        obj.insert("kind".to_string(), serde_json::json!(to_singular_kind(&api_kind)));
    }

    // 4. Write to a temp file with a recognizable name
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| anyhow::anyhow!("failed to create temp file: {}", e))?;
    let yaml_content = serde_yaml::to_string(&for_editor)
        .map_err(|e| anyhow::anyhow!("failed to serialize resource to YAML: {}", e))?;
    temp_file
        .write_all(yaml_content.as_bytes())
        .map_err(|e| anyhow::anyhow!("failed to write to temp file: {}", e))?;
    temp_file.flush()?;

    let temp_path = temp_file.path().to_path_buf();

    // 5. Open in editor (blocking)
    let editor = std::env::var("CR1T_EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to launch editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(anyhow::anyhow!("editor exited with error"));
    }

    // 6. Re-read the file; if unchanged, exit with "no changes"
    let edited_yaml = std::fs::read_to_string(&temp_path)
        .map_err(|e| anyhow::anyhow!("failed to read edited file: {}", e))?;

    if edited_yaml.trim() == yaml_content.trim() {
        println!("{}/{} — no changes", kind, id);
        return Ok(());
    }

    // 7. Parse the edited YAML and extract kind/id
    let mut edited_value: Value = serde_yaml::from_str(&edited_yaml)
        .map_err(|e| anyhow::anyhow!("failed to parse edited YAML: {}", e))?;

    // Verify kind and id are still present
    let edited_kind = edited_value
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("edited document is missing required field 'kind'"))?
        .to_string();

    let edited_id = edited_value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("edited document is missing required field 'id'"))?
        .to_string();

    // Kind and ID should match the originals
    if edited_kind != to_singular_kind(&api_kind) {
        return Err(anyhow::anyhow!(
            "kind mismatch: expected '{}', got '{}'",
            to_singular_kind(&api_kind),
            edited_kind
        ));
    }
    if edited_id != id {
        return Err(anyhow::anyhow!(
            "id mismatch: expected '{}', got '{}'",
            id,
            edited_id
        ));
    }

    // Strip 'kind' from the edited value (not a DB field)
    if let Some(obj) = edited_value.as_object_mut() {
        obj.remove("kind");
    }

    // 7. Re-fetch hash_code from server (same pattern as apply.rs)
    if let Some(existing) = api::try_get_kind(&ctx.url, &ctx.token, &api_kind, id).await? {
        if let Some(hash) = existing.get("hash_code").and_then(|v| v.as_str()) {
            if let Some(obj) = edited_value.as_object_mut() {
                obj.insert(
                    "hash_code".to_string(),
                    serde_json::Value::String(hash.to_string()),
                );
            }
        }
    }

    // 8 & 9. POST via apply_object() and handle 409 conflicts
    api::apply_object(&ctx.url, &ctx.token, &api_kind, id, edited_value).await
        .map_err(|e| {
            if e.to_string().contains("(409 Conflict)") {
                anyhow::anyhow!("{}/{} was modified since last read — re-run edit to retry", kind, id)
            } else {
                e
            }
        })?;

    // 10. Success
    println!("{}/{} edited", kind, id);
    Ok(())
}
