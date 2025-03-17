use crate::errors;
use crate::errors::AppError;
use crate::settings::Auth;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

#[allow(dead_code)]
#[derive(Deserialize, Serialize)]
struct Claims {
    sub: String,
    aud: String,
    exp: usize,
    iss: String,
    nbf: usize,
    email_verified: String,
}

static VALIDATION: OnceCell<Validation> = OnceCell::const_new();
static DECODING_KEY: OnceCell<DecodingKey> = OnceCell::const_new();

pub async fn authorization_middleware(
    State(config): State<Auth>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let auth_token = req
        .headers()
        .get("Authorization")
        .ok_or(errors::AuthError::TokenNotFound)?
        .to_str()
        .map_err(|_| errors::AuthError::TokenNotFound)?;

    let auth_token = auth_token.trim_start_matches("Bearer ").trim();

    authorize_user(auth_token, &config).await?;
    Ok(next.run(req).await)
}

async fn authorize_user(auth_token: &str, auth_config: &Auth) -> Result<(), AppError> {
    let validation = VALIDATION
        .get_or_init(|| create_validation(auth_config))
        .await;
    let decoding_key = DECODING_KEY
        .get_or_init(|| async { DecodingKey::from_secret(auth_config.secret.as_ref()) })
        .await;

    let claims = decode::<Claims>(auth_token, decoding_key, validation)
        .map_err(|_| errors::AuthError::InvalidToken)?;

    if claims.claims.email_verified.eq_ignore_ascii_case("false") {
        return Err(errors::AuthError::UnverifiedEmail)?;
    }

    Ok(())
}

async fn create_validation(auth_config: &Auth) -> Validation {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[auth_config.audience.clone()]);
    validation.set_issuer(&[auth_config.issuer.clone()]);

    validation.set_required_spec_claims(&["sub", "aud", "iss", "nbf", "exp"]);

    validation
}
