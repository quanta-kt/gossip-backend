use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{extract::State, http::StatusCode, routing::post, Extension, Json, Router};
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use rand::Rng;

use crate::{
    features::auth::{jwt, repositories::AuthRepoImpl},
    state::AppState,
};

use super::{
    models::{LoginRequest, LoginResponse, RegisterRequest, VerifyEmailRequest},
    repositories::{self, AuthRepoExt},
};

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/verify", post(verify_email))
        .layer(Extension(Arc::new(repositories::AuthRepo {
            db: state.db.clone(),
        })))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Extension(repo): Extension<AuthRepoExt>,
    Json(LoginRequest { email, password }): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let argon2 = Argon2::default();

    let user = repo
        .find_user_id_password_by_email(&email)
        .await
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?;

    let hash =
        PasswordHash::new(&user.password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match argon2.verify_password(password.as_ref(), &hash) {
        Ok(()) => {
            let token = jwt::encode(user.id, state.config.jwt_secret.as_ref())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(LoginResponse { token }))
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn register(
    Extension(repo): Extension<AuthRepoExt>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> StatusCode {
    if repo.is_email_taken(&body.email).await {
        return StatusCode::CONFLICT;
    }

    let config = state.config.clone();

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2.hash_password(body.password.as_ref(), &salt);

    let password_hash = match password_hash {
        Ok(hash) => hash.to_string(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let mut rng = rand::rngs::OsRng;
    let verification_code = rng.gen_range(100000..999999);

    let user_id = repo
        .create_user(
            &body.email,
            &password_hash,
            &body.name,
            verification_code.to_string().as_str(),
        )
        .await;

    let message = MessageBuilder::new()
        .from((config.mail_author, config.mail_email))
        .to(("", body.email.as_str()))
        .subject("Your Gossip verification code")
        .html_body(format!(
            r#"Your verification code is: {}

            <br><br>

            If you didn't request this code, please ignore this email. 
            "#,
            verification_code
        ));

    SmtpClientBuilder::new(config.mail_host.as_str(), config.mail_port)
        .implicit_tls(config.mail_tls)
        .credentials((config.mail_username.as_str(), config.mail_password.as_str()))
        .connect()
        .await
        .unwrap()
        .send(message)
        .await
        .unwrap();

    match user_id {
        Some(_) => StatusCode::CREATED,
        None => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn verify_email(
    Extension(repo): Extension<AuthRepoExt>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let config = state.config.clone();

    let pending_verification = repo
        .get_pending_verification(&body.email)
        .await
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?;

    if pending_verification.code != body.code {
        return Err(StatusCode::UNAUTHORIZED);
    }

    repo.verify_email(&body.email)
        .await
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?;

    return Ok(Json(LoginResponse {
        token: jwt::encode(pending_verification.user_id, config.jwt_secret.as_ref())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    }));
}
