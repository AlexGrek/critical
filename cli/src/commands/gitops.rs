use anyhow::Result;
use serde_json::Value;

use crate::{api, context, OutputFormat};
use crate::api::{to_api_kind, to_singular_kind};

// ─── field helpers ────────────────────────────────────────────────────────────

fn str_val(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn field(item: &Value, key: &str) -> String {
    item.get(key).map(str_val).unwrap_or_default()
}

fn nested(item: &Value, k1: &str, k2: &str) -> String {
    item.get(k1)
        .and_then(|v| v.get(k2))
        .map(str_val)
        .unwrap_or_default()
}

fn truncate(s: String, max: usize) -> String {
    if s.chars().count() > max {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", t)
    } else {
        s
    }
}

// ─── per-kind column definitions ─────────────────────────────────────────────

fn headers_for(kind: &str) -> &'static [&'static str] {
    match kind {
        "groups" | "projects" => &["NAME", "ID", "DESCRIPTION"],
        "users" => &["NAME", "ID", "JOB_TITLE"],
        "memberships" => &["ID", "PRINCIPAL", "GROUP"],
        "ticketgroups" => &["ID", "NAME", "TYPES", "PROJECT"],
        _ => &["ID", "NAME"],
    }
}

fn extract_row(kind: &str, item: &Value) -> Vec<String> {
    match kind {
        "groups" | "projects" => vec![
            field(item, "name"),
            field(item, "id"),
            truncate(field(item, "description"), 60),
        ],
        "users" => vec![
            nested(item, "personal", "name"),
            field(item, "id"),
            truncate(nested(item, "personal", "job_title"), 40),
        ],
        "memberships" => vec![
            field(item, "id"),
            field(item, "principal"),
            field(item, "group"),
        ],
        "ticketgroups" => {
            let type_count = item
                .get("ticket_types")
                .and_then(|v| v.as_array())
                .map(|a| a.len().to_string())
                .unwrap_or_else(|| "-".to_string());
            vec![
                field(item, "id"),
                field(item, "name"),
                type_count,
                field(item, "project"),
            ]
        }
        _ => vec![field(item, "id"), field(item, "name")],
    }
}

// ─── ASCII table renderer ─────────────────────────────────────────────────────

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.chars().count());
            }
        }
    }

    let format_row = |cells: &[String]| -> String {
        cells
            .iter()
            .enumerate()
            .map(|(i, cell)| format!("{:<width$}", cell, width = widths.get(i).copied().unwrap_or(0)))
            .collect::<Vec<_>>()
            .join("   ")
    };

    let header_cells: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
    println!("{}", format_row(&header_cells));

    for row in rows {
        println!("{}", format_row(row));
    }
}

// ─── shared list/get implementation ──────────────────────────────────────────

async fn list_impl(api_kind: &str, fmt: OutputFormat) -> Result<()> {
    let ctx = context::require_current()?;
    let response = api::list_kind(&ctx.url, &ctx.token, api_kind).await?;

    let items = response
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if items.is_empty() {
        println!("No {} found.", api_kind);
        return Ok(());
    }

    match fmt {
        OutputFormat::Table => {
            let headers = headers_for(api_kind);
            let rows: Vec<Vec<String>> = items.iter().map(|item| extract_row(api_kind, item)).collect();
            print_table(headers, &rows);
        }
        OutputFormat::Yaml => {
            for item in &items {
                let yaml = serde_yaml::to_string(item)?;
                print!("---\n{}", yaml);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "items": items }))?);
        }
    }

    Ok(())
}

async fn get_impl(api_kind: &str, id: &str, fmt: OutputFormat) -> Result<()> {
    let ctx = context::require_current()?;
    let response = api::get_kind(&ctx.url, &ctx.token, api_kind, id).await?;

    match fmt {
        OutputFormat::Table => {
            let headers = headers_for(api_kind);
            let row = extract_row(api_kind, &response);
            print_table(headers, &[row]);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&response)?;
            print!("{}", yaml);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}

async fn describe_impl(api_kind: &str, id: &str) -> Result<()> {
    let singular = to_singular_kind(api_kind).to_string();
    let ctx = context::require_current()?;
    let mut response = api::get_kind(&ctx.url, &ctx.token, api_kind, id).await?;

    if let Some(obj) = response.as_object_mut() {
        obj.insert("kind".to_string(), serde_json::json!(singular));
    }

    let yaml = serde_yaml::to_string(&response)?;
    print!("{}", yaml);
    Ok(())
}

async fn list_interactive(api_kind: &str) -> Result<()> {
    use dialoguer::{FuzzySelect, theme::ColorfulTheme};

    let ctx = context::require_current()?;
    let data = api::list_kind(&ctx.url, &ctx.token, api_kind).await?;

    let items = data["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    if items.is_empty() {
        println!("No {} found.", api_kind);
        return Ok(());
    }

    // Build display labels: "id   NAME" format
    let labels: Vec<String> = items.iter().map(|item| {
        let id = field(item, "id");
        let name = field(item, "name");
        if name.is_empty() { id } else { format!("{:<30} {}", id, name) }
    }).collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select {}", api_kind))
        .items(&labels)
        .interact_opt()?;

    match selection {
        Some(idx) => {
            let selected_id = field(&items[idx], "id");
            describe_impl(api_kind, &selected_id).await
        }
        None => Ok(()), // user pressed Escape/Ctrl-C
    }
}

// ─── public API ──────────────────────────────────────────────────────────────

pub async fn list_groups(fmt: OutputFormat) -> Result<()> {
    list_impl("groups", fmt).await
}

pub async fn describe_group(id: &str) -> Result<()> {
    describe_impl("groups", id).await
}

pub async fn list_users(fmt: OutputFormat) -> Result<()> {
    list_impl("users", fmt).await
}

pub async fn describe_user(id: &str) -> Result<()> {
    describe_impl("users", id).await
}

/// `cr1t get <kind>` — list all resources of a kind (or interactively if `-i` flag set)
pub async fn list_resources(kind: &str, fmt: OutputFormat, interactive: bool) -> Result<()> {
    let api_kind = to_api_kind(kind);
    if interactive {
        return list_interactive(&api_kind).await;
    }
    list_impl(&api_kind, fmt).await
}

/// `cr1t get <kind> <id>` — get a single resource (table by default)
pub async fn get_resource(kind: &str, id: &str, fmt: OutputFormat) -> Result<()> {
    let api_kind = to_api_kind(kind);
    get_impl(&api_kind, id, fmt).await
}

/// `cr1t describe <kind> <id>` — full YAML with kind field injected
pub async fn describe_resource(kind: &str, id: &str) -> Result<()> {
    let api_kind = to_api_kind(kind);
    describe_impl(&api_kind, id).await
}

// ─── scoped (project-namespaced) helpers ─────────────────────────────────────

async fn list_scoped_impl(project: &str, api_kind: &str, fmt: OutputFormat) -> Result<()> {
    let ctx = context::require_current()?;
    let response = api::list_scoped_kind(&ctx.url, &ctx.token, project, api_kind).await?;

    let items = response
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if items.is_empty() {
        println!("No {} found in project '{}'.", api_kind, project);
        return Ok(());
    }

    match fmt {
        OutputFormat::Table => {
            let headers = headers_for(api_kind);
            let rows: Vec<Vec<String>> = items.iter().map(|item| extract_row(api_kind, item)).collect();
            print_table(headers, &rows);
        }
        OutputFormat::Yaml => {
            for item in &items {
                let yaml = serde_yaml::to_string(item)?;
                print!("---\n{}", yaml);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "items": items }))?);
        }
    }

    Ok(())
}

async fn get_scoped_impl(project: &str, api_kind: &str, id: &str, fmt: OutputFormat) -> Result<()> {
    let ctx = context::require_current()?;
    let response = api::get_scoped_kind(&ctx.url, &ctx.token, project, api_kind, id).await?;

    match fmt {
        OutputFormat::Table => {
            let headers = headers_for(api_kind);
            let row = extract_row(api_kind, &response);
            print_table(headers, &[row]);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&response)?;
            print!("{}", yaml);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}

async fn describe_scoped_impl(project: &str, api_kind: &str, id: &str) -> Result<()> {
    let singular = to_singular_kind(api_kind).to_string();
    let ctx = context::require_current()?;
    let mut response = api::get_scoped_kind(&ctx.url, &ctx.token, project, api_kind, id).await?;

    if let Some(obj) = response.as_object_mut() {
        obj.insert("kind".to_string(), serde_json::json!(singular));
    }

    let yaml = serde_yaml::to_string(&response)?;
    print!("{}", yaml);
    Ok(())
}

async fn list_scoped_interactive(project: &str, api_kind: &str) -> Result<()> {
    use dialoguer::{FuzzySelect, theme::ColorfulTheme};

    let ctx = context::require_current()?;
    let data = api::list_scoped_kind(&ctx.url, &ctx.token, project, api_kind).await?;

    let items = data["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    if items.is_empty() {
        println!("No {} found in project '{}'.", api_kind, project);
        return Ok(());
    }

    let labels: Vec<String> = items.iter().map(|item| {
        let id = field(item, "id");
        let name = field(item, "name");
        if name.is_empty() { id } else { format!("{:<30} {}", id, name) }
    }).collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select {} in project '{}'", api_kind, project))
        .items(&labels)
        .interact_opt()?;

    match selection {
        Some(idx) => {
            let selected_id = field(&items[idx], "id");
            describe_scoped_impl(project, api_kind, &selected_id).await
        }
        None => Ok(()),
    }
}

// ─── public: ticketgroups ─────────────────────────────────────────────────────

pub async fn list_ticket_groups(project: &str, fmt: OutputFormat, interactive: bool) -> Result<()> {
    if interactive {
        return list_scoped_interactive(project, "ticketgroups").await;
    }
    list_scoped_impl(project, "ticketgroups", fmt).await
}

pub async fn get_ticket_group(project: &str, id: &str, fmt: OutputFormat) -> Result<()> {
    get_scoped_impl(project, "ticketgroups", id, fmt).await
}

pub async fn describe_ticket_group(project: &str, id: &str) -> Result<()> {
    describe_scoped_impl(project, "ticketgroups", id).await
}
