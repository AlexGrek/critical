use anyhow::{Result, anyhow};
use serde_json::{Value, json};

use super::{ArangoDb, PaginatedResult};

impl ArangoDb {
    //
    // ------------------- GENERIC DOCUMENT OPERATIONS (GITOPS) --------------------
    //

    /// Ensure a collection exists, creating it if needed. Returns the collection name for use in AQL.
    pub async fn ensure_collection(&self, collection: &str) -> Result<()> {
        // Try to get it; if it fails, create it (ignore race conditions).
        if self.db.collection(collection).await.is_err() {
            let _ = self.db.create_collection(collection).await;
        }
        Ok(())
    }

    pub async fn generic_list(
        &self,
        collection: &str,
        fields: Option<&[&str]>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<PaginatedResult> {
        // Build the RETURN clause (with or without projection)
        let return_clause = match fields {
            Some(f) => {
                let quoted: Vec<String> = f.iter().map(|s| format!("\"{}\"", s)).collect();
                format!("RETURN KEEP(doc, {})", quoted.join(", "))
            }
            None => "RETURN doc".to_string(),
        };

        // Build the full query
        let mut vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
        ]);

        let cursor_filter = if let Some(c) = cursor {
            vars.insert("cursor", Value::String(c.to_string()));
            "FILTER doc._key > @cursor AND doc.deletion == null"
        } else {
            "FILTER doc.deletion == null"
        };

        // LIMIT in AQL does not support bind parameters — inline the literal.
        // Safe: limit is a u32, no injection possible.
        let limit_clause = limit.map(|l| format!("LIMIT {}", l + 1)).unwrap_or_default();

        let query = format!(
            "FOR doc IN @@col {} SORT doc._key ASC {} {}",
            cursor_filter, limit_clause, return_clause
        );

        let mut docs: Vec<Value> = self.aql(&query, vars).await?;

        // Determine pagination state
        let has_more = match limit {
            Some(l) => docs.len() > l as usize,
            None => false,
        };

        if has_more {
            docs.pop(); // Remove the extra sentinel document
        }

        let next_cursor = if has_more {
            docs.last()
                .and_then(|d| d.get("_key"))
                .and_then(|v| v.as_str())
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

    /// List documents with ACL filtering pushed into AQL.
    /// For global (non-scoped) resources.
    /// `principals`: pre-resolved user principals (user ID + transitive groups).
    /// `required_perm`: bitmask of required permission bits.
    /// `super_bypass`: if true, skip ACL check entirely (user is godmode or has specific permission for this operation only).
    pub async fn generic_list_acl(
        &self,
        collection: &str,
        principals: &[String],
        required_perm: u8,
        super_bypass: bool,
        fields: Option<&[&str]>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<PaginatedResult> {
        let return_clause = match fields {
            Some(f) => {
                let quoted: Vec<String> = f.iter().map(|s| format!("\"{}\"", s)).collect();
                format!("RETURN KEEP(doc, {})", quoted.join(", "))
            }
            None => "RETURN doc".to_string(),
        };

        let mut vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("principals", serde_json::to_value(principals)?),
            ("required_perm", json!(required_perm)),
            ("super_bypass", Value::Bool(super_bypass)),
        ]);

        let cursor_filter = if let Some(c) = cursor {
            vars.insert("cursor", Value::String(c.to_string()));
            "FILTER doc._key > @cursor"
        } else {
            ""
        };

        let limit_clause = limit.map(|l| format!("LIMIT {}", l + 1)).unwrap_or_default();

        let query = format!(
            r#"
            FOR doc IN @@col
                FILTER doc.deletion == null
                {cursor_filter}

                LET acl_pass = @super_bypass OR (
                    LENGTH(doc.acl.list || []) == 0 OR
                    LENGTH(
                        FOR entry IN (doc.acl.list || [])
                            FILTER BIT_AND(entry.permissions, @required_perm) == @required_perm
                            FILTER LENGTH(INTERSECTION(entry.principals, @principals)) > 0
                            LIMIT 1
                            RETURN 1
                    ) > 0
                )
                FILTER acl_pass

                SORT doc._key ASC
                {limit_clause}
                {return_clause}
            "#
        );

        let mut docs: Vec<Value> = self.aql(&query, vars).await?;

        let has_more = match limit {
            Some(l) => docs.len() > l as usize,
            None => false,
        };
        if has_more {
            docs.pop();
        }

        let next_cursor = if has_more {
            docs.last()
                .and_then(|d| d.get("_key"))
                .and_then(|v| v.as_str())
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

    /// Search documents by `_key` prefix with ACL filtering pushed into AQL.
    /// Always returns at most 15 results. Uses brief projection fields if provided.
    pub async fn generic_search_acl(
        &self,
        collection: &str,
        principals: &[String],
        required_perm: u8,
        super_bypass: bool,
        fields: Option<&[&str]>,
        startwith: &str,
    ) -> Result<Vec<Value>> {
        let return_clause = match fields {
            Some(f) => {
                let quoted: Vec<String> = f.iter().map(|s| format!("\"{}\"", s)).collect();
                format!("RETURN KEEP(doc, {})", quoted.join(", "))
            }
            None => "RETURN doc".to_string(),
        };

        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("principals", serde_json::to_value(principals)?),
            ("required_perm", json!(required_perm)),
            ("super_bypass", Value::Bool(super_bypass)),
            ("startwith", Value::String(startwith.to_string())),
        ]);

        let query = format!(
            r#"
            FOR doc IN @@col
                FILTER doc.deletion == null
                FILTER STARTS_WITH(doc._key, @startwith)

                LET acl_pass = @super_bypass OR (
                    LENGTH(doc.acl.list || []) == 0 OR
                    LENGTH(
                        FOR entry IN (doc.acl.list || [])
                            FILTER BIT_AND(entry.permissions, @required_perm) == @required_perm
                            FILTER LENGTH(INTERSECTION(entry.principals, @principals)) > 0
                            LIMIT 1
                            RETURN 1
                    ) > 0
                )
                FILTER acl_pass

                SORT doc._key ASC
                LIMIT 15
                {return_clause}
            "#
        );

        self.aql(&query, vars).await
    }

    /// List project-scoped documents with hybrid ACL resolution in a single AQL query.
    /// If a document has its own ACL entries, they are used.
    /// Otherwise, falls back to the project's full ACL (all entries, no scope filtering).
    pub async fn generic_list_scoped(
        &self,
        collection: &str,
        project_id: &str,
        principals: &[String],
        required_perm: u8,
        super_bypass: bool,
        fields: Option<&[&str]>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<PaginatedResult> {
        let return_clause = match fields {
            Some(f) => {
                let quoted: Vec<String> = f.iter().map(|s| format!("\"{}\"", s)).collect();
                format!("RETURN KEEP(doc, {})", quoted.join(", "))
            }
            None => "RETURN doc".to_string(),
        };

        let mut vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("project_id", Value::String(project_id.to_string())),
            ("principals", serde_json::to_value(principals)?),
            ("required_perm", json!(required_perm)),
            ("super_bypass", Value::Bool(super_bypass)),
        ]);

        let cursor_filter = if let Some(c) = cursor {
            vars.insert("cursor", Value::String(c.to_string()));
            "FILTER doc._key > @cursor"
        } else {
            ""
        };

        let limit_clause = limit.map(|l| format!("LIMIT {}", l + 1)).unwrap_or_default();

        let query = format!(
            r#"
            LET project_doc = DOCUMENT("projects", @project_id)
            LET project_acl = (project_doc != null AND project_doc.deletion == null)
                ? (project_doc.acl.list || [])
                : []

            FOR doc IN @@col
                FILTER doc.project == @project_id
                FILTER doc.deletion == null
                {cursor_filter}

                LET effective_acl = LENGTH(doc.acl.list || []) > 0
                    ? (doc.acl.list || [])
                    : project_acl

                LET acl_pass = @super_bypass OR (
                    LENGTH(
                        FOR entry IN effective_acl
                            FILTER BIT_AND(entry.permissions, @required_perm) == @required_perm
                            FILTER LENGTH(INTERSECTION(entry.principals, @principals)) > 0
                            LIMIT 1
                            RETURN 1
                    ) > 0
                )
                FILTER acl_pass

                SORT doc._key ASC
                {limit_clause}
                {return_clause}
            "#
        );

        let mut docs: Vec<Value> = self.aql(&query, vars).await?;

        let has_more = match limit {
            Some(l) => docs.len() > l as usize,
            None => false,
        };
        if has_more {
            docs.pop();
        }

        let next_cursor = if has_more {
            docs.last()
                .and_then(|d| d.get("_key"))
                .and_then(|v| v.as_str())
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

    /// Fetch a single project-scoped document, validating project membership.
    pub async fn generic_get_scoped(
        &self,
        collection: &str,
        project_id: &str,
        key: &str,
    ) -> Result<Option<Value>> {
        let query = r#"
            LET doc = DOCUMENT(@@col, @key)
            FILTER doc != null AND doc.deletion == null AND doc.project == @project_id
            RETURN doc
        "#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("key", Value::String(key.to_string())),
            ("project_id", Value::String(project_id.to_string())),
        ]);
        let result: Vec<Value> = self.aql(query, vars).await?;
        Ok(result.into_iter().next())
    }

    pub async fn generic_get(&self, collection: &str, key: &str) -> Result<Option<Value>> {
        let query = r#"
            LET doc = DOCUMENT(@@col, @key)
            FILTER doc != null AND doc.deletion == null
            RETURN doc
        "#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("key", Value::String(key.to_string())),
        ]);
        let result: Vec<Value> = self.aql(query, vars).await?;
        Ok(result.into_iter().next())
    }

    pub async fn generic_create(&self, collection: &str, doc: Value) -> Result<()> {
        let query = r#"INSERT @doc INTO @@col"#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("doc", doc),
        ]);
        self.aql::<Value>(query, vars).await?;
        Ok(())
    }

    pub async fn generic_upsert(&self, collection: &str, key: &str, doc: Value) -> Result<()> {
        let query = r#"
            UPSERT { _key: @key }
            INSERT @doc
            UPDATE @doc
            IN @@col
        "#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("key", Value::String(key.to_string())),
            ("doc", doc),
        ]);

        // Same write-write conflict hazard as grant_permission: concurrent upserts
        // on the same key race at the ArangoDB level. Retry transparently.
        super::upsert_with_retry(|| {
            let vars = vars.clone();
            async move {
                self.aql::<Value>(query, vars).await
                    .map(|_| ())
            }
        })
        .await
    }

    pub async fn generic_update(&self, collection: &str, key: &str, doc: Value) -> Result<()> {
        let query = r#"
            LET existing = DOCUMENT(@@col, @key)
            FILTER existing != null
            REPLACE existing WITH @doc IN @@col
            RETURN NEW
        "#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("key", Value::String(key.to_string())),
            ("doc", doc),
        ]);
        let result: Vec<Value> = self.aql(query, vars).await?;
        if result.is_empty() {
            return Err(anyhow!("document not found: {}/{}", collection, key));
        }
        Ok(())
    }

    pub async fn generic_delete(&self, collection: &str, key: &str) -> Result<()> {
        let query = r#"
            LET existing = DOCUMENT(@@col, @key)
            FILTER existing != null
            REMOVE existing IN @@col
            RETURN OLD
        "#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("key", Value::String(key.to_string())),
        ]);
        let result: Vec<Value> = self.aql(query, vars).await?;
        if result.is_empty() {
            return Err(anyhow!("document not found: {}/{}", collection, key));
        }
        Ok(())
    }
}
