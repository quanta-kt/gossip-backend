use std::sync::Arc;

use axum::async_trait;

use crate::{db::Db, features::auth::models::PendingEmailVerification};

use super::models::{AuthUser, UserIdPassword};

pub type AuthRepoExt = Arc<AuthRepo>;

pub struct AuthRepo {
    pub db: Db,
}

#[async_trait]
pub trait AuthRepoImpl {
    async fn find_user_id_password_by_email(&self, email: &str) -> Option<UserIdPassword>;

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
    async fn find_user_id_password_by_email(&self, email: &str) -> Option<UserIdPassword> {
        sqlx::query_as!(
            UserIdPassword,
            "SELECT id, password_hash FROM gossip_user WHERE email = $1",
            email
        )
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
        sqlx::query_scalar!(
            r#"
            WITH insert_result AS(
                INSERT INTO gossip_user (email, password_hash, username)
                VALUES ($1, $2, $3)
 
                -- Account creation is idempotent for unverified accounts,
                -- if the email is taken, but the account is not verified, the creation should pass.
                -- when this happens, we update the password and resend the OTP.
                ON CONFLICT (email)
                    DO UPDATE
                    SET
                        password_hash = $2,
                        username = $3
                    WHERE gossip_user.is_verified = FALSE

                RETURNING id
            )

            INSERT INTO pending_email_verification (user_id, code)
            SELECT id, $4
            FROM insert_result
            ON CONFLICT (user_id)
                DO UPDATE
                SET code = $4
            RETURNING user_id 
            "#r,
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
        sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM gossip_user WHERE email = $1 AND is_verified = TRUE)",
            email
        )
        .fetch_one(&self.db)
        .await
        .unwrap()
        .expect("Query should return a boolean")
    }

    async fn get_pending_verification(&self, email: &str) -> Option<PendingEmailVerification> {
        sqlx::query_as!(
            PendingEmailVerification,
            r#"SELECT user_id, code
            FROM pending_email_verification
            JOIN gossip_user ON
                gossip_user.id = pending_email_verification.user_id
            WHERE
                gossip_user.email = $1 AND gossip_user.is_verified = FALSE
            "#r,
            email
        )
        .fetch_optional(&self.db)
        .await
        .unwrap()
    }

    async fn verify_email(&self, email: &str) -> Option<AuthUser> {
        sqlx::query_as!(
            AuthUser,
            r#"
            WITH
                update_result AS (
                    UPDATE gossip_user
                    SET is_verified = TRUE
                    WHERE email = $1
                    RETURNING id
                ),
                _ AS (
                    DELETE FROM pending_email_verification
                    WHERE user_id IN (SELECT id FROM update_result)
                )

            SELECT id, username, email, password_hash, is_verified
            FROM gossip_user
            WHERE id IN (SELECT id FROM update_result)
            "#r,
            email
        )
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
