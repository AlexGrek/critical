use std::collections::HashMap;

use anyhow::Result;
use crit_shared::event_models::Event;
use serde_json::Value;

use super::{ArangoDb, PaginatedResult};

impl ArangoDb {
    /// Insert a system event into the `system_events` collection.
    pub async fn write_system_event(&self, event: Event) -> Result<()> {
        let doc = serde_json::to_value(&event)?;
        let query = "INSERT @doc INTO system_events";
        let mut vars: HashMap<&str, Value> = HashMap::new();
        vars.insert("doc", doc);
        self.aql::<Value>(query, vars).await?;
        Ok(())
    }

    /// List system events with optional kind/priority filters and keyset pagination.
    /// `cursor` is the last-seen `_key` for keyset pagination.
    pub async fn list_system_events(
        &self,
        kind: Option<&str>,
        priority: Option<&str>,
        limit: u32,
        cursor: Option<String>,
    ) -> Result<PaginatedResult> {
        let mut filters = Vec::new();
        let mut vars: HashMap<&str, Value> = HashMap::new();

        if kind.is_some() {
            filters.push("doc.kind == @kind");
            vars.insert("kind", Value::String(kind.unwrap().to_string()));
        }
        if priority.is_some() {
            filters.push("doc.priority == @priority");
            vars.insert("priority", Value::String(priority.unwrap().to_string()));
        }
        if cursor.is_some() {
            filters.push("doc._key > @cursor");
            vars.insert("cursor", Value::String(cursor.unwrap()));
        }

        let filter_clause = if filters.is_empty() {
            String::new()
        } else {
            format!("FILTER {}", filters.join(" AND "))
        };

        vars.insert("limit", Value::Number((limit + 1).into()));

        let query = format!(
            "FOR doc IN system_events {filter_clause} SORT doc.moment DESC LIMIT @limit RETURN doc"
        );

        let mut docs: Vec<Value> = self.aql::<Value>(&query, vars).await?;

        let has_more = docs.len() > limit as usize;
        if has_more {
            docs.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            docs.last()
                .and_then(|d| d.get("_key"))
                .and_then(|k| k.as_str())
                .map(String::from)
        } else {
            None
        };

        Ok(PaginatedResult {
            docs,
            next_cursor,
            has_more,
        })
    }
}
