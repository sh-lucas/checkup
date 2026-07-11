use std::sync::OnceLock;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use poem::http;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: chrono::DateTime<chrono::Utc>,
    pub aud: TokenType,
}

static SECRET: OnceLock<String> = OnceLock::new();

pub fn init(secret: String) {
    SECRET
        .set(secret)
        .expect("auth::init must be called exactly once at startup");
}

fn secret() -> &'static str {
    SECRET.get().expect("auth::init was not called")
}

pub fn gen_auth_token(user_id: i64, token_type: TokenType, exp_hours: u64) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(exp_hours.cast_signed()))
        .expect("Invalid expiration time");

    let claims = Claims {
        sub: user_id,
        exp: expiration,
        aud: token_type,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret().as_ref()),
    )
    .expect("Could not generate token")
}

pub fn parse_auth_token(token: &str) -> Result<Claims, poem::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret().as_ref()),
        &Validation::default(),
    )
    .map_err(|e| poem::Error::from_string(e.to_string(), http::StatusCode::UNAUTHORIZED))?;

    // `aud` is already required by the Claims type, but we double-check that
    // it deserialized to one of the known variants (defense in depth).
    match data.claims.aud {
        TokenType::Access | TokenType::Refresh => {}
    }

    Ok(data.claims)
}
