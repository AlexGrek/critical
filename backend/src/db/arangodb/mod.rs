use std::sync::Arc;

use anyhow::anyhow;

use arangors::{
    AqlQuery, Connection, Database,
    client::ClientExt,
    collection::{Collection, CollectionType, options::CreateOptions},
    document::{
        Document,
        options::{InsertOptions, RemoveOptions, ReplaceOptions},
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::AppError;
use crate::models::{Group, Project, Ticket};
use crate::{
    db::{BoxFuture, DatabaseInterface, GroupsRepo, ProjectsRepo, TicketsRepo, UsersRepo},
    models::User,
}; // Assuming User is in models, not schema

pub async fn connect_or_create_db_no_auth(
    conn: &Connection,
    db_name: &str,
) -> Result<Database<arangors::client::reqwest::ReqwestClient>, arangors::ClientError> {
    match conn.db(db_name).await {
        Ok(db) => Ok(db),
        Err(_) => {
            conn.create_database(db_name).await?;
            conn.db(db_name).await
        }
    }
}

// ===================================================================
// Error Handling Helpers
// ===================================================================

/// Custom error wrapper for ArangoDB errors
#[derive(Error, Debug)]
pub enum ArangoError {
    #[error("ArangoDB client error: {0}")]
    Arango(#[from] arangors::ClientError), // CORRECTED
    #[error("Collection not found: {0}")]
    CollectionMissing(String),
}

/// Convert our internal ArangoError into the application's AppError
impl From<ArangoError> for AppError {
    fn from(err: ArangoError) -> Self {
        match err {
            ArangoError::Arango(ar_err) => {
                // Handle common ArangoDB API errors
                if let arangors::ClientError::Arango(api_err) = ar_err {
                    let code = api_err.code().into();
                    let msg = api_err.message().to_string();

                    return match code {
                        // 404 Not Found
                        404 => AppError::NotFound(msg),
                        // 409 Conflict (e.g., unique key violation)
                        409 => AppError::Conflict(msg),
                        // 400 Bad Request, 412 Precondition Failed, etc.
                        _ => AppError::Internal(anyhow!(msg)),
                    };
                }
                // Handle other driver-level errors
                AppError::Internal(anyhow!(ar_err.to_string()))
            }
            ArangoError::CollectionMissing(name) => AppError::Internal(anyhow!(
                "Configuration error: Collection '{}' does not exist.",
                name
            )),
        }
    }
}

/// Helper trait to simplify error mapping
trait MapArangoError<T> {
    fn map_err_app_error(self) -> Result<T, AppError>;
}

/// Implementation of the helper trait for any Result from arangors
// CORRECTED: Map from arangors::ClientError
impl<T> MapArangoError<T> for Result<T, arangors::ClientError> {
    fn map_err_app_error(self) -> Result<T, AppError> {
        self.map_err(|e| ArangoError::from(e).into())
    }
}

// ===================================================================
// ArangoDB Storage Document Structs
// ===================================================================
// (These structs remain the same)

/// Represents a User document as stored in the 'principals' collection.
/// `_key` is set to the `user.username`.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ArangoUser {
    #[serde(rename = "_key")]
    key: String,
    #[serde(flatten)]
    user: User,
    doc_type: String, // Always "user"
}

/// Represents a Group document as stored in the 'principals' collection.
/// `_key` is set to the `group.id`.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ArangoGroup {
    #[serde(rename = "_key")]
    key: String,
    #[serde(flatten)]
    group: Group,
    doc_type: String, // Always "group"
}

/// Represents a Project document as stored in the 'projects' collection.
/// `_key` is set to the `project.id`.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ArangoProject {
    #[serde(rename = "_key")]
    key: String,
    #[serde(flatten)]
    project: Project,
}

/// Represents a Ticket document as stored in the 'tickets' collection.
/// `_key` is set to the `ticket.id`.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ArangoTicket {
    #[serde(rename = "_key")]
    key: String,
    #[serde(flatten)]
    ticket: Ticket,
}

// ===================================================================
// Main Database Struct
// ===================================================================

// CORRECTED: Struct is now generic over <C: ClientExt + Send + Sync>
pub struct ArangoDatabase<C: ClientExt + Send + Sync> {
    db: Arc<Database<C>>,
    users_repo: ArangoUsersRepo<C>,
    projects_repo: ArangoProjectsRepo<C>,
    groups_repo: ArangoGroupsRepo<C>,
    tickets_repo: ArangoTicketsRepo<C>,
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> ArangoDatabase<C> {
    /// Creates a new ArangoDatabase instance.
    /// Assumes connection and database name are correct.
    /// Does not create collections; use `initialize` for that.
    pub fn new(db: Database<C>) -> Self {
        let db_arc = Arc::new(db);
        Self {
            db: db_arc.clone(),
            users_repo: ArangoUsersRepo::new(db_arc.clone()),
            projects_repo: ArangoProjectsRepo::new(db_arc.clone()),
            groups_repo: ArangoGroupsRepo::new(db_arc.clone()),
            tickets_repo: ArangoTicketsRepo::new(db_arc.clone()),
        }
    }

    /// Helper function to create all required collections and edges.
    /// Called by the `initialize()` trait method.
    // CORRECTED: Function is generic
    pub async fn setup_schema(db: &Database<C>) -> Result<(), AppError> {
        // Document Collections
        Self::create_collection(db, "principals", CollectionType::Document).await?;
        Self::create_collection(db, "projects", CollectionType::Document).await?;
        Self::create_collection(db, "tickets", CollectionType::Document).await?;

        // Edge Collections
        Self::create_collection(db, "membership", CollectionType::Edge).await?;
        Self::create_collection(db, "parentOf", CollectionType::Edge).await?;
        Self::create_collection(db, "owns", CollectionType::Edge).await?;

        Ok(())
    }

    /// Private helper to create a collection if it doesn't exist.
    async fn create_collection(
        db: &Database<C>,
        name: &str,
        col_type: CollectionType,
    ) -> Result<(), AppError> {
        if db.collection(name).await.is_ok() {
            return Ok(()); // Collection already exists
        }

        let options = CreateOptions::builder()
            .collection_type(col_type)
            .name(name)
            .build();

        db.create_collection_with_options(options, Default::default())
            .await
            .map_err_app_error()?;

        Ok(())
    }
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> DatabaseInterface for ArangoDatabase<C> {
    fn users(&self) -> &dyn UsersRepo {
        &self.users_repo
    }

    fn projects(&self) -> &dyn ProjectsRepo {
        &self.projects_repo
    }

    fn groups(&self) -> &dyn GroupsRepo {
        &self.groups_repo
    }

    fn tickets(&self) -> &dyn TicketsRepo {
        &self.tickets_repo
    }

    // ADDED: initialize method
    fn initialize<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            // Call the static setup_schema helper, passing the db instance
            ArangoDatabase::setup_schema(&self.db).await
        })
    }

    // Transactions are complex and require a different trait design
    // (e.g., passing a transaction handle).
    // For now, we implement them as no-ops like the in-memory version.
    fn begin_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move { Ok(()) })
    }

    fn commit_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_transaction<'a>(&'a self) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move { Ok(()) })
    }
}

// ===================================================================
// Users Repository Implementation
// ===================================================================

// CORRECTED: Struct is generic
pub struct ArangoUsersRepo<C: ClientExt + Send + Sync> {
    db: Arc<Database<C>>,
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> ArangoUsersRepo<C> {
    pub fn new(db: Arc<Database<C>>) -> Self {
        Self { db }
    }
    async fn collection(&self) -> Result<Collection<C>, AppError> {
        self.db.collection("principals").await.map_err_app_error()
    }
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> UsersRepo for ArangoUsersRepo<C> {
    fn get_user<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<User, AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc: Document<ArangoUser> = collection.document(id).await.map_err_app_error()?;

            if doc.document.doc_type != "user" {
                return Err(AppError::NotFound(format!("User {} not found", id)));
            }

            Ok(doc.document.user)
        })
    }

    fn create_user<'a>(&'a self, user: User) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc = ArangoUser {
                key: user.username.clone(),
                user,
                doc_type: "user".to_string(),
            };

            let options = InsertOptions::builder().overwrite(false).build();
            collection
                .create_document(doc, options)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn update_user<'a>(&'a self, id: &'a str, user: User) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            self.get_user(id).await?; // Check type and existence

            let doc = ArangoUser {
                key: id.to_string(),
                user,
                doc_type: "user".to_string(),
            };

            let options = ReplaceOptions::builder().ignore_revs(true).build();
            collection
                .replace_document(id, doc, options, None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn delete_user<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            self.get_user(id).await?; // Check type and existence

            let options = RemoveOptions::builder().silent(true).build();
            collection
                .remove_document::<ArangoUser>(id, options, None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn list_users<'a>(&'a self) -> BoxFuture<'a, Result<Vec<User>, AppError>> {
        Box::pin(async move {
            let query = "FOR doc IN principals FILTER doc.doc_type == 'user' RETURN doc";
            // CORRECTED: Use AqlQuery::builder()
            let aql = AqlQuery::builder().query(query).build();

            let arango_users: Vec<ArangoUser> = self.db.aql_query(aql).await.map_err_app_error()?;

            let users = arango_users.into_iter().map(|au| au.user).collect();
            Ok(users)
        })
    }
}

// ===================================================================
// Groups Repository Implementation
// ===================================================================

// CORRECTED: Struct is generic
pub struct ArangoGroupsRepo<C: ClientExt + Send + Sync> {
    db: Arc<Database<C>>,
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> ArangoGroupsRepo<C> {
    pub fn new(db: Arc<Database<C>>) -> Self {
        Self { db }
    }
    async fn collection(&self) -> Result<Collection<C>, AppError> {
        self.db.collection("principals").await.map_err_app_error()
    }
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> GroupsRepo for ArangoGroupsRepo<C> {
    fn get_group<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Group, AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc: Document<ArangoGroup> = collection.document(id).await.map_err_app_error()?;

            if doc.document.doc_type != "group" {
                return Err(AppError::NotFound(format!("Group {} not found", id)));
            }

            Ok(doc.document.group)
        })
    }

    fn create_group<'a>(&'a self, group: Group) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc = ArangoGroup {
                key: group.gid.to_string(), // Assuming Group has an `id` field
                group,
                doc_type: "group".to_string(),
            };

            let options = InsertOptions::builder().overwrite(false).build();
            collection
                .create_document(doc, options)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn update_group<'a>(
        &'a self,
        id: &'a str,
        group: Group,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            self.get_group(id).await?; // Check type and existence

            let doc = ArangoGroup {
                key: id.to_string(),
                group,
                doc_type: "group".to_string(),
            };
            let options = ReplaceOptions::builder().silent(true).build();
            collection
                .replace_document(id, doc, options, None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn delete_group<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            self.get_group(id).await?; // Check type and existence

            let options = RemoveOptions::builder().silent(true);
            collection
                .remove_document::<ArangoGroup>(id, options.build(), None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn list_groups<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Group>, AppError>> {
        Box::pin(async move {
            let query = "FOR doc IN principals FILTER doc.doc_type == 'group' RETURN doc";
            // CORRECTED: Use AqlQuery::builder()
            let aql = AqlQuery::builder().query(query).build();

            let arango_groups: Vec<ArangoGroup> =
                self.db.aql_query(aql).await.map_err_app_error()?;

            let groups = arango_groups.into_iter().map(|ag| ag.group).collect();
            Ok(groups)
        })
    }
}

// ===================================================================
// Projects Repository Implementation
// ===================================================================

// CORRECTED: Struct is generic
pub struct ArangoProjectsRepo<C: ClientExt + Send + Sync> {
    db: Arc<Database<C>>,
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> ArangoProjectsRepo<C> {
    pub fn new(db: Arc<Database<C>>) -> Self {
        Self { db }
    }
    async fn collection(&self) -> Result<Collection<C>, AppError> {
        self.db.collection("projects").await.map_err_app_error()
    }
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> ProjectsRepo for ArangoProjectsRepo<C> {
    fn get_project<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Project, AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc: Document<ArangoProject> = collection.document(id).await.map_err_app_error()?;
            Ok(doc.document.project)
        })
    }

    fn create_project<'a>(&'a self, project: Project) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc = ArangoProject {
                key: project.id.to_string(),
                project,
            };

            let options = InsertOptions::builder().overwrite(false).build();
            collection
                .create_document(doc, options)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn update_project<'a>(
        &'a self,
        id: &'a str,
        project: Project,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc = ArangoProject {
                key: id.to_string(),
                project,
            };

            let options = ReplaceOptions::builder().silent(true).build();
            collection
                .replace_document(id, doc, options, None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn delete_project<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;

            let options = RemoveOptions::builder().silent(true);
            collection
                .remove_document::<ArangoProject>(id, options.build(), None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn list_projects<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Project>, AppError>> {
        Box::pin(async move {
            let query = "FOR doc IN projects RETURN doc";
            // CORRECTED: Use AqlQuery::builder()
            let aql = AqlQuery::builder().query(query).build();

            let arango_projects: Vec<ArangoProject> =
                self.db.aql_query(aql).await.map_err_app_error()?;

            let projects = arango_projects.into_iter().map(|ap| ap.project).collect();
            Ok(projects)
        })
    }
}

// ===================================================================
// Tickets Repository Implementation
// ===================================================================

// CORRECTED: Struct is generic
pub struct ArangoTicketsRepo<C: ClientExt + Send + Sync> {
    db: Arc<Database<C>>,
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> ArangoTicketsRepo<C> {
    pub fn new(db: Arc<Database<C>>) -> Self {
        Self { db }
    }
    async fn collection(&self) -> Result<Collection<C>, AppError> {
        self.db.collection("tickets").await.map_err_app_error()
    }
}

// CORRECTED: Impl block is generic
impl<C: ClientExt + Send + Sync> TicketsRepo for ArangoTicketsRepo<C> {
    fn get_ticket<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<Ticket, AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc: Document<ArangoTicket> = collection.document(id).await.map_err_app_error()?;
            Ok(doc.document.ticket)
        })
    }

    fn create_ticket<'a>(&'a self, ticket: Ticket) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc = ArangoTicket {
                key: ticket.id.to_string(),
                ticket,
            };

            let options = InsertOptions::builder().overwrite(false);
            collection
                .create_document(doc, options.build())
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn update_ticket<'a>(
        &'a self,
        id: &'a str,
        ticket: Ticket,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;
            let doc = ArangoTicket {
                key: id.to_string(),
                ticket,
            };

            let options = ReplaceOptions::builder().silent(true);
            collection
                .replace_document(id, doc, options.build(), None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn delete_ticket<'a>(&'a self, id: &'a str) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(async move {
            let collection = self.collection().await?;

            let options = RemoveOptions::builder().silent(true).build();
            collection
                .remove_document::<ArangoTicket>(id, options, None)
                .await
                .map_err_app_error()?;
            Ok(())
        })
    }

    fn list_tickets<'a>(&'a self) -> BoxFuture<'a, Result<Vec<Ticket>, AppError>> {
        Box::pin(async move {
            let query = "FOR doc IN tickets RETURN doc";
            // CORRECTED: Use AqlQuery::builder()
            let aql = AqlQuery::builder().query(query).build();

            let arango_tickets: Vec<ArangoTicket> =
                self.db.aql_query(aql).await.map_err_app_error()?;

            let tickets = arango_tickets.into_iter().map(|at| at.ticket).collect();
            Ok(tickets)
        })
    }
}
