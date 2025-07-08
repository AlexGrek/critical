use std::collections::HashMap;

use clap::ArgMatches;
use crit_shared::entities::{
    Project, ProjectGitopsSerializable, ProjectLinks, User, UserGitopsSerializable,
    VisibilityConfig,
};

pub async fn handle_template(matches: &ArgMatches) -> anyhow::Result<()> {
    let resource = matches.get_one::<String>("resource").unwrap();

    match resource.as_str() {
        "User" => println!(
            "{}",
            serde_yaml::to_string(&UserGitopsSerializable::from(create_user()))?
        ),
        "Project" => println!(
            "{}",
            serde_yaml::to_string(&ProjectGitopsSerializable::from(create_project()))?
        ),
        _ => (),
    }

    Ok(())
}

pub fn create_user() -> User {
    return User {
        uid: "".to_string(),
        email: "".to_string(),
        password_hash: Some("".to_string()),
        oauth: None,
        created_at: "".to_string(),
        annotations: HashMap::new(),
        has_admin_status: false,
    };
}

pub fn create_project() -> Project {
    return Project {
        name_id: "".to_string(),
        public_name: "".to_string(),
        owner_uid: "".to_string(),
        admins_uid: vec![],
        visibility: VisibilityConfig {
            public_visible: true,
            public_can_report: false,
            public_can_see_tickets: vec![],
        },
        links: ProjectLinks {
            github: "".to_string(),
        },
        issue_categories: vec![],
    };
}
