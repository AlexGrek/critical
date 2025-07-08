use console::style;
use crit_shared::entities::{Project, ProjectGitopsSerializable, User, UserGitopsSerializable};

pub async fn format_cli_output(text: &str, resource_type: &str) {
    match resource_type {
        "users" => {
            if let Ok(users) = serde_json::from_str::<Vec<UserGitopsSerializable>>(text) {
                println!("{}", style("USERS").bold().underlined());
                println!(
                    "{:<20} {:<30} {:<10} {:<20}",
                    style("UID").bold(),
                    style("EMAIL").bold(),
                    style("ADMIN").bold(),
                    style("CREATED").bold()
                );

                for user in users {
                    println!(
                        "{:<20} {:<30} {:<10} {:<20}",
                        style(&user.uid).yellow(),
                        style(&user.email).cyan(),
                        if user.has_admin_status {
                            style("Yes").green()
                        } else {
                            style("No").red()
                        },
                        style(&user.created_at).dim()
                    );
                }
            } else {
                println!("{}", text);
            }
        }
        "projects" => {
            if let Ok(projects) = serde_json::from_str::<Vec<ProjectGitopsSerializable>>(text) {
                println!("{}", style("PROJECTS").bold().underlined());
                println!(
                    "{:<20} {:<30} {:<20} {:<10}",
                    style("NAME_ID").bold(),
                    style("PUBLIC_NAME").bold(),
                    style("OWNER").bold(),
                    style("VISIBILITY").bold()
                );

                for project in projects {
                    println!(
                        "{:<20} {:<30} {:<20} {:<10}",
                        style(&project.name_id).yellow(),
                        style(&project.public_name).cyan(),
                        style(&project.owner_uid).dim(),
                        if project.visibility.public_visible {
                            style("Public").green()
                        } else {
                            style("Private").red()
                        }
                    );
                }
            } else {
                println!("{}", text);
            }
        }
        "user" => {
            if let Ok(user) = serde_json::from_str::<User>(text) {
                println!(
                    "{} {}",
                    style("USER").bold().underlined(),
                    style(&user.uid).yellow()
                );
                println!("{}: {}", style("Email").bold(), user.email);
                println!(
                    "{}: {}",
                    style("Admin Status").bold(),
                    if user.has_admin_status {
                        style("Yes").green()
                    } else {
                        style("No").red()
                    }
                );
                println!("{}: {}", style("Created At").bold(), user.created_at);
                if !user.annotations.is_empty() {
                    println!("{}: ", style("Annotations").bold());
                    for (key, value) in user.annotations {
                        println!("  {}: {}", style(key).dim(), value);
                    }
                }
            } else {
                println!("{}", text);
            }
        }
        "project" => {
            if let Ok(project) = serde_json::from_str::<Project>(text) {
                println!(
                    "{} {}",
                    style("PROJECT").bold().underlined(),
                    style(&project.name_id).yellow()
                );
                println!("{}: {}", style("Public Name").bold(), project.public_name);
                println!("{}: {}", style("Owner").bold(), project.owner_uid);
                println!(
                    "{}: {}",
                    style("Visibility").bold(),
                    if project.visibility.public_visible {
                        style("Public").green()
                    } else {
                        style("Private").red()
                    }
                );
                println!(
                    "{}: {}",
                    style("Admins").bold(),
                    project.admins_uid.join(", ")
                );
                println!(
                    "{}: {}",
                    style("Categories").bold(),
                    project.issue_categories.join(", ")
                );

                if !project.links.github.is_empty()
                    || !project.links.github.is_empty()
                    || !project.links.github.is_empty()
                {
                    println!("{}: ", style("Links").bold());
                    if let repo = &project.links.github {
                        println!("  {}: {}", style("Repository").dim(), repo);
                    }
                    if let docs = &project.links.github {
                        println!("  {}: {}", style("Documentation").dim(), docs);
                    }
                    if let website = &project.links.github {
                        println!("  {}: {}", style("Website").dim(), website);
                    }
                }
            } else {
                println!("{}", text);
            }
        }
        _ => {
            println!("{}", text);
        }
    }
}
