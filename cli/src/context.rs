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
    #[allow(dead_code)]
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

pub fn config_path_for(home: &std::path::Path) -> PathBuf {
    home.join(CONFIG_DIR).join(CONFIG_FILE)
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(config_path_for(&home))
}

pub fn load_from(path: &std::path::Path) -> Result<ContextFile> {
    if !path.exists() {
        return Ok(ContextFile::default());
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&contents).with_context(|| "failed to parse context.yaml")
}

pub fn save_to(ctx: &ContextFile, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let yaml = serde_yaml::to_string(ctx)?;
    std::fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load() -> Result<ContextFile> {
    load_from(&config_path()?)
}

pub fn save(ctx: &ContextFile) -> Result<()> {
    save_to(ctx, &config_path()?)
}

#[allow(dead_code)]
pub fn require_current() -> Result<ContextEntry> {
    let ctx = load()?;
    match ctx.current_context() {
        Some(entry) => Ok(entry.clone()),
        None => bail!("no active context. Run `cr1t login` first."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_path(dir: &TempDir) -> PathBuf {
        config_path_for(dir.path())
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let ctx = load_from(&test_path(&dir)).unwrap();
        assert!(ctx.current.is_none());
        assert!(ctx.contexts.is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = test_path(&dir);

        let ctx = ContextFile {
            current: Some("local".to_string()),
            contexts: vec![ContextEntry {
                name: "local".to_string(),
                url: "http://localhost:3742".to_string(),
                token: "tok123".to_string(),
            }],
        };

        save_to(&ctx, &path).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded.current.as_deref(), Some("local"));
        assert_eq!(loaded.contexts.len(), 1);
        assert_eq!(loaded.contexts[0].name, "local");
        assert_eq!(loaded.contexts[0].url, "http://localhost:3742");
        assert_eq!(loaded.contexts[0].token, "tok123");
    }

    #[test]
    fn upsert_updates_existing_entry() {
        let mut ctx = ContextFile {
            current: Some("srv".to_string()),
            contexts: vec![ContextEntry {
                name: "srv".to_string(),
                url: "http://old".to_string(),
                token: "old_tok".to_string(),
            }],
        };

        ctx.upsert(ContextEntry {
            name: "srv".to_string(),
            url: "http://new".to_string(),
            token: "new_tok".to_string(),
        });

        assert_eq!(ctx.contexts.len(), 1);
        assert_eq!(ctx.contexts[0].url, "http://new");
        assert_eq!(ctx.contexts[0].token, "new_tok");
    }

    #[test]
    fn upsert_adds_new_entry() {
        let mut ctx = ContextFile::default();
        assert!(ctx.contexts.is_empty());

        ctx.upsert(ContextEntry {
            name: "new".to_string(),
            url: "http://new".to_string(),
            token: "tok".to_string(),
        });

        assert_eq!(ctx.contexts.len(), 1);
        assert_eq!(ctx.contexts[0].name, "new");
    }

    #[test]
    fn current_context_returns_matching_entry() {
        let ctx = ContextFile {
            current: Some("b".to_string()),
            contexts: vec![
                ContextEntry {
                    name: "a".to_string(),
                    url: "http://a".to_string(),
                    token: "ta".to_string(),
                },
                ContextEntry {
                    name: "b".to_string(),
                    url: "http://b".to_string(),
                    token: "tb".to_string(),
                },
            ],
        };

        let entry = ctx.current_context().unwrap();
        assert_eq!(entry.name, "b");
        assert_eq!(entry.url, "http://b");
    }

    #[test]
    fn current_context_returns_none_when_unset() {
        let ctx = ContextFile::default();
        assert!(ctx.current_context().is_none());
    }

    #[test]
    fn current_context_returns_none_when_name_not_found() {
        let ctx = ContextFile {
            current: Some("missing".to_string()),
            contexts: vec![],
        };
        assert!(ctx.current_context().is_none());
    }
}
