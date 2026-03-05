use std::env;

use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

/// This is crazy: enums for values in a systems programming language
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// `user_id`
    sub: i64,
    /// expiration timestamp
    exp: chrono::DateTime<chrono::Utc>,
    /// token type
    aud: TokenType,
}

pub fn gen_auth_token(user_id: i64, token_type: TokenType, exp_hours: u64) -> String {
    let refresh_expiration = chrono::Utc::now()
        .checked_add_days(chrono::Days::new(exp_hours))
        .expect("Invalid expiration time");

    let claims = Claims {
        sub: user_id,
        exp: refresh_expiration,
        aud: token_type,
    };

    // jwt secret should be defined anyways
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET has to be defined");

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Could not generate token")
}
