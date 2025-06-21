//! # Database Configuration
//!
//! Defines the structures for configuring database backends for different resource types.
//! This allows for runtime selection of storage implementations (e.g., Filesystem, Sqlite).

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::any::TypeId;

/// Defines the configuration for a specific database backend.
///
/// This enum can be extended to support other database types like Postgres, Sled, etc.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BackendConfig {
    /// Use the filesystem as a database. Stores resources as YAML files.
    Filesystem {
        /// The root directory where resource files will be stored.
        path: PathBuf,
    },
    /// Use SQLite as a database. (Implementation is a placeholder).
    Sqlite {
        /// The connection string or file path for the SQLite database.
        #[serde(rename = "connectionString")]
        connection_string: String,
    },
}

/// Holds the complete store configuration, mapping resource kinds to their backends.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreConfig {
    /// The default backend to use for any resource type not explicitly configured.
    pub default_backend: Option<BackendConfig>,

    /// A map from a resource's `kind` string to the backend config that should be used for it.
    #[serde(default)]
    pub resource_backends: HashMap<String, BackendConfig>,

    #[serde(default)]
    pub namespaced_resource_backends: HashMap<String, BackendConfig>,

    /// A map from a namespace to a specific backend config. Overrides resource-specific and default backends.
    #[serde(default)]
    pub namespace_backends: HashMap<String, BackendConfig>,
}
