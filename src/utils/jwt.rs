use std::env;

use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyClaims {
    pub user_id: ObjectId,
    aud: String,
    exp: u64,
}

pub fn create_token(user_id: ObjectId) -> String {
    let exp = Utc::now() + Duration::hours(1);
    let my_claims = MyClaims {
        user_id,
        aud: "example.com".to_string(),
        exp: exp.timestamp() as u64,
    };

    let secret = env::var("JWT_SECRET").expect("secret key not found");

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    );

    let access_token = match token {
        Ok(token) => token,
        Err(e) => e.to_string(),
    };

    access_token
}

pub fn decode_token(token: &str) -> Result<TokenData<MyClaims>, DecodeTokenError> {
    let mut validation = Validation::default();
    validation.set_audience(&["example.com"]);

    let secret = env::var("JWT_SECRET").expect("secret key not found");
    let key = &DecodingKey::from_secret(secret.as_ref());

    let token_data = match decode::<MyClaims>(&token, &key, &validation) {
        Ok(token) => token,
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => return Err(DecodeTokenError::InvalidToken),
            ErrorKind::InvalidIssuer => return Err(DecodeTokenError::InvalidIssuer),
            _ => return Err(DecodeTokenError::OtherError),
        },
    };

    Ok(token_data)
}

pub enum DecodeTokenError {
    InvalidToken,
    InvalidIssuer,
    OtherError,
}
