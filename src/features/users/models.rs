use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema, FromRow)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub bio: String,
}
