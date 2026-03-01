use anyhow::{Result, anyhow};
use arangors::document::Document;
use serde_json::{Value, json};

use crit_shared::util_models::*;

use super::ArangoDb;

impl ArangoDb {
    /// Patch a user document to update a single image ULID field (`avatar_ulid` or
    /// `wallpaper_ulid`). Uses AQL UPDATE WITH (non-destructive merge) rather than
    /// REPLACE so all other user fields are preserved.
    pub async fn patch_user_image_ulid(
        &self,
        user_id: &str,
        field: &str,
        ulid: Option<&str>,
    ) -> Result<()> {
        let value = ulid
            .map(|u| Value::String(u.to_string()))
            .unwrap_or(Value::Null);
        let patch = serde_json::json!({ field: value });
        let query = r#"
            FOR doc IN users
              FILTER doc._key == @id
              UPDATE doc WITH @patch IN users
        "#;
        let vars = std::collections::HashMap::from([
            ("id", Value::String(user_id.to_string())),
            ("patch", patch),
        ]);
        self.aql::<Value>(query, vars).await?;
        Ok(())
    }

    /// Soft-delete a document: capture connected membership edges into deletion info,
    /// then mark the document with a `deletion` field. Edges are NOT removed here —
    /// that is handled by the controller's `after_delete` hook so cascade logic works.
    /// Returns an error if the document doesn't exist (or is already deleted).
    pub async fn generic_soft_delete(
        &self,
        collection: &str,
        key: &str,
        deleted_by: &str,
    ) -> Result<()> {
        let from_path = format!("{}/{}", collection, key);
        // When deleting a group, also capture edges of members pointing TO this group
        let to_path = format!("groups/{}", key);

        // Find all membership edges connected to this document
        let edge_query = r#"
            FOR m IN memberships
                FILTER m._from == @from_path OR m._to == @to_path
                RETURN { key: m._key, `from`: m._from, `to`: m._to }
        "#;
        let vars = std::collections::HashMap::from([
            ("from_path", Value::String(from_path)),
            ("to_path", Value::String(to_path)),
        ]);

        let edges: Vec<Value> = self.aql(edge_query, vars).await?;

        let disconnected_edges: Vec<DisconnectedEdge> = edges
            .into_iter()
            .filter_map(|e| {
                Some(DisconnectedEdge {
                    collection: "memberships".to_string(),
                    key: e.get("key")?.as_str()?.to_string(),
                    from: e.get("from")?.as_str()?.to_string(),
                    to: e.get("to")?.as_str()?.to_string(),
                })
            })
            .collect();

        let deletion = DeletionInfo {
            deleted_at: chrono::Utc::now(),
            deleted_by: deleted_by.to_string(),
            disconnected_edges,
        };
        let deletion_val = serde_json::to_value(&deletion).map_err(|e| anyhow!(e))?;

        let update_query = r#"
            LET existing = DOCUMENT(@@col, @key)
            FILTER existing != null AND existing.deletion == null
            UPDATE existing WITH { deletion: @deletion } IN @@col
            RETURN NEW
        "#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("key", Value::String(key.to_string())),
            ("deletion", deletion_val),
        ]);
        let result: Vec<Value> = self.aql(update_query, vars).await?;

        if result.is_empty() {
            return Err(anyhow!("document not found or already deleted: {}/{}", collection, key));
        }

        Ok(())
    }

    /// Write an immutable snapshot of a resource's desired state to `resource_history`.
    /// Revision numbers are 1-based and auto-incremented per resource.
    pub async fn write_history_entry(
        &self,
        kind: &str,
        key: &str,
        snapshot: Value,
        changed_by: &str,
    ) -> Result<()> {
        // Count existing history entries for this resource to determine next revision
        let count_query = r#"
            RETURN LENGTH(
                FOR h IN resource_history
                    FILTER h.resource_kind == @kind AND h.resource_key == @key
                    RETURN 1
            )
        "#;
        let vars = std::collections::HashMap::from([
            ("kind", Value::String(kind.to_string())),
            ("key", Value::String(key.to_string())),
        ]);
        let counts: Vec<u64> = self.aql(count_query, vars).await?;
        let revision = counts.into_iter().next().unwrap_or(0) + 1;

        let history_id = format!("{}_{}_{:06}", kind, key, revision);
        let entry = HistoryEntry {
            id: history_id,
            resource_kind: kind.to_string(),
            resource_key: key.to_string(),
            revision,
            snapshot,
            changed_by: changed_by.to_string(),
            changed_at: chrono::Utc::now(),
        };

        let entry_val = serde_json::to_value(&entry).map_err(|e| anyhow!(e))?;
        self.resource_history
            .create_document(Document::new(entry_val), Default::default())
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(())
    }

    /// Fetch the most recent history entry for a resource (highest revision).
    /// Returns `None` if no history exists yet.
    pub async fn get_latest_history_entry(&self, kind: &str, key: &str) -> Result<Option<Value>> {
        let query = r#"
            FOR h IN resource_history
                FILTER h.resource_kind == @kind AND h.resource_key == @key
                SORT h.revision DESC
                LIMIT 1
                RETURN h
        "#;
        let vars = std::collections::HashMap::from([
            ("kind", Value::String(kind.to_string())),
            ("key", Value::String(key.to_string())),
        ]);
        let mut result: Vec<Value> = self.aql(query, vars).await?;
        Ok(result.pop())
    }

    /// Write a runtime event associated with a resource to `resource_events`.
    pub async fn write_event(
        &self,
        kind: &str,
        key: &str,
        event_type: &str,
        actor: Option<&str>,
        details: Option<Value>,
    ) -> Result<()> {
        // Build a unique event ID using nanosecond timestamp + event info
        let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_else(|| chrono::Utc::now().timestamp_micros());
        let event_id = format!("ev_{}_{}", event_type, ts_ns);

        let event = ResourceEvent {
            id: event_id,
            resource_kind: kind.to_string(),
            resource_key: key.to_string(),
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now(),
            actor: actor.map(String::from),
            details,
        };

        let event_val = serde_json::to_value(&event).map_err(|e| anyhow!(e))?;
        self.resource_events
            .create_document(Document::new(event_val), Default::default())
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(())
    }

    /// List all non-system ArangoDB collections in the current database.
    /// Returns each collection as `{ "name": "..." }`.
    pub async fn list_collections(&self) -> Result<Vec<Value>> {
        let collections = self
            .db
            .accessible_collections()
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(collections
            .into_iter()
            .filter(|c| !c.is_system)
            .map(|c| json!({ "name": c.name }))
            .collect())
    }

    /// Return every document in `collection` as raw JSON.
    /// Rejects system collections (names starting with `_`).
    pub async fn dump_collection(&self, collection: &str) -> Result<Vec<Value>> {
        if collection.starts_with('_') {
            return Err(anyhow!("access to system collections is not allowed"));
        }
        let query = "FOR doc IN @@col RETURN doc";
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
        ]);
        self.aql(query, vars).await
    }
}
