use std::time::Duration;

use anyhow::{Result, anyhow};
use arangors::Connection;
use arangors::client::reqwest::ReqwestClient;
use arangors::collection::Collection;
use arangors::database::Database;
use arangors::transaction::{
    Transaction as ArangoInnerTx, TransactionCollections, TransactionSettings,
};
use serde_json::Value;

mod init;
mod entities;
mod permissions;
mod gitops;
mod audit;

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
    pub unprocessed_images: Collection<ReqwestClient>,
    pub persistent_files: Collection<ReqwestClient>,
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

        // arangors has no index management API — create indexes via raw HTTP.
        init::ensure_indexes(url, db_name, user, pass).await?;

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
            unprocessed_images: handles.unprocessed_images,
            persistent_files: handles.persistent_files,
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
            unprocessed_images: handles.unprocessed_images,
            persistent_files: handles.persistent_files,
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
            unprocessed_images: handles.unprocessed_images,
            persistent_files: handles.persistent_files,
        })
    }

    //
    // ------------------- QUERY HELPERS --------------------
    //

    /// Execute an AQL query with bind variables.
    /// Logs the query and bind vars at DEBUG level before every execution,
    /// including during tests (`RUST_LOG=debug` or `RUST_LOG=axum_api=debug`).
    async fn aql<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        vars: std::collections::HashMap<&str, Value>,
    ) -> Result<Vec<T>> {
        log::debug!(
            "[AQL]\n{}\nbind_vars: {}",
            query.trim(),
            serde_json::to_value(&vars)
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "<serialize error>".into()),
        );
        self.db
            .aql_bind_vars(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }

    /// Execute a bare AQL string (no bind variables).
    /// Logs the query at DEBUG level.
    async fn aql_str_query<T: serde::de::DeserializeOwned>(&self, query: &str) -> Result<Vec<T>> {
        log::debug!("[AQL]\n{}", query.trim());
        self.db
            .aql_str(query)
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }

    //
    // ------------------- TRANSACTION --------------------
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
