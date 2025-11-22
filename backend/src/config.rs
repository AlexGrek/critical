use std::env;

use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    pub user_login_allowed: bool,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub database_connection_string: String,
    pub database_name: String,
    pub client_api_keys: Vec<String>,
    pub management_token: String,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn runtime_from_env() -> Result<RuntimeConfig, AppError> {
        // Load .env file if it exists
        dotenv().ok();

        let allow_user_reg = env::var("API_ALLOW_USER_REGISTRATION")
            .map(|s| s.to_lowercase().contains("true"))
            .unwrap_or(true);

        Ok(RuntimeConfig {
            user_login_allowed: allow_user_reg,
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenv().ok();

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_jwt_secret_change_in_production".to_string());

        let management_token = env::var("MGMT_TOKEN")
            .unwrap_or_else(|_| "default_mgmt_token_change_in_production".to_string());

        let database_connection_string =
            env::var("DB_CONNECTION_STRING").unwrap_or_else(|_| "./data".to_string());
        
        let database_name =
            env::var("DB_NAME").unwrap_or_else(|_| "unnamed".to_string());

        let client_api_keys = env::var("CLIENT_API_KEYS")
            .unwrap_or_else(|_| String::new())
            .split(':')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3069".to_string())
            .parse::<u16>()?;

        Ok(Self {
            jwt_secret,
            database_connection_string,
            client_api_keys,
            host,
            port,
            management_token,
            database_name
        })
    }
}
