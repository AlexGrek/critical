use anyhow::{Context, Result};
use clap::Parser;
use crit_shared::{entities::{
    ProjectGitopsSerializable, ProjectGitopsUpdate, UserGitopsSerializable, UserGitopsUpdate,
}, KindOnly};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::Value;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
pub struct ApplyArgs {
    #[arg(short = 'f', long)]
    pub file: Option<PathBuf>,
    pub url: String,
}

pub async fn handle_apply(args: ApplyArgs) -> Result<()> {
    let input = if let Some(path) = args.file {
        fs::read_to_string(path)?
    } else {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let docs: Vec<&str> = input.split("---").collect();

    let pb = ProgressBar::new(docs.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let client = Client::new();

    for (i, doc) in docs.iter().enumerate() {
        pb.set_message(format!("Parsing doc {}", i + 1));

        let kind_only: Result<KindOnly, _> = serde_yaml::from_str(doc);
        let kind = match kind_only {
            Ok(k) => k.kind.to_lowercase(),
            Err(e) => {
                pb.println(format!(
                    "✘ Skipped document {}: Invalid kind - {}",
                    i + 1,
                    e
                ));
                pb.inc(1);
                continue;
            }
        };

        let json_value: Value = match match_kind_to_type(&kind, doc) {
            Ok(val) => val,
            Err(e) => {
                pb.println(format!(
                    "✘ Document {}: Failed to parse as any known type: {}",
                    i + 1,
                    e
                ));
                pb.inc(1);
                continue;
            }
        };

        let resp = client
            .post("http://localhost:8000/api/v1/ops/upsert")
            .json(&json_value)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                pb.println(format!("✔ Document {} applied successfully", i + 1));
            }
            Ok(r) => {
                let err_text = r.text().await.unwrap_or_default();
                pb.println(format!("✘ Document {} failed: {}", i + 1, err_text));
            }
            Err(e) => {
                pb.println(format!("✘ Network error for document {}: {}", i + 1, e));
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Done");
    Ok(())
}

fn match_kind_to_type(kind: &str, yaml: &str) -> Result<Value> {
    match kind {
        "project" => {
            // Try ProjectGitopsSerializable first
            if let Ok(parsed) = serde_yaml::from_str::<ProjectGitopsSerializable>(yaml) {
                return Ok(serde_json::to_value(parsed)?);
            }
            // Then ProjectGitopsUpdate
            if let Ok(parsed) = serde_yaml::from_str::<ProjectGitopsUpdate>(yaml) {
                return Ok(serde_json::to_value(parsed)?);
            }
            Err(anyhow::anyhow!(
                "Failed to parse as ProjectGitopsSerializable or ProjectGitopsUpdate"
            ))
        }

        "user" => {
            if let Ok(parsed) = serde_yaml::from_str::<UserGitopsSerializable>(yaml) {
                return Ok(serde_json::to_value(parsed)?);
            }
            if let Ok(parsed) = serde_yaml::from_str::<UserGitopsUpdate>(yaml) {
                return Ok(serde_json::to_value(parsed)?);
            }
            Err(anyhow::anyhow!(
                "Failed to parse as UserGitopsSerializable or UserGitopsUpdate"
            ))
        }

        other => Err(anyhow::anyhow!("Unsupported kind: {}", other)),
    }
}

/// Attempts to parse YAML using multiple parsers
fn try_parse_as<T, F>(parsers: &[F], yaml: &str) -> Result<Value>
where
    T: serde::Serialize,
    F: Fn(&str) -> Result<T, serde_yaml::Error>,
{
    for parser in parsers {
        if let Ok(val) = parser(yaml) {
            return Ok(serde_json::to_value(val)?);
        }
    }
    Err(anyhow::anyhow!("All parses failed"))
}
