use anyhow::Result;

use crate::{api, context};

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
