use std::time::Duration;

use anyhow::{Result, anyhow};
use arangors::Connection;
use arangors::client::reqwest::ReqwestClient;
use arangors::collection::Collection;
use arangors::database::Database;
use arangors::document::Document;
use arangors::transaction::{
    Transaction as ArangoInnerTx, TransactionCollections, TransactionSettings,
};
use serde_json::{Value, json};

use crit_shared::data_models::*;
use crit_shared::util_models::*;

//
// ------------------- PAGINATION --------------------
//

pub struct PaginatedResult {
    pub docs: Vec<Value>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

//
// ------------------- TRANSACTION WRAPPER --------------------
//

pub struct ArangoTx {
    inner: ArangoInnerTx<ReqwestClient>,
}

impl ArangoTx {
    pub fn new(inner: ArangoInnerTx<ReqwestClient>) -> Self {
        Self { inner }
    }

    pub async fn commit(&mut self) -> Result<()> {
        self.inner
            .commit()
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(())
    }

    pub async fn abort(&mut self) -> Result<()> {
        self.inner
            .abort()
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(())
    }
}

mod init;

//
// ------------------- MAIN ARANGO BACKEND --------------------
//

pub struct ArangoDb {
    pub conn: Connection,
    pub db: Database<ReqwestClient>,
    /// Cached collection handles for non-transactional operations
    pub users: Collection<ReqwestClient>,
    pub groups: Collection<ReqwestClient>,
    pub service_accounts: Collection<ReqwestClient>,
    pub pipeline_accounts: Collection<ReqwestClient>,
    pub projects: Collection<ReqwestClient>,
    pub memberships: Collection<ReqwestClient>, // edge collection
    pub permissions: Collection<ReqwestClient>,
    pub resource_history: Collection<ReqwestClient>,
    pub resource_events: Collection<ReqwestClient>,
}

// ---------------------------------------------------------------------------
// Retry helper for UPSERT operations
// ---------------------------------------------------------------------------

/// Runs an async UPSERT-style closure and retries on ArangoDB write-write
/// conflicts (error code 1200).
///
/// ArangoDB's `UPSERT` statement is not atomic: it performs a read followed by
/// a conditional insert or update.  When two concurrent requests land on the
/// same document key the second writer sees the conflict and ArangoDB aborts it
/// with error 1200 rather than blocking.  The operation itself is safe to
/// retry — both callers carry the same intent — so we back off briefly and try
/// again instead of propagating a spurious 500 to the caller.
///
/// Three attempts with exponential backoff (5 ms → 10 ms → give up) are enough
/// to absorb the tight races that occur during parallel test runs while adding
/// negligible latency in production where such collisions are rare.
async fn upsert_with_retry<F, Fut>(mut f: F) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    const MAX_ATTEMPTS: u32 = 3;
    for attempt in 0..MAX_ATTEMPTS {
        match f().await {
            // ArangoDB surfaces conflict 1200 as a plain error string.
            Err(e) if e.to_string().contains("1200") => {
                if attempt + 1 == MAX_ATTEMPTS {
                    return Err(e);
                }
                tokio::time::sleep(Duration::from_millis(5 * (1 << attempt))).await;
            }
            other => return other,
        }
    }
    unreachable!()
}

impl ArangoDb {
    pub async fn connect_basic(url: &str, user: &str, pass: &str, db_name: &str) -> Result<Self> {
        let conn = Connection::establish_basic_auth(url, user, pass)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        let db = init::ensure_database(&conn, db_name).await?;
        let handles = init::ensure_and_open_collections(&db).await?;

        let instance = Self {
            conn,
            db,
            users: handles.users,
            groups: handles.groups,
            service_accounts: handles.service_accounts,
            pipeline_accounts: handles.pipeline_accounts,
            projects: handles.projects,
            memberships: handles.memberships,
            permissions: handles.permissions,
            resource_history: handles.resource_history,
            resource_events: handles.resource_events,
        };

        instance.seed_permissions().await?;
        Ok(instance)
    }

    pub async fn connect_anon(url: &str, db_name: &str) -> Result<Self> {
        let conn = Connection::establish_without_auth(url)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        let db = match conn.db(db_name).await {
            Ok(db) => db,
            Err(_) => {
                println!("Creating database...");
                init::ensure_database(&conn, db_name).await?
            }
        };

        let handles = init::ensure_and_open_collections(&db).await?;

        Ok(Self {
            conn,
            db,
            users: handles.users,
            groups: handles.groups,
            service_accounts: handles.service_accounts,
            pipeline_accounts: handles.pipeline_accounts,
            projects: handles.projects,
            memberships: handles.memberships,
            permissions: handles.permissions,
            resource_history: handles.resource_history,
            resource_events: handles.resource_events,
        })
    }

    /// Connect to ArangoDB (JWT auth) and prepare collection handles.
    /// Assumes the database and collections already exist.
    pub async fn connect_jwt(
        url: &str,
        username: &str,
        password: &str,
        db_name: &str,
    ) -> Result<Self> {
        let conn = Connection::establish_jwt(url, username, password)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        let db = conn.db(db_name).await.map_err(|e| anyhow!(e.to_string()))?;
        let handles = init::open_collections(&db).await?;

        Ok(Self {
            conn,
            db,
            users: handles.users,
            groups: handles.groups,
            service_accounts: handles.service_accounts,
            pipeline_accounts: handles.pipeline_accounts,
            projects: handles.projects,
            memberships: handles.memberships,
            permissions: handles.permissions,
            resource_history: handles.resource_history,
            resource_events: handles.resource_events,
        })
    }

    //
    // ------------------- SEED / BOOTSTRAP --------------------
    //

    /// Ensure all super-permissions exist and that u_root has every one of them.
    async fn seed_permissions(&self) -> Result<()> {
        use super_permissions::*;

        let all = [
            ADM_USER_MANAGER,
            ADM_CONFIG_EDITOR,
            USR_CREATE_GROUPS,
            USR_CREATE_PROJECTS,
        ];

        for perm in all {
            self.grant_permission(perm, "u_root").await?;
        }

        Ok(())
    }

    //
    // ------------------- DATABASE OPERATIONS --------------------
    //

    pub async fn begin_transaction(&self) -> Result<ArangoTx> {
        let collections = TransactionCollections::builder()
            .write(
                init::WRITE_COLLECTIONS
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect::<Vec<_>>(),
            )
            .build();

        let settings = TransactionSettings::builder()
            .collections(collections)
            .wait_for_sync(true)
            .build();

        let tx = self
            .db
            .begin_transaction(settings)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(ArangoTx::new(tx))
    }

    pub async fn create_user(&self, user: User, tx: Option<&mut ArangoTx>) -> Result<()> {
        let doc = Document::new(user);

        if let Some(tr) = tx {
            let col = tr
                .inner
                .collection("users")
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            col.create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        } else {
            self.users
                .create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn create_group(&self, group: Group, tx: Option<&mut ArangoTx>) -> Result<()> {
        let doc = Document::new(group);

        if let Some(tr) = tx {
            let col = tr
                .inner
                .collection("groups")
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            col.create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        } else {
            self.groups
                .create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn add_principal_to_group(
        &self,
        principal_id: &str,
        group_id: &str,
        tx: Option<&mut ArangoTx>,
    ) -> Result<()> {
        let key = format!("{}::{}", principal_id, group_id);
        let from_collection = collection_for_principal(principal_id);
        let from = format!("{}/{}", from_collection, principal_id);
        let to = format!("groups/{}", group_id);
        let body = json!({
            "_key": key,
            "_from": from,
            "_to": to,
            "principal": principal_id,
            "group": group_id,
        });

        if let Some(tr) = tx {
            let col = tr
                .inner
                .collection("memberships")
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            col.create_document(Document::new(body), Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        } else {
            self.memberships
                .create_document(Document::new(body), Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn get_users_list(&self) -> Result<Vec<User>> {
        let query = "FOR u IN users RETURN u";
        let users: Vec<User> = self
            .db
            .aql_str(query)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(users)
    }

    pub async fn get_groups_list(&self) -> Result<Vec<Group>> {
        let query = "FOR g IN groups RETURN g";
        let groups: Vec<Group> = self
            .db
            .aql_str(query)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(groups)
    }

    pub async fn get_users_in_group(&self, group_id: &str) -> Result<Vec<String>> {
        let query = r#"
            FOR m IN memberships
                FILTER m.group == @group
                FILTER LIKE(m.principal, "u_%")
                RETURN m.principal
        "#;

        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);

        let res: Vec<String> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(res)
    }

    pub async fn get_groups_in_group(&self, group_id: &str) -> Result<Vec<String>> {
        let query = r#"
            FOR m IN memberships
                FILTER m.group == @group
                FILTER LIKE(m.principal, "g:%")
                RETURN m.principal
        "#;

        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);

        let res: Vec<String> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(res)
    }

    /// Remove a principal from all groups it belongs to.
    /// Returns the list of group IDs that became empty after removal.
    pub async fn remove_principal_from_all_groups(&self, principal_id: &str) -> Result<Vec<String>> {
        // Step 1: Find all groups this principal belongs to (active memberships only)
        let find_query = r#"
            FOR m IN memberships
                FILTER m.principal == @principal
                FILTER m.deletion == null
                RETURN m.group
        "#;
        let vars = std::collections::HashMap::from([(
            "principal",
            serde_json::Value::String(principal_id.to_string()),
        )]);
        let affected_groups: Vec<String> = self
            .db
            .aql_bind_vars(find_query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        // Step 2: Remove all membership edges for this principal
        let remove_query = r#"
            FOR m IN memberships
                FILTER m.principal == @principal
                REMOVE m IN memberships
        "#;
        let vars = std::collections::HashMap::from([(
            "principal",
            serde_json::Value::String(principal_id.to_string()),
        )]);
        self.db
            .aql_bind_vars::<serde_json::Value>(remove_query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        // Step 3: Check which of the affected groups are now empty
        let mut empty_groups = Vec::new();
        for group_id in &affected_groups {
            let count = self.count_group_members(group_id).await?;
            if count == 0 {
                empty_groups.push(group_id.clone());
            }
        }

        Ok(empty_groups)
    }

    /// Count the number of members in a group.
    pub async fn count_group_members(&self, group_id: &str) -> Result<u64> {
        let query = r#"
            RETURN LENGTH(
                FOR m IN memberships
                    FILTER m.group == @group
                    FILTER m.deletion == null
                    RETURN 1
            )
        "#;
        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);
        let result: Vec<u64> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(result.into_iter().next().unwrap_or(0))
    }

    /// Remove all membership edges where this group is the target (members OF this group).
    pub async fn remove_all_members_of_group(&self, group_id: &str) -> Result<()> {
        let query = r#"
            FOR m IN memberships
                FILTER m.group == @group
                REMOVE m IN memberships
        "#;
        let vars = std::collections::HashMap::from([(
            "group",
            serde_json::Value::String(group_id.to_string()),
        )]);
        self.db
            .aql_bind_vars::<serde_json::Value>(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(())
    }

    pub async fn modify_user(&self, user: User, tx: Option<&mut ArangoTx>) -> Result<()> {
        let key = user.id.clone();
        let doc = Document::new(user);
        if let Some(tr) = tx {
            let col = tr.inner.collection("users").await?;
            col.replace_document(&key, doc, Default::default(), None)
                .await?;
        } else {
            self.users
                .replace_document(&key, doc, Default::default(), None)
                .await?;
        }
        Ok(())
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>> {
        let id = if user_id.starts_with("u_") {
            user_id.to_string()
        } else {
            format!("u_{}", user_id)
        };
        match self.users.document::<User>(&id).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(ref e)) if e.code() == 404 => Ok(None),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub async fn get_group_by_id(&self, group_id: &str) -> Result<Option<Group>> {
        match self.groups.document::<Group>(group_id).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(it)) => {
                let error = it;
                let message = error.message().to_string();
                if error.code() == 1202 {
                    return Ok(None);
                }
                Err(anyhow!(message))
            }
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    //
    // ------------------- PERMISSION OPERATIONS --------------------
    //

    pub async fn has_permission(&self, user_id: &str, permission: &str) -> Result<bool> {
        let query = r#"
            LET perm = DOCUMENT("permissions", @permission)
            FILTER perm != null

            LET user_principals = UNION_DISTINCT(
                [@user],
                (FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
                    RETURN v._key)
            )

            RETURN LENGTH(INTERSECTION(user_principals, perm.principals)) > 0
        "#;

        let vars = std::collections::HashMap::from([
            ("user", serde_json::Value::String(user_id.to_string())),
            (
                "permission",
                serde_json::Value::String(permission.to_string()),
            ),
        ]);

        let result: Vec<bool> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(result.first().copied().unwrap_or(false))
    }

    /// Get all principal IDs for a user: the user's own ID plus all group IDs
    /// reachable through the membership graph (up to 10 levels deep).
    pub async fn get_user_principals(&self, user_id: &str) -> Result<Vec<String>> {
        let query = r#"
            LET user_principals = UNION_DISTINCT(
                [@user],
                (FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
                    RETURN v._key)
            )
            RETURN user_principals
        "#;

        let vars = std::collections::HashMap::from([(
            "user",
            serde_json::Value::String(user_id.to_string()),
        )]);

        let result: Vec<Vec<String>> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(result.into_iter().next().unwrap_or_default())
    }

    pub async fn grant_permission(&self, permission: &str, principal: &str) -> Result<()> {
        let query = r#"
            UPSERT { _key: @permission }
            INSERT { _key: @permission, principals: [@principal] }
            UPDATE { principals: UNION_DISTINCT(OLD.principals, [@principal]) }
            IN permissions
        "#;

        let vars = std::collections::HashMap::from([
            (
                "permission",
                serde_json::Value::String(permission.to_string()),
            ),
            (
                "principal",
                serde_json::Value::String(principal.to_string()),
            ),
        ]);

        // ArangoDB UPSERT is a read-then-write and is not atomic: two concurrent
        // calls on the same permission key produce a write-write conflict (error 1200).
        // This is benign — both writers carry the same intent — so we retry with
        // a short exponential backoff instead of surfacing the error to the caller.
        upsert_with_retry(|| {
            let vars = vars.clone();
            async move {
                self.db
                    .aql_bind_vars::<serde_json::Value>(query, vars)
                    .await
                    .map(|_| ())
                    .map_err(|e| anyhow!(e.to_string()))
            }
        })
        .await
    }

    pub async fn revoke_permission(&self, permission: &str, principal: &str) -> Result<()> {
        let query = r#"
            LET perm = DOCUMENT("permissions", @permission)
            FILTER perm != null
            UPDATE perm WITH {
                principals: REMOVE_VALUE(perm.principals, @principal)
            } IN permissions
        "#;

        let vars = std::collections::HashMap::from([
            (
                "permission",
                serde_json::Value::String(permission.to_string()),
            ),
            (
                "principal",
                serde_json::Value::String(principal.to_string()),
            ),
        ]);

        self.db
            .aql_bind_vars::<serde_json::Value>(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(())
    }

    pub async fn get_permission(&self, permission: &str) -> Result<Option<GlobalPermission>> {
        match self.permissions.document::<GlobalPermission>(permission).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(ref e)) if e.code() == 404 => Ok(None),
            Err(arangors::ClientError::Arango(ref e)) if e.code() == 1202 => Ok(None),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>> {
        let query = r#"
            LET user_principals = UNION_DISTINCT(
                [@user],
                (FOR v IN 1..10 OUTBOUND CONCAT("users/", @user) memberships
                    RETURN v._key)
            )

            FOR perm IN permissions
                FILTER LENGTH(INTERSECTION(user_principals, perm.principals)) > 0
                RETURN perm._key
        "#;

        let vars = std::collections::HashMap::from([(
            "user",
            serde_json::Value::String(user_id.to_string()),
        )]);

        let result: Vec<String> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(result)
    }

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

        let mut docs: Vec<Value> = self
            .db
            .aql_bind_vars(&query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

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
        let result: Vec<Value> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(result.into_iter().next())
    }

    pub async fn generic_create(&self, collection: &str, doc: Value) -> Result<()> {
        let query = r#"INSERT @doc INTO @@col"#;
        let vars = std::collections::HashMap::from([
            ("@col", Value::String(collection.to_string())),
            ("doc", doc),
        ]);
        self.db
            .aql_bind_vars::<Value>(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
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
        upsert_with_retry(|| {
            let vars = vars.clone();
            async move {
                self.db
                    .aql_bind_vars::<Value>(query, vars)
                    .await
                    .map(|_| ())
                    .map_err(|e| anyhow!(e.to_string()))
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
        let result: Vec<Value> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
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
        let result: Vec<Value> = self
            .db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        if result.is_empty() {
            return Err(anyhow!("document not found: {}/{}", collection, key));
        }
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

        let edges: Vec<Value> = self
            .db
            .aql_bind_vars(edge_query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

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
        let result: Vec<Value> = self
            .db
            .aql_bind_vars(update_query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

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
        let counts: Vec<u64> = self
            .db
            .aql_bind_vars(count_query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
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
}

/// Resolve a principal ID prefix to its ArangoDB collection name.
pub fn collection_for_principal(principal_id: &str) -> &'static str {
    if principal_id.starts_with("g_") {
        "groups"
    } else if principal_id.starts_with("sa_") {
        "service_accounts"
    } else if principal_id.starts_with("pa_") {
        "pipeline_accounts"
    } else {
        "users" // u_ prefix or fallback
    }
}
