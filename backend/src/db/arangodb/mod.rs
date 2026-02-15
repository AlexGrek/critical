use anyhow::{Result, anyhow};
use arangors::Connection;
use arangors::client::reqwest::ReqwestClient;
use arangors::collection::Collection;
use arangors::database::Database;
use arangors::document::Document;
use arangors::transaction::{
    Transaction as ArangoInnerTx, TransactionCollections, TransactionSettings,
};
use serde_json::json;

use crit_shared::models::*;

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
    /// Optional cached collection handles for non-transactional operations
    pub users: Collection<ReqwestClient>,
    pub groups: Collection<ReqwestClient>,
    pub memberships: Collection<ReqwestClient>,
    pub permissions: Collection<ReqwestClient>,
}

impl ArangoDb {
    pub async fn connect_basic(url: &str, user: &str, pass: &str, db_name: &str) -> Result<Self> {
        let conn = Connection::establish_basic_auth(url, user, pass)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        // obtain or create database
        let db = match conn.db(db_name).await {
            Ok(db) => db,
            Err(_) => {
                // ignore create error (e.g. race condition with parallel tests)
                let _ = conn.create_database(db_name).await;
                conn.db(db_name).await.map_err(|e| anyhow!(e.to_string()))?
            }
        };

        // obtain or create collections (ignore create errors for race conditions)
        let _ = db.create_collection("users").await;
        let _ = db.create_collection("groups").await;
        let _ = db.create_edge_collection("memberships").await;
        let _ = db.create_collection("permissions").await;

        let users = db.collection("users").await.map_err(|e| anyhow!(e.to_string()))?;
        let groups = db.collection("groups").await.map_err(|e| anyhow!(e.to_string()))?;
        let memberships = db.collection("memberships").await.map_err(|e| anyhow!(e.to_string()))?;
        let permissions = db.collection("permissions").await.map_err(|e| anyhow!(e.to_string()))?;

        let instance = Self {
            conn,
            db,
            users,
            groups,
            memberships,
            permissions,
        };

        instance.seed_permissions().await?;

        Ok(instance)
    }

    pub async fn connect_anon(url: &str, db_name: &str) -> Result<Self> {
        // establish connection anonymously
        let conn = Connection::establish_without_auth(url)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        // obtain database handle
        let db = match conn.db(db_name).await {
            Ok(db) => db,
            Err(_) => {
                println!("Creating database...");
                conn.create_database(db_name)
                    .await
                    .map_err(|e| anyhow!(e.to_string()))?;
                conn.db(db_name).await.map_err(|e| anyhow!(e.to_string()))?
            }
        };

        // obtain or create collections
        let users = match db.collection("users").await {
            Ok(collection) => collection,
            Err(_) => db
                .create_collection("users")
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        let groups = match db.collection("groups").await {
            Ok(collection) => collection,
            Err(_) => db
                .create_collection("groups")
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        let memberships = match db.collection("memberships").await {
            Ok(collection) => collection,
            Err(_) => db
                .create_edge_collection("memberships")
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        let permissions = match db.collection("permissions").await {
            Ok(collection) => collection,
            Err(_) => db
                .create_collection("permissions")
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        Ok(Self {
            conn,
            db,
            users,
            groups,
            memberships,
            permissions,
        })
    }

    /// Connect to ArangoDB (JWT auth) and prepare collection handles.
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

        let users = db
            .collection("users")
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        let groups = db
            .collection("groups")
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        let memberships = db
            .collection("memberships")
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        let permissions = db
            .collection("permissions")
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(Self {
            conn,
            db,
            users,
            groups,
            memberships,
            permissions,
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
            ADM_PROJECT_MANAGER,
            USR_CREATE_PROJECTS,
            ADM_CONFIG_EDITOR,
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
            .write(vec![
                "users".to_string(),
                "groups".to_string(),
                "memberships".to_string(),
                "permissions".to_string(),
            ])
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
        let from_collection = if principal_id.starts_with("g_") {
            "groups"
        } else {
            "users"
        };
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

        self.db
            .aql_bind_vars::<serde_json::Value>(query, vars)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(())
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
}
