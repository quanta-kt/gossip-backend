use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{DecodingKey, TokenData, Validation};

use super::features::auth::models::TokenClaims;

pub fn decode(
    token: &str,
    secret: &[u8],
) -> Result<TokenData<TokenClaims>, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
}

pub fn encode(user_id: i32, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    let exp_duration = 60 * 60 * 24 * 30 * 6;

    let exp = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        + exp_duration) as usize;

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &TokenClaims { id: user_id, exp },
        &jsonwebtoken::EncodingKey::from_secret(secret),
    )
}
