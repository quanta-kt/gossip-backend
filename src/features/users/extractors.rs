use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};

use crate::{jwt, state::AppState};

use super::models::UserProfile;

#[async_trait]
impl FromRequestParts<Arc<AppState>> for UserProfile {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, (StatusCode, &'static str)> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|auth_header| auth_header.to_str().ok())
            .map(|auth_value| {
                auth_value
                    .trim()
                    .trim_start_matches("Bearer")
                    .trim()
                    .to_owned()
            });

        let token =
            token.ok_or_else(|| (StatusCode::UNAUTHORIZED, "No authorization token provided"))?;

        let claims = jwt::decode(&token, state.config.jwt_secret.as_ref())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?
            .claims;

        let user_id = claims.id;

        let user =
            sqlx::query_file_as!(UserProfile, "queries/users/get_profile_by_id.sql", user_id)
                .fetch_optional(&state.db)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))?;

        let user = user.ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        Ok(user)
    }
}
