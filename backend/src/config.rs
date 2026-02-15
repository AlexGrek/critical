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
    pub database_user: String,
    pub database_password: String,
    pub client_api_keys: Vec<String>,
    pub management_token: String,
    pub host: String,
    pub port: u16,
    pub root_password: String,
    pub jwt_expiry_days: u64,
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

        let database_user =
            env::var("DB_USER").unwrap_or_else(|_| "root".to_string());

        let database_password =
            env::var("DB_PASSWORD").unwrap_or_else(|_| String::new());

        let client_api_keys = env::var("CLIENT_API_KEYS")
            .unwrap_or_else(|_| String::new())
            .split(':')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let root_password = env::var("ROOT_PASSWORD")
            .unwrap_or_else(|_| "changeme".to_string());

        let jwt_expiry_days = env::var("JWT_EXPIRY_DAYS")
            .unwrap_or_else(|_| "90".to_string())
            .parse::<u64>()?;

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3069".to_string())
            .parse::<u16>()?;

        Ok(Self {
            jwt_secret,
            database_connection_string,
            database_name,
            database_user,
            database_password,
            client_api_keys,
            host,
            port,
            management_token,
            root_password,
            jwt_expiry_days,
        })
    }
}
