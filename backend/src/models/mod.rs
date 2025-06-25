use serde::{Deserialize, Serialize};

pub mod entities;
pub mod managers;


#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub invite_id: String,
    pub invite_key: String,
    pub uid: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub uid: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
