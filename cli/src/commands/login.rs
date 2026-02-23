use std::io::{self, IsTerminal, Write};

use anyhow::Result;

use crate::api;
use crate::context::{self, ContextEntry, ContextFile};

pub async fn run(url: Option<String>, user: Option<String>) -> Result<()> {
    let url = match url {
        Some(u) => u,
        None => prompt("Server URL")?,
    };

    let user = match user {
        Some(u) => u,
        None => prompt("Username")?,
    };

    // Use secure TTY prompt when interactive; fall back to stdin for piped/test input.
    let password = if io::stdin().is_terminal() {
        rpassword::prompt_password("Password: ")?
    } else {
        let mut pw = String::new();
        io::stdin().read_line(&mut pw)?;
        pw.trim().to_string()
    };

    eprintln!("Logging in to {} as {}...", &url, &user);

    let resp = api::login(&url, &user, &password).await?;

    let context_name = derive_context_name(&url);

    let mut ctx = context::load()?;
    ctx.upsert(ContextEntry {
        name: context_name.clone(),
        url: url.clone(),
        token: resp.token,
    });
    ctx.current = Some(context_name.clone());
    context::save(&ctx)?;

    eprintln!("Logged in successfully. Context '{}' saved and set as current.", &context_name);
    print_contexts(&ctx);

    Ok(())
}

pub fn run_context(show: bool) -> Result<()> {
    let ctx = context::load()?;

    if show || ctx.contexts.is_empty() {
        if ctx.contexts.is_empty() {
            eprintln!("No contexts configured. Run `cr1t login` to get started.");
        } else {
            print_contexts(&ctx);
        }
        return Ok(());
    }

    print_contexts(&ctx);
    Ok(())
}

pub fn use_context(name: &str) -> Result<()> {
    let mut ctx = context::load()?;

    if !ctx.contexts.iter().any(|c| c.name == name) {
        anyhow::bail!("context '{}' not found", name);
    }

    ctx.current = Some(name.to_string());
    context::save(&ctx)?;
    eprintln!("Switched to context '{}'.", name);
    Ok(())
}

fn prompt(label: &str) -> Result<String> {
    eprint!("{}: ", label);
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        anyhow::bail!("{} cannot be empty", label);
    }
    Ok(trimmed)
}

fn derive_context_name(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .replace(['/', ':'], "-")
}

fn print_contexts(ctx: &ContextFile) {
    for entry in &ctx.contexts {
        let marker = if ctx.current.as_deref() == Some(&entry.name) {
            "*"
        } else {
            " "
        };
        eprintln!("  {} {} ({})", marker, entry.name, entry.url);
    }
}
