use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub role: String,
}

pub fn create_jwt(username: impl AsRef<str>) -> String {
    let expiration = Utc::now() + Duration::hours(1);
    // Create claims object
    let claims = Claims {
        sub: username.as_ref().to_string(),
        exp: expiration.timestamp(),
        role: "user".to_string(),
    };
    // Load secret key from environment variable
    let secret_key = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    // Generate JWT token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .expect("JWT encoding failed");

    token
}

pub async fn verify_jwt(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authentication header from request
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // Remove "Bearer " prefix
    let token = &auth_header[7..];
    // Load secret key
    let secret_key = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    // Decode and validate token (checks expiration time and signature)
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    // Add claims to request extensions so handlers can access them
    request.extensions_mut().insert(token_data.claims);
    Ok(next.run(request).await)
}
