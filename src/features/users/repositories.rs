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
        sqlx::query_file_as!(UserProfile, "queries/users/get_profile_by_id.sql", id)
            .fetch_optional(&self.db)
            .await
            .unwrap()
    }

    async fn find_by_email(&self, email: &str) -> Option<UserProfile> {
        sqlx::query_file_as!(UserProfile, "queries/users/get_profile_by_email.sql", email)
            .fetch_optional(&self.db)
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn test_should_not_fetch_unverified_users_by_id(pool: PgPool) {
        let repo = UserRepo { db: pool.clone() };

        let user_id = sqlx::query_file!(
            "queries/auth/create_user.sql",
            "abc@def.com",
            "def",
            "ghi",
            "jkl"
        )
        .fetch_optional(&pool.clone())
        .await
        .unwrap()
        .unwrap()
        .user_id;

        let user = repo.find_by_id(user_id).await;
        assert!(user.is_none());

        let user = repo.find_by_email("abc@def.com").await;
        assert!(user.is_none());

        let auth_user = sqlx::query_file!("queries/auth/verify_email.sql", "abc@def.com")
            .fetch_one(&pool)
            .await
            .unwrap();

        let user = repo.find_by_id(user_id).await.unwrap();
        assert_eq!(user.username, "ghi");
        assert_eq!(auth_user.email, "abc@def.com")
    }
}
