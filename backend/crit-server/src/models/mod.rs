use serde::{Deserialize, Serialize};

pub mod entities;
pub mod managers;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
