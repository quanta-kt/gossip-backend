use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, OAuth2, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::features::auth::routes::login,
        crate::features::auth::routes::register,
        crate::features::auth::routes::verify_email,

        crate::features::users::routes::user_by_id,
        crate::features::users::routes::user_by_email,
        crate::features::users::routes::me,
    ),
    components(schemas(
        crate::features::auth::models::LoginRequest,
        crate::features::auth::models::LoginResponse,
        crate::features::auth::models::RegisterRequest,
        crate::features::auth::models::VerifyEmailRequest,

        crate::features::users::models::User,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth",),
        (name = "users",)
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            )
        }
    }
}
