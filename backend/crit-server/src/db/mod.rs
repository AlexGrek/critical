use gitops_lib::store::{qstorage::KvStorage};

use crate::{db::indexable_consts::{USER_TO_PROJECTS, USER_TO_TICKETS}, exlogging::{log_event, LogLevel}};

pub mod index_view;
pub mod indexable_consts;

pub fn initialize_index(storage: &mut dyn KvStorage) {
    let items = vec![USER_TO_PROJECTS, USER_TO_TICKETS];
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
