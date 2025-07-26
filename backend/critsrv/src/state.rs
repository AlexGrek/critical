use gitops_lib::store::Store;

use crate::{auth::Auth};
use std::{path::PathBuf, sync::Arc};

pub struct AppState {
    // pub db: IssueTrackerDb,
    pub auth: Auth,
    pub admin_file_path: PathBuf,
    pub data_dir_path: PathBuf,
    pub store: Arc<Store>,
}