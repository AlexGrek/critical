use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL};
use crit_shared::entities::{ProjectGitopsSerializable, UserGitopsSerializable};

pub async fn format_cli_output(text: &str, resource_type: &str) {
    match resource_type {
        "User" => {
            if let Ok(users) = serde_json::from_str::<Vec<UserGitopsSerializable>>(text) {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["username", "email", "is admin"]);

                for user in users {
                    table.add_row(vec![
                        user.uid,
                        user.email,
                        user.has_admin_status.to_string(),
                    ]);
                }

                println!("{table}");
            } else {
                println!("{}", text);
            }
        }
        "Project" => {
            if let Ok(projects) = serde_json::from_str::<Vec<ProjectGitopsSerializable>>(text) {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["id", "name", "owner", "public"]);

                for project in projects {
                    table.add_row(vec![
                        project.name_id,
                        project.public_name,
                        project.owner_uid,
                        project.visibility.public_visible.to_string(),
                    ]);
                }
                println!("{table}");
            } else {
                println!("{}", text);
            }
        }
        _ => {
            println!("{}", text);
        }
    }
}
