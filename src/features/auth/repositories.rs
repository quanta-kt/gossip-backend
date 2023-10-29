use std::sync::Arc;

use axum::async_trait;

use crate::{db::Db, features::auth::models::PendingEmailVerification};

use super::models::AuthUser;

pub type AuthRepoExt = Arc<AuthRepo>;

pub struct AuthRepo {
    pub db: Db,
}

#[async_trait]
pub trait AuthRepoImpl {
    async fn find_user_id_password_by_email(&self, email: &str) -> Option<AuthUser>;

    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        name: &str,
        code: &str,
    ) -> Option<i32>;

    async fn is_email_taken(&self, email: &str) -> bool;

    async fn get_pending_verification(&self, email: &str) -> Option<PendingEmailVerification>;

    async fn verify_email(&self, email: &str) -> Option<AuthUser>;
}

#[async_trait]
impl AuthRepoImpl for AuthRepo {
    async fn find_user_id_password_by_email(&self, email: &str) -> Option<AuthUser> {
        sqlx::query_file_as!(AuthUser, "queries/auth/get_user_by_email.sql", email)
            .fetch_optional(&self.db)
            .await
            .unwrap()
    }

    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        name: &str,
        verification_code: &str,
    ) -> Option<i32> {
        sqlx::query_file_scalar!(
            "queries/auth/create_user.sql",
            email,
            password_hash,
            name,
            verification_code,
        )
        .fetch_optional(&self.db)
        .await
        .unwrap()
    }

    async fn is_email_taken(&self, email: &str) -> bool {
        sqlx::query_file_scalar!("queries/auth/is_email_taken.sql", email)
            .fetch_one(&self.db)
            .await
            .unwrap()
            .expect("Query should return a boolean")
    }

    async fn get_pending_verification(&self, email: &str) -> Option<PendingEmailVerification> {
        sqlx::query_file_as!(
            PendingEmailVerification,
            "queries/auth/get_pending_verification.sql",
            email
        )
        .fetch_optional(&self.db)
        .await
        .unwrap()
    }

    async fn verify_email(&self, email: &str) -> Option<AuthUser> {
        sqlx::query_file_as!(AuthUser, "queries/auth/verify_email.sql", email)
            .fetch_optional(&self.db)
            .await
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::features::users::repositories::{UserRepo, UserRepoImpl};

    #[sqlx::test]
    async fn test_create_user(pool: PgPool) {
        let repo = AuthRepo { db: pool.clone() };
        let user_repo = UserRepo { db: pool.clone() };

        let id = repo
            .create_user("a.b@c.com", "abc", "abc", "123")
            .await
            .expect("should return user ID");

        repo.create_user("user1@c.com", "abc", "user1", "123")
            .await
            .unwrap();
        repo.create_user("user2@c.com", "abc", "user2", "123")
            .await
            .unwrap();
        repo.create_user("user3@c.com", "abc", "user3", "123")
            .await
            .unwrap();
        repo.create_user("user4@c.com", "abc", "user4", "123")
            .await
            .unwrap();

        let user = sqlx::query_as!(
            AuthUser,
            r#"
            SELECT id, username, email, password_hash, is_verified
            FROM gossip_user
            WHERE id = $1
            "#r,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("should return user");

        assert_eq!(user.email, "a.b@c.com");
        assert_eq!(user.password_hash, "abc");

        let id = repo
            .create_user("a.b@c.com", "def", "abc", "123")
            .await
            .expect("should return user ID");

        let user = sqlx::query_as!(
            AuthUser,
            r#"
            SELECT id, username, email, password_hash, is_verified
            FROM gossip_user
            WHERE id = $1
            "#r,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("should return user");

        assert_eq!(&id, &user.id);
        assert_eq!(&user.email, "a.b@c.com");
        assert_eq!(&user.password_hash, "def");

        sqlx::query!(
            "UPDATE gossip_user SET is_verified = TRUE WHERE id = $1",
            id
        )
        .execute(&pool)
        .await
        .unwrap();

        let user = user_repo
            .find_by_email("a.b@c.com")
            .await
            .expect("should return user");

        let id = repo.create_user("a.b@c.com", "ghi", "abc", "123").await;

        assert_eq!(id, None);

        let user = sqlx::query_as!(
            AuthUser,
            r#"
            SELECT id, username, email, password_hash, is_verified
            FROM gossip_user
            WHERE id = $1
            "#r,
            user.id
        )
        .fetch_one(&pool)
        .await
        .expect("should return user");

        assert_eq!(user.email, "a.b@c.com");
        assert_eq!(user.password_hash, "def");
    }

    #[sqlx::test]
    async fn test_verify_email_deletes_verification_code(pool: PgPool) {
        let repo = AuthRepo { db: pool.clone() };

        let id = repo
            .create_user("a.b@c.com", "abc", "me", "123456")
            .await
            .expect("should return user ID");

        let verification = repo.get_pending_verification("a.b@c.com").await;
        assert!(verification.is_some());

        let user = repo
            .verify_email("a.b@c.com")
            .await
            .expect("should return user");

        let verification = repo.get_pending_verification("a.b@c.com").await;
        assert!(verification.is_none());

        assert_eq!(user.id, id);
        assert_eq!(user.email, "a.b@c.com");
        assert_eq!(user.username, "me");
        assert_eq!(user.password_hash, "abc");

        let user = sqlx::query_as!(
            AuthUser,
            r#"
            SELECT id, email, username, password_hash, is_verified
            FROM gossip_user WHERE id = $1
            "#r,
            id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(user.is_verified);
    }
}
