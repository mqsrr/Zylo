use crate::errors::AppError;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use crate::errors;
use crate::settings::Auth;

#[allow(dead_code)]
#[derive(Deserialize, Serialize)]
struct Claims {
    aud: String,
    exp: usize,
    iss: String,
    nbf: usize,
}

static VALIDATION: OnceCell<Validation> = OnceCell::const_new();
static DECODING_KEY: OnceCell<DecodingKey> = OnceCell::const_new();

pub async fn authorization_middleware(State(config): State<Auth>, req: Request, next: Next) -> Result<impl IntoResponse, AppError> {
    let auth_token = req.headers()
        .get("Authorization")
        .ok_or_else(|| errors::AuthError::TokenNotFound)?.to_str()
        .map_err(|_| errors::AuthError::TokenNotFound)?;

    let auth_token = auth_token.trim_start_matches("Bearer ").trim();

    authorize_user(auth_token, &config).await?;
    Ok(next.run(req).await)
}


async fn authorize_user(auth_token: &str, auth_config: &Auth) -> Result<(), AppError> {
    let validation = VALIDATION.get_or_init(|| create_validation(&auth_config)).await;
    let decoding_key = DECODING_KEY.get_or_init(||async {DecodingKey::from_secret(auth_config.secret.as_ref())}).await;

    decode::<Claims>(auth_token, decoding_key, validation).ok().ok_or_else(|| errors::AuthError::InvalidToken)?;
    Ok(())
}

async fn create_validation(auth_config: &Auth) -> Validation {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&vec![auth_config.audience.clone()]);
    validation.set_issuer(&vec![auth_config.issuer.clone()]);

    validation.set_required_spec_claims(&["aud","iss", "nbf", "exp"]);

    validation
}