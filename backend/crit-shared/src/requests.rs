use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub uid: String,
    pub password: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub invite_id: String,
    pub invite_key: String,
    pub uid: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ns {
    pub ns: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdNs {
    pub ns: Option<String>,
    pub id: String,
    limit: Option<isize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}
