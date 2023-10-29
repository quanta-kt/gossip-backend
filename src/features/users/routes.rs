use std::sync::Arc;

use axum::{extract::Path, http::StatusCode, routing::get, Extension, Json, Router};

use crate::{features::auth::models::AuthUser, state::AppState};

use super::{
    models::User,
    repositories::{UserRepo, UserRepoExt, UserRepoImpl},
};

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/:id", get(user_by_id))
        .route("/by-email/:email", get(user_by_email))
        .route("/me", get(me))
        .layer(Extension(Arc::new(UserRepo {
            db: state.db.clone(),
        })))
}

async fn user_by_id(
    Path(id): Path<i32>,
    Extension(repo): UserRepoExt,
) -> Result<Json<User>, StatusCode> {
    let user = repo.find_by_id(id).await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn user_by_email(
    Path(email): Path<String>,
    Extension(repo): UserRepoExt,
) -> Result<Json<User>, StatusCode> {
    let user = repo.find_by_email(&email).await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn me(AuthUser(user): AuthUser) -> Json<User> {
    Json(user)
}
