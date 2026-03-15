use std::env;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use poem::{
    http,
    web::headers::{Authorization, authorization::Bearer},
};
use serde::{Deserialize, Serialize};

/// This is crazy: enums for values in a systems programming language
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// `user_id`
    pub sub: i64,
    /// expiration timestamp
    pub exp: chrono::DateTime<chrono::Utc>,
    /// token type
    pub aud: TokenType,
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

pub fn parse_auth_token(auth: &Authorization<Bearer>) -> Result<Claims, poem::Error> {
    let token = auth.token();

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET has to be defined");

    let result = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    );

    match result {
        Ok(decoded) => Ok(decoded.claims),
        Err(e) => Err(poem::Error::from_string(
            e.to_string(),
            http::StatusCode::UNAUTHORIZED,
        )),
    }
}
