use utoipa::ToSchema;

use crate::features::users::models::User;

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct PendingEmailVerification {
    pub user_id: i32,
    pub code: String,
}

#[derive(Debug, serde::Deserialize, Clone, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, ToSchema)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TokenClaims {
    pub id: i32,
    pub exp: usize,
}

pub struct AuthUser(pub User);

#[derive(sqlx::FromRow, Debug, serde::Serialize, serde::Deserialize)]
pub struct UserIdPassword {
    pub id: i32,
    pub password_hash: String,
}
