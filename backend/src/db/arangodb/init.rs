//! Startup database initialization: create database, create collections,
//! and open collection handles.

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
    })
}

/// Get-or-create each collection then open handles. Used when the database
/// may be empty (e.g. first run or tests).
pub async fn ensure_and_open_collections(
    db: &Database<ReqwestClient>,
) -> Result<CollectionHandles> {
    ensure_collections(db).await?;
    open_collections(db).await
}
