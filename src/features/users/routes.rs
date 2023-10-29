use std::sync::Arc;

use axum::{extract::Path, http::StatusCode, routing::get, Extension, Json, Router};

use crate::state::AppState;

use super::{
    models::UserProfile,
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

#[utoipa::path(
    get,
    path = "/user/{id}",
    responses(
        (status = 200, body = UserProfile),
        (status = 404, description = "User not found."),
    ),
    tag = "users",
)]
async fn user_by_id(
    Path(id): Path<i32>,
    Extension(repo): UserRepoExt,
) -> Result<Json<UserProfile>, StatusCode> {
    let user = repo.find_by_id(id).await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[utoipa::path(
    get,
    path = "/user/by-email/{email}",
    responses(
        (status = 200, body = UserProfile),
        (status = 404, description = "User not found."),
    ),
    tag = "users",
)]
async fn user_by_email(
    Path(email): Path<String>,
    Extension(repo): UserRepoExt,
) -> Result<Json<UserProfile>, StatusCode> {
    let user = repo.find_by_email(&email).await;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[utoipa::path(
    get,
    path = "/user/me",
    responses(
        (status = 200, body = UserProfile),
        (status = 401, description = "Unauthorized."),
    ),
    tag = "users",
    security(
        ("api_key" = [])
    )
)]
async fn me(user: UserProfile) -> Json<UserProfile> {
    Json(user)
}
