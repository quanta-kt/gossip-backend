use std::sync::Arc;

use axum::{async_trait, Extension};

use crate::{db::Db, features::users::models::UserProfile};

pub type UserRepoExt = Extension<Arc<UserRepo>>;

pub struct UserRepo {
    pub db: Db,
}

#[async_trait]
pub trait UserRepoImpl {
    async fn find_by_id(&self, id: i32) -> Option<UserProfile>;
    async fn find_by_email(&self, email: &str) -> Option<UserProfile>;
}

#[async_trait]
impl UserRepoImpl for UserRepo {
    async fn find_by_id(&self, id: i32) -> Option<UserProfile> {
        sqlx::query_as!(
            UserProfile,
            "SELECT id, username, bio FROM gossip_user WHERE id = $1",
            id
        )
        .fetch_optional(&self.db)
        .await
        .unwrap()
    }

    async fn find_by_email(&self, email: &str) -> Option<UserProfile> {
        sqlx::query_as!(
            UserProfile,
            "SELECT id, username, bio FROM gossip_user WHERE email = $1",
            email
        )
        .fetch_optional(&self.db)
        .await
        .unwrap()
    }
}
