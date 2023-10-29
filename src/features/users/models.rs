use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    pub is_verified: bool,
}
