use std::sync::Arc;

use crit_shared::event_models::{Event, EventKind, EventPriority};
use serde_json::Value;
use uuid::Uuid;

use crate::db::ArangoDb;

pub struct EventLogger {
    db: Arc<ArangoDb>,
    node_id: String,
}

impl EventLogger {
    pub fn new(db: Arc<ArangoDb>, node_id: String) -> Self {
        Self { db, node_id }
    }

    /// Log any event. Errors are swallowed (logged at warn level) to avoid
    /// disrupting normal request handling if the event store is unavailable.
    pub async fn log(
        &self,
        priority: EventPriority,
        kind: EventKind,
        principal: Option<&str>,
        affects: Vec<String>,
        details: Option<Value>,
    ) {
        let event = Event {
            node: self.node_id.clone(),
            moment: chrono::Utc::now(),
            priority,
            kind,
            principal: principal.map(|s| s.to_string()),
            affects,
            uid: Uuid::new_v4().to_string(),
            details,
        };
        if let Err(e) = self.db.write_system_event(event).await {
            log::warn!("[EventLogger] failed to write event: {}", e);
        }
    }

    /// Lifecycle event: entity created/updated/deleted.
    pub async fn entity_lifecycle(
        &self,
        priority: EventPriority,
        principal: &str,
        resource_id: &str,
        action: &str,
        details: Option<Value>,
    ) {
        self.log(
            priority,
            EventKind::EntityManagement,
            Some(principal),
            vec![resource_id.to_string()],
            details.or_else(|| Some(serde_json::json!({ "action": action }))),
        )
        .await;
    }

    /// Server lifecycle event (startup, shutdown, etc.)
    pub async fn server_event(&self, priority: EventPriority, details: Option<Value>) {
        self.log(priority, EventKind::Server, None, vec![], details).await;
    }

    /// Database event (collection created, etc.)
    pub async fn database_event(&self, priority: EventPriority, details: Option<Value>) {
        self.log(priority, EventKind::Database, None, vec![], details).await;
    }

    /// Background job event.
    pub async fn background_event(
        &self,
        priority: EventPriority,
        affects: Vec<String>,
        details: Option<Value>,
    ) {
        self.log(priority, EventKind::Background, None, affects, details).await;
    }

    /// Object storage event.
    pub async fn objectstore_event(
        &self,
        priority: EventPriority,
        principal: Option<&str>,
        affects: Vec<String>,
        details: Option<Value>,
    ) {
        self.log(priority, EventKind::ObjectStorage, principal, affects, details).await;
    }
}
