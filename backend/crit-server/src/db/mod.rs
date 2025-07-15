use gitops_lib::store::{qstorage::KvStorage};

use crate::exlogging::{LogLevel, log_event};

pub mod index_view;

pub fn initialize_index(storage: &mut dyn KvStorage) {
    let items = vec!["user_to_projects", "user_to_issues"];
    for item in items.into_iter() {
        storage.initialize(item.into()).unwrap_or_else(|e| {
            log_event(LogLevel::Error, e.to_string(), None::<&str>);
            panic!(
                "Failed to initialize index db: {}, item: {}",
                e.to_string(),
                item
            )
        });
    }
}
