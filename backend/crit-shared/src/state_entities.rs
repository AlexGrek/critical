use serde::{Deserialize, Serialize};

use crate::entities::ProjectGitopsSerializable;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectState {
    pub total_tickets: isize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStateResponse {
    pub meta: ProjectGitopsSerializable,
    pub state: ProjectState,
}


