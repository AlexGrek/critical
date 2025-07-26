use gitops_lib::GitopsResourceRoot;
use serde::{Deserialize, Serialize};

use crate::entities::{
    ProjectGitopsSerializable, TicketGitopsSerializable, User, UserGitopsSerializable,
    UserPublicData, UserPublicDataGitopsSerializable,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectState {
    pub total_tickets: isize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStateResponse {
    pub meta: ProjectGitopsSerializable,
    pub state: ProjectState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketState {
    pub comments: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketStateResponse {
    pub meta: TicketGitopsSerializable,
    pub state: TicketState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDashboard {
    pub recent_and_owned_projects: Vec<ProjectGitopsSerializable>,
    pub recent_tickets: Vec<TicketGitopsSerializable>,
    pub me: UserPublicDataGitopsSerializable,
}

impl Default for UserDashboard {
    fn default() -> Self {
        Self {
            recent_and_owned_projects: Default::default(),
            recent_tickets: Default::default(),
            me: UserPublicData::default().into_serializable(),
        }
    }
}
