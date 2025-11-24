use anyhow::{Result, anyhow};
use arangors::Connection;
use arangors::client::reqwest::ReqwestClient;
use arangors::collection::Collection;
use arangors::database::Database;
use arangors::document::Document;
use arangors::transaction::{
    Transaction as ArangoInnerTx, TransactionCollections, TransactionSettings,
};
use async_trait::async_trait;
use log::Log;
use serde_json::json;

use crate::db::*;

//
// ------------------- TRANSACTION WRAPPER --------------------
//

// Concrete transaction wrapper that delegates to arangors' Transaction.
pub struct ArangoTx {
    inner: ArangoInnerTx<ReqwestClient>,
}

impl ArangoTx {
    pub fn new(inner: ArangoInnerTx<ReqwestClient>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl Transaction for ArangoTx {
    async fn commit(&mut self) -> Result<()> {
        // arangors Transaction provides `commit().await`
        self.inner
            .commit()
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(())
    }

    async fn abort(&mut self) -> Result<()> {
        self.inner
            .abort()
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
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
}

impl ArangoDb {
    pub async fn connect_basic(url: &str, user: &str, pass: &str, db_name: &str) -> Result<Self> {
        // establish connection using an API Key
        let conn = Connection::establish_basic_auth(url, user, pass)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        // obtain database handle
        let db = conn.db(db_name).await.map_err(|e| anyhow!(e.to_string()))?;

        // obtain collections (ensure these collections exist beforehand)
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

        Ok(Self {
            conn,
            db,
            users,
            groups,
            memberships,
        })
    }
    pub async fn connect_anon(url: &str, db_name: &str) -> Result<Self> {
        // establish connection anonymously
        let conn = Connection::establish_without_auth(url)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        // obtain database handle\
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
        // Create users collection if it doesn't exist
        let users = match db.collection("users").await {
            Ok(collection) => collection,
            Err(_) => db
                .create_collection("users")
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        // Create groups collection if it doesn't exist
        let groups = match db.collection("groups").await {
            Ok(collection) => collection,
            Err(_) => db
                .create_collection("groups")
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        // Create memberships edge collection if it doesn't exist
        let memberships = match db.collection("memberships").await {
            Ok(collection) => collection,
            Err(_) => db
                .create_edge_collection("memberships")
                .await
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        Ok(Self {
            conn,
            db,
            users,
            groups,
            memberships,
        })
    }
    /// Connect to ArangoDB (JWT auth) and prepare collection handles.
    pub async fn connect_jwt(
        url: &str,
        username: &str,
        password: &str,
        db_name: &str,
    ) -> Result<Self> {
        // establish connection
        let conn = Connection::establish_jwt(url, username, password)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        // obtain database handle
        let db = conn.db(db_name).await.map_err(|e| anyhow!(e.to_string()))?;

        // obtain collections (ensure these collections exist beforehand)
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

        Ok(Self {
            conn,
            db,
            users,
            groups,
            memberships,
        })
    }
}

//
// ------------------- DATABASE INTERFACE IMPL --------------------
//

#[async_trait]
impl DatabaseInterface for ArangoDb {
    async fn begin_transaction(&self) -> Result<Option<BoxTransaction>> {
        // Build transaction settings: declare write collections.
        let collections = TransactionCollections::builder()
            .write(vec![
                "users".to_string(),
                "groups".to_string(),
                "memberships".to_string(),
            ])
            .build();

        let settings = TransactionSettings::builder()
            .collections(collections)
            .wait_for_sync(true)
            .build();

        // Begin transaction
        let tx = self
            .db
            .begin_transaction(settings)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(Some(Box::new(ArangoTx::new(tx))))
    }

    async fn create_user(&self, user: User, tx: Option<&mut BoxTransaction>) -> Result<()> {
        // wrap into Document
        let doc = Document::new(user);

        if let Some(tr) = tx {
            // downcast to ArangoTx to use transactional collection
            let ar = tr
                .as_any()
                .downcast_mut::<ArangoTx>()
                .ok_or_else(|| anyhow!("transaction is not ArangoTx"))?;

            // get transactional collection handle and create document
            let col = ar
                .inner
                .collection("users")
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
            col.create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        } else {
            // no transaction: use cached collection
            self.users
                .create_document(doc, Default::default())
                .await
                .map_err(|e| anyhow!(e.to_string()))?;
        }

        Ok(())
    }

    async fn create_group(&self, group: Group, tx: Option<&mut BoxTransaction>) -> Result<()> {
        let doc = Document::new(group);

        if let Some(tr) = tx {
            let ar = tr
                .as_any()
                .downcast_mut::<ArangoTx>()
                .ok_or_else(|| anyhow!("transaction is not ArangoTx"))?;
            let col = ar
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

    async fn add_principal_to_group(
        &self,
        principal_id: &str,
        group_id: &str,
        tx: Option<&mut BoxTransaction>,
    ) -> Result<()> {
        // membership document body
        let key = format!("{}::{}", principal_id, group_id);
        let body = json!({
            "_key": key,
            "principal": principal_id,
            "group": group_id,
        });

        if let Some(tr) = tx {
            let ar = tr
                .as_any()
                .downcast_mut::<ArangoTx>()
                .ok_or_else(|| anyhow!("transaction is not ArangoTx"))?;
            let col = ar
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

    async fn get_users_list(&self) -> Result<Vec<User>> {
        // Use AQL to fetch all user docs
        let query = "FOR u IN users RETURN u";
        let users: Vec<User> = self
            .db
            .aql_str(query)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(users)
    }

    async fn get_groups_list(&self) -> Result<Vec<Group>> {
        let query = "FOR g IN groups RETURN g";
        let groups: Vec<Group> = self
            .db
            .aql_str(query)
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        Ok(groups)
    }

    async fn get_users_in_group(&self, group_id: &str) -> Result<Vec<String>> {
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

    async fn get_groups_in_group(&self, group_id: &str) -> Result<Vec<String>> {
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

    async fn modify_user(&self, user: User, tx: Option<&mut BoxTransaction>) -> Result<()> {
        let key = user.id.clone();
        let doc = Document::new(user);
        if let Some(tr) = tx {
            let ar = tr
                .as_any()
                .downcast_mut::<ArangoTx>()
                .ok_or_else(|| anyhow!("transaction is not ArangoTx"))?;
            let col = ar.inner.collection("users").await?;
            col.replace_document(&key, doc, Default::default(), None)
                .await?;
        } else {
            self.users
                .replace_document(&key, doc, Default::default(), None)
                .await?;
        }
        Ok(())
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<User>> {
        println!("Getting user {} by id", user_id);
        let id = if user_id.starts_with("_") {
            user_id
        } else {
            &format!("u_{}", user_id)
        };
        match self.users.document::<User>(id).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(it)) => {
                let error = it;
                let message = error.message().to_string();
                if error.code() == 404 {
                    let all = self.get_users_list().await?;
                    println!("All users: {:?}", all);
                    println!("User not found, but that's OK");
                    return Ok(None);
                }
                println!(
                    "Something went wrong, {} :: {}",
                    error.code(),
                    error.message()
                );
                Err(anyhow!(message))
            } // document not found
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    async fn get_group_by_id(&self, group_id: &str) -> Result<Option<Group>> {
        match self.groups.document::<Group>(group_id).await {
            Ok(doc) => Ok(Some(doc.document)),
            Err(arangors::ClientError::Arango(it)) => {
                let error = it;
                let message = error.message().to_string();
                if error.code() == 1202 {
                    return Ok(None);
                }
                Err(anyhow!(message))
            } // document not found
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }
}
