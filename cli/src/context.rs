use std::path::PathBuf;

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};

const CONFIG_DIR: &str = ".cr1tical";
const CONFIG_FILE: &str = "context.yaml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextEntry {
    pub name: String,
    pub url: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ContextFile {
    #[serde(default)]
    pub current: Option<String>,
    #[serde(default)]
    pub contexts: Vec<ContextEntry>,
}

impl ContextFile {
    pub fn current_context(&self) -> Option<&ContextEntry> {
        let name = self.current.as_ref()?;
        self.contexts.iter().find(|c| &c.name == name)
    }

    pub fn upsert(&mut self, entry: ContextEntry) {
        if let Some(existing) = self.contexts.iter_mut().find(|c| c.name == entry.name) {
            existing.url = entry.url;
            existing.token = entry.token;
        } else {
            self.contexts.push(entry);
        }
    }
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(CONFIG_DIR).join(CONFIG_FILE))
}

pub fn load() -> Result<ContextFile> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(ContextFile::default());
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&contents).with_context(|| "failed to parse context.yaml")
}

pub fn save(ctx: &ContextFile) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let yaml = serde_yaml::to_string(ctx)?;
    std::fs::write(&path, yaml).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn require_current() -> Result<ContextEntry> {
    let ctx = load()?;
    match ctx.current_context() {
        Some(entry) => Ok(entry.clone()),
        None => bail!("no active context. Run `cr1t login` first."),
    }
}
