use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, FromRow)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_verified: bool,
}

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
