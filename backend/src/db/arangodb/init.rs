//! Startup database initialization: create database, create collections,
//! indexes, and open collection handles.
//!
//! Index management is implemented via raw HTTP because the `arangors` crate
//! does not expose a native index creation API (it is listed as "not yet
//! implemented" in the crate's roadmap).  We call `POST /_db/{db}/_api/index`
//! directly with `reqwest` and treat both 200 (already exists) and 201
//! (created) as success.

use anyhow::{Result, anyhow};
use arangors::client::reqwest::ReqwestClient;
use arangors::collection::Collection;
use arangors::database::Database;
use arangors::Connection;

/// Vertex collections created at startup.
const VERTEX_COLLECTIONS: &[&str] = &[
    "users",
    "groups",
    "service_accounts",
    "pipeline_accounts",
    "projects",
    "permissions",
    "resource_history",
    "resource_events",
    "unprocessed_images",
    "persistent_files",
];

/// Edge collections created at startup.
const EDGE_COLLECTIONS: &[&str] = &["memberships"];

/// Collection names included in write transactions.
pub const WRITE_COLLECTIONS: &[&str] = &[
    "users",
    "groups",
    "service_accounts",
    "pipeline_accounts",
    "projects",
    "memberships",
    "permissions",
    "resource_history",
    "resource_events",
];

/// Cached collection handles opened from a database.
pub struct CollectionHandles {
    pub users: Collection<ReqwestClient>,
    pub groups: Collection<ReqwestClient>,
    pub service_accounts: Collection<ReqwestClient>,
    pub pipeline_accounts: Collection<ReqwestClient>,
    pub projects: Collection<ReqwestClient>,
    pub memberships: Collection<ReqwestClient>,
    pub permissions: Collection<ReqwestClient>,
    pub resource_history: Collection<ReqwestClient>,
    pub resource_events: Collection<ReqwestClient>,
    pub unprocessed_images: Collection<ReqwestClient>,
    pub persistent_files: Collection<ReqwestClient>,
}

/// Obtain the database, creating it if it does not exist.
pub async fn ensure_database(
    conn: &Connection,
    db_name: &str,
) -> Result<Database<ReqwestClient>> {
    match conn.db(db_name).await {
        Ok(db) => Ok(db),
        Err(_) => {
            let _ = conn.create_database(db_name).await;
            conn.db(db_name).await.map_err(|e| anyhow!(e.to_string()))
        }
    }
}

/// Create all vertex and edge collections if they do not exist. Idempotent.
pub async fn ensure_collections(db: &Database<ReqwestClient>) -> Result<()> {
    for name in VERTEX_COLLECTIONS {
        let _ = db.create_collection(name).await;
    }
    for name in EDGE_COLLECTIONS {
        let _ = db.create_edge_collection(name).await;
    }
    Ok(())
}

/// Open a handle for each collection. Collections must already exist
/// (e.g. after ensure_collections).
pub async fn open_collections(db: &Database<ReqwestClient>) -> Result<CollectionHandles> {
    let users = db.collection("users").await.map_err(|e| anyhow!(e.to_string()))?;
    let groups = db.collection("groups").await.map_err(|e| anyhow!(e.to_string()))?;
    let service_accounts = db
        .collection("service_accounts")
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let pipeline_accounts = db
        .collection("pipeline_accounts")
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let projects = db.collection("projects").await.map_err(|e| anyhow!(e.to_string()))?;
    let memberships = db.collection("memberships").await.map_err(|e| anyhow!(e.to_string()))?;
    let permissions = db.collection("permissions").await.map_err(|e| anyhow!(e.to_string()))?;
    let resource_history = db
        .collection("resource_history")
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let resource_events = db
        .collection("resource_events")
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let unprocessed_images = db
        .collection("unprocessed_images")
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let persistent_files = db
        .collection("persistent_files")
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(CollectionHandles {
        users,
        groups,
        service_accounts,
        pipeline_accounts,
        projects,
        memberships,
        permissions,
        resource_history,
        resource_events,
        unprocessed_images,
        persistent_files,
    })
}

/// Create a single persistent index on `collection` with the given `fields`
/// using the ArangoDB REST API directly.
///
/// `arangors` does not expose an index creation API (marked "not yet
/// implemented" in the crate), so we call `POST /_db/{db}/_api/index` with
/// `reqwest`.  Both HTTP 200 (index already exists) and 201 (just created)
/// are treated as success; any other status is an error.
async fn create_persistent_index(
    base_url: &str,
    db_name: &str,
    user: &str,
    password: &str,
    collection: &str,
    fields: &[&str],
) -> Result<()> {
    let url = format!(
        "{}/_db/{}/_api/index?collection={}",
        base_url.trim_end_matches('/'),
        db_name,
        collection
    );
    let body = serde_json::json!({
        "type": "persistent",
        "fields": fields,
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .basic_auth(user, Some(password))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| anyhow!("index creation HTTP request failed: {}", e))?;

    let status = resp.status().as_u16();
    if status == 200 || status == 201 {
        Ok(())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(anyhow!(
            "failed to create index on {}.{:?}: HTTP {} — {}",
            collection,
            fields,
            status,
            text
        ))
    }
}

/// Ensure persistent indexes exist for all collections that need them.
///
/// **Global collections** — index on `deletion` alone speeds up the soft-delete
/// filter used by every list/get query.
///
/// **Project-scoped collections** — composite index on `["project", "deletion"]`
/// lets ArangoDB satisfy `FILTER doc.project == @id AND doc.deletion == null`
/// without a full collection scan.  Add an entry here whenever a new scoped
/// collection (e.g. `tasks`) is introduced.
pub async fn ensure_indexes(
    base_url: &str,
    db_name: &str,
    user: &str,
    password: &str,
) -> Result<()> {
    // Indexes for global (non-scoped) collections: filter on deletion only.
    for col in &["users", "groups", "projects", "service_accounts", "pipeline_accounts"] {
        create_persistent_index(base_url, db_name, user, password, col, &["deletion"]).await?;
    }

    // Indexes for project-scoped collections: composite filter on project + deletion.
    // Add new scoped collections here as they are introduced.
    // Example (uncomment when tasks collection is added):
    // create_persistent_index(base_url, db_name, user, password, "tasks", &["project", "deletion"]).await?;

    Ok(())
}

/// Get-or-create each collection then open handles. Used when the database
/// may be empty (e.g. first run or tests).
/// Does NOT create indexes — call `ensure_indexes` separately when credentials
/// are available (anonymous connections skip index management).
pub async fn ensure_and_open_collections(
    db: &Database<ReqwestClient>,
) -> Result<CollectionHandles> {
    ensure_collections(db).await?;
    open_collections(db).await
}
