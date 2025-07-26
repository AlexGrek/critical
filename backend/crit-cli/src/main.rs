use clap::{Arg, ArgMatches, ColorChoice, Command, ValueEnum};
use console::style;
use crit_shared::requests::{LoginRequest, LoginResponse};
use dialoguer::{Input, Password};
use reqwest::Client;
use std::fs;
use std::path::PathBuf;
use tokio;

use crate::apply::handle_apply;
use crate::auth::{AuthConfig, get_auth_file_path, load_auth_config, save_auth_config};
use crate::cli::format_cli_output;
use crate::template::handle_template;

pub mod apply;
pub mod auth;
pub mod cli;
pub mod template;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "crit";

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    #[clap(name = "json")]
    Json,
    #[clap(name = "yaml")]
    Yaml,
    #[clap(name = "cli")]
    Cli,
}

#[tokio::main]
async fn main() {
    let matches = Command::new(APP_NAME)
        .version(VERSION)
        .about("GitOps-style project management CLI client")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_parser(clap::value_parser!(OutputFormat))
                .default_value("cli")
                .global(true),
        )
        .subcommand(Command::new("version").about("Print version information"))
        .subcommand(Command::new("login").about("Login and store authentication"))
        .subcommand(Command::new("logout").about("Clear authentication"))
        .subcommand(Command::new("status").about("Check authentication status"))
        .subcommand(
            Command::new("apply")
                .about("Apply GitOps resource(s) from a file or stdin")
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .help("Path to YAML file to apply (reads stdin if omitted)")
                        .value_name("FILE")
                        .value_parser(clap::value_parser!(std::path::PathBuf))
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("get").about("Get resources").arg(
                Arg::new("resource")
                    .help("Resource type (users, projects)")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("describe")
                .about("Describe a specific resource")
                .arg(
                    Arg::new("resource")
                        .help("Resource type (user, project)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("name")
                        .help("Resource name/ID")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("template")
                .about("Template for creating a specific resource")
                .arg(
                    Arg::new("resource")
                        .help("Resource type (user, project)")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("delete")
                .about("Delete a resource")
                .arg(
                    Arg::new("resource")
                        .help("Resource type (user, project)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("name")
                        .help("Resource name/ID")
                        .required(true)
                        .index(2),
                ),
        )
        .color(ColorChoice::Auto)
        .get_matches();

    let output_format = matches.get_one::<OutputFormat>("output").unwrap();

    match matches.subcommand() {
        Some(("version", _)) => print_version(),
        Some(("login", _)) => handle_login().await,
        Some(("apply", sub_m)) => {
            let file = sub_m.get_one::<PathBuf>("file").cloned();
            handle_apply_f(file).await
        }
        Some(("logout", _)) => handle_logout().await,
        Some(("status", _)) => handle_status().await,
        Some(("get", sub_matches)) => handle_get(sub_matches, output_format).await,
        Some(("template", sub_matches)) => {
            let _ = handle_template(sub_matches).await;
            ()
        }
        Some(("describe", sub_matches)) => handle_describe(sub_matches, output_format).await,
        Some(("delete", sub_matches)) => handle_delete(sub_matches, output_format).await,
        _ => {
            println!(
                "{} No command specified. Use --help for usage information.",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    }
}

fn print_version() {
    println!("{} {}", style(APP_NAME).bold(), style(VERSION).green());
}

async fn handle_login() {
    println!("{}", style("🔐 Login").bold().cyan());
    println!();

    let url: String = Input::new()
        .with_prompt("Server URL")
        .interact_text()
        .unwrap_or_else(|_| {
            println!("{} Failed to read URL input", style("Error:").red().bold());
            std::process::exit(1);
        });

    let username: String = Input::new()
        .with_prompt("Username")
        .interact_text()
        .unwrap_or_else(|_| {
            println!(
                "{} Failed to read username input",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        });

    let password = Password::new()
        .with_prompt("Password")
        .interact()
        .unwrap_or_else(|_| {
            println!(
                "{} Failed to read password input",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        });

    println!();
    println!("{} Authenticating...", style("🔄").yellow());

    let client = Client::new();
    let login_url = format!("{}/api/v1/login", url.trim_end_matches('/'));

    let login_request = LoginRequest {
        uid: username.clone(),
        password,
    };

    match client.post(&login_url).json(&login_request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<LoginResponse>().await {
                    Ok(login_response) => {
                        let auth_config = AuthConfig {
                            url: url.clone(),
                            username: username.clone(),
                            jwt_token: login_response.token,
                        };

                        if let Err(e) = save_auth_config(&auth_config) {
                            println!(
                                "{} Failed to save auth config: {}",
                                style("Error:").red().bold(),
                                e
                            );
                            std::process::exit(1);
                        }

                        println!(
                            "{} Successfully logged in as {}",
                            style("✓").green().bold(),
                            style(&username).yellow()
                        );
                        println!("  Server: {}", style(&url).yellow());
                    }
                    Err(e) => {
                        println!(
                            "{} Failed to parse login response: {}",
                            style("Error:").red().bold(),
                            e
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                println!(
                    "{} Login failed: {}",
                    style("Error:").red().bold(),
                    response.status()
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!(
                "{} Network error: {}",
                style("Error:").red().bold(),
                e.to_string()
            );
            std::process::exit(1);
        }
    }
}

async fn handle_logout() {
    match fs::remove_file(get_auth_file_path()) {
        Ok(_) => println!("{} Successfully logged out", style("✓").green().bold()),
        Err(_) => println!("{} No active session found", style("⚠").yellow()),
    }
}

async fn handle_status() {
    match load_auth_config() {
        Ok(config) => {
            println!("{} Authenticated", style("✓").green().bold());
            println!("  Server: {}", style(&config.url).yellow());
            println!("  Username: {}", style(&config.username).yellow());
        }
        Err(_) => {
            println!(
                "{} Not authenticated. Use 'crit login' to authenticate.",
                style("⚠").yellow()
            );
        }
    }
}

async fn handle_apply_f(file: Option<PathBuf>) {
    let auth_config = match load_auth_config() {
        Ok(config) => config,
        Err(_) => {
            println!(
                "{} Not authenticated. Use 'crit login' first.",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };
    let result = handle_apply(apply::ApplyArgs {
        file: file,
        url: auth_config.url,
        jwt: auth_config.jwt_token,
    })
    .await;
    match result {
        Err(e) => {
            println!("{} Error. {}", style("⚠").yellow(), e);
            std::process::exit(1);
        }
        _ => (),
    }
}

async fn handle_get(matches: &ArgMatches, output_format: &OutputFormat) {
    let resource = matches.get_one::<String>("resource").unwrap();

    let auth_config = match load_auth_config() {
        Ok(config) => config,
        Err(_) => {
            println!(
                "{} Not authenticated. Use 'crit login' first.",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    let client = Client::new();
    let url = format!(
        "{}/api/v1/ops/list/{}",
        auth_config.url.trim_end_matches('/'),
        resource
    );

    match client
        .get(&url)
        .bearer_auth(&auth_config.jwt_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await.unwrap_or_default();
                output_response(&text, output_format, resource).await;
            } else {
                println!(
                    "{} Request failed: {} at {}",
                    style("Error:").red().bold(),
                    response.status(),
                    style(response.url().as_str()).italic()
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("{} Network error: {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
}

async fn handle_describe(matches: &ArgMatches, output_format: &OutputFormat) {
    let resource = matches.get_one::<String>("resource").unwrap();
    let name = matches.get_one::<String>("name").unwrap();

    let auth_config = match load_auth_config() {
        Ok(config) => config,
        Err(_) => {
            println!(
                "{} Not authenticated. Use 'crit login' first.",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    let client = Client::new();
    let url = format!(
        "{}/api/v1/ops/get/{}/{}",
        auth_config.url.trim_end_matches('/'),
        resource,
        name
    );

    match client
        .get(&url)
        .bearer_auth(&auth_config.jwt_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await.unwrap_or_default();
                output_response(&text, output_format, resource).await;
            } else {
                println!(
                    "{} Request failed: {}",
                    style("Error:").red().bold(),
                    response.status()
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("{} Network error: {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
}

async fn handle_delete(matches: &ArgMatches, output_format: &OutputFormat) {
    let resource = matches.get_one::<String>("resource").unwrap();
    let name = matches.get_one::<String>("name").unwrap();

    let auth_config = match load_auth_config() {
        Ok(config) => config,
        Err(_) => {
            println!(
                "{} Not authenticated. Use 'crit login' first.",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    let client = Client::new();
    let url = format!(
        "{}/api/v1/ops/delete/{}/{}",
        auth_config.url.trim_end_matches('/'),
        resource,
        name
    );

    match client
        .delete(&url)
        .bearer_auth(&auth_config.jwt_token)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                println!(
                    "{} {} {} deleted successfully",
                    style("✓").green().bold(),
                    style(resource).yellow(),
                    style(name).yellow()
                );
            } else {
                println!(
                    "{} Delete failed: {}",
                    style("Error:").red().bold(),
                    response.status()
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("{} Network error: {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
}

async fn output_response(text: &str, format: &OutputFormat, resource_type: &str) {
    match format {
        OutputFormat::Json => {
            println!("{}", text);
        }
        OutputFormat::Yaml => {
            // Try to parse as JSON first, then convert to YAML
            match serde_json::from_str::<serde_json::Value>(text) {
                Ok(json_value) => match serde_yaml::to_string(&json_value) {
                    Ok(yaml_output) => println!("{}", yaml_output),
                    Err(_) => println!("{}", text),
                },
                Err(_) => println!("{}", text),
            }
        }
        OutputFormat::Cli => {
            format_cli_output(text, resource_type).await;
        }
    }
}
