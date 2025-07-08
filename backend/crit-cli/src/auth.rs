use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub url: String,
    pub username: String,
    pub jwt_token: String,
}

pub fn get_auth_file_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".crit").join("auth.yaml")
}

pub fn load_auth_config() -> Result<AuthConfig, Box<dyn std::error::Error>> {
    let auth_path = get_auth_file_path();
    let content = fs::read_to_string(auth_path)?;
    let config: AuthConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn save_auth_config(config: &AuthConfig) -> Result<(), Box<dyn std::error::Error>> {
    let auth_path = get_auth_file_path();

    // Create directory if it doesn't exist
    if let Some(parent) = auth_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_yaml::to_string(config)?;
    fs::write(auth_path, content)?;
    Ok(())
}