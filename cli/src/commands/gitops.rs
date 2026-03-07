use anyhow::Result;

use crate::{api, context};
use crate::api::{to_api_kind, to_singular_kind};

pub async fn list_groups() -> Result<()> {
    let ctx = context::require_current()?;
    let response = api::list_groups(&ctx.url, &ctx.token).await?;

    // Extract items from the response
    let items: Vec<_> = response
        .get("items")
        .and_then(|v| v.as_array())
        .map(|a| a.clone())
        .unwrap_or_default();

    if items.is_empty() {
        println!("No groups found.");
        return Ok(());
    }

    println!("Groups:\n");
    for item in items {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                println!("  {} ({})", name, id);
            } else {
                println!("  {}", id);
            }
        }
    }

    Ok(())
}

pub async fn describe_group(id: &str) -> Result<()> {
    let ctx = context::require_current()?;
    let mut response = api::get_group(&ctx.url, &ctx.token, id).await?;

    // Inject kind field
    if let Some(obj) = response.as_object_mut() {
        obj.insert("kind".to_string(), serde_json::json!("group"));
    }

    let yaml = serde_yaml::to_string(&response)?;
    print!("{}", yaml);

    Ok(())
}

pub async fn list_users() -> Result<()> {
    let ctx = context::require_current()?;
    let response = api::list_users(&ctx.url, &ctx.token).await?;

    // Extract items from the response
    let items: Vec<_> = response
        .get("items")
        .and_then(|v| v.as_array())
        .map(|a| a.clone())
        .unwrap_or_default();

    if items.is_empty() {
        println!("No users found.");
        return Ok(());
    }

    println!("Users:\n");
    for item in items {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
            if let Some(personal) = item.get("personal").and_then(|v| v.as_object()) {
                if let Some(name) = personal.get("name").and_then(|v| v.as_str()) {
                    if !name.is_empty() {
                        println!("  {} ({})", name, id);
                    } else {
                        println!("  {}", id);
                    }
                } else {
                    println!("  {}", id);
                }
            } else {
                println!("  {}", id);
            }
        }
    }

    Ok(())
}

pub async fn describe_user(id: &str) -> Result<()> {
    let ctx = context::require_current()?;
    let mut response = api::get_user(&ctx.url, &ctx.token, id).await?;

    // Inject kind field
    if let Some(obj) = response.as_object_mut() {
        obj.insert("kind".to_string(), serde_json::json!("user"));
    }

    let yaml = serde_yaml::to_string(&response)?;
    print!("{}", yaml);

    Ok(())
}

/// Generic list: `cr1t get <kind>`
pub async fn list_resources(kind: &str) -> Result<()> {
    let api_kind = to_api_kind(kind);
    let ctx = context::require_current()?;
    let response = api::list_kind(&ctx.url, &ctx.token, &api_kind).await?;

    let items = response
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if items.is_empty() {
        println!("No {} found.", api_kind);
        return Ok(());
    }

    for item in items {
        let yaml = serde_yaml::to_string(&item)?;
        print!("---\n{}", yaml);
    }

    Ok(())
}

/// Generic get: `cr1t get <kind> <id>`
pub async fn get_resource(kind: &str, id: &str) -> Result<()> {
    let api_kind = to_api_kind(kind);
    let ctx = context::require_current()?;
    let response = api::get_kind(&ctx.url, &ctx.token, &api_kind, id).await?;

    let yaml = serde_yaml::to_string(&response)?;
    print!("{}", yaml);

    Ok(())
}

/// Generic describe: `cr1t describe <kind> <id>` — YAML output with kind field injected
pub async fn describe_resource(kind: &str, id: &str) -> Result<()> {
    let api_kind = to_api_kind(kind);
    let singular = to_singular_kind(&api_kind).to_string();
    let ctx = context::require_current()?;
    let mut response = api::get_kind(&ctx.url, &ctx.token, &api_kind, id).await?;

    if let Some(obj) = response.as_object_mut() {
        obj.insert("kind".to_string(), serde_json::json!(singular));
    }

    let yaml = serde_yaml::to_string(&response)?;
    print!("{}", yaml);

    Ok(())
}
