use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::util_models::PrincipalId;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventKind {
    Error, // error severity depends on EventPriority
    Background, // background jobs: database compacting, log rotation, whatever
    EntityManagement, // often used with "Lifecycle" priority: global recources being created or destroyed, integrations, etc.
    Server, // logs every time new app instance is launched or other server-related stuff,
    Stats, // regular statistics posted by background jobs
    Database, // log every table creation and other pure db-related stuff (not errors)
    ObjectStorage, // object storage logs (not errors), all upload operations should be logged, also connection or reachability probes

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventPriority {
    Expected, // Use only for events like scheduled maintenance, backups, background actions, etc
    Important, // Use for events that should be highlighted and reviewed by admins, for issues and errors
    Note, // Use for events that affect system state, config changes
    Minor, // Use for less important debug info
    Lifecycle // Use only for "user created", "project removed" and other global recource lifecycle phases
}

/// Events are stored in a highly-indexed table in ArangoDB
/// with TTL set to CRITICAL_EVENTS_TTL env variable days, 30 by default
/// Events should be used alongside logs when logging everything that should be visible to admins
/// in indexable form, but should not be stored forever
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub node: String, // unique instance identifier as string
    pub moment: DateTime<Utc>,
    pub priority: EventPriority,
    pub kind: EventKind,
    pub principal: Option<PrincipalId>, // should be used when event was caused by some user
    pub affects: Vec<String>, // default empty, list of free form strings of entities/systems affected by this event
    #[serde(rename = "_key")]
    pub uid: String, // long unique event ID made of UUIDv4, use corresponding lib instead of a string, primary key in db
    pub details: Option<serde_json::Value>
}

// create a couple of useful constructors
