use std::{sync::Arc, time::Duration};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_cookie::{cookie::Cookie, prelude::SameSite, CookieManager};
use axum_macros::debug_handler;
use mongodb::{bson::doc, Collection};

use crate::{
    config::app_state::AppState,
    models::{auth_model::Login, user_model::User},
    utils::{
        bcrypt::verify_password,
        jwt::{create_token, decode_token, DecodeTokenError},
    },
};

#[debug_handler]
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    cookie: CookieManager,
    Json(input): Json<Login>,
) -> impl IntoResponse {
    let is_valid_email = input.email.contains("@");

    if !is_valid_email {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let collection: Collection<User> = app_state.db.collection("users");
    let filter = doc! {"email": &input.email};

    let user = collection
        .find_one(filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    match user {
        Ok(Some(user)) => {
            match verify_password(input.password, &user.password) {
                Ok(valid) => {
                    if !valid {
                        return (StatusCode::BAD_REQUEST, "Invalid password").into_response();
                    }
                    let id = match user.id {
                        Some(id) => id,
                        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                    };
                    let access_token = create_token(id);
                    let mut auth_cookie = Cookie::new("access_token", access_token);
                    auth_cookie.set_http_only(true);
                    auth_cookie.set_max_age(Duration::from_secs(3600));
                    auth_cookie.set_same_site(SameSite::Strict);

                    cookie.set(auth_cookie);
                    return StatusCode::OK.into_response();
                }
                Err(_) => return StatusCode::BAD_REQUEST.into_response(),
            };
        }
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
}

#[debug_handler]
pub async fn me(
    State(app_state): State<Arc<AppState>>,
    cookie: CookieManager,
) -> impl IntoResponse {
    let cookie = match cookie.get("access_token") {
        Some(cookie) => cookie,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let decoded_token = match decode_token(cookie.value()) {
        Ok(data) => data,
        Err(DecodeTokenError::InvalidToken) => {
            return (StatusCode::BAD_REQUEST, "your token is invalid").into_response()
        }
        Err(DecodeTokenError::InvalidIssuer) => {
            return (StatusCode::BAD_REQUEST, "Invalid Issuer").into_response()
        }
        Err(DecodeTokenError::OtherError) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "ServerError").into_response()
        }
    };

    let collection: Collection<User> = app_state.db.collection("users");
    let user_id = &decoded_token.claims.user_id;

    let filter = doc! {"_id":  user_id};

    let user = collection
        .find_one(filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());

    match user {
        Ok(Some(user)) => {
            return (StatusCode::OK, Json(user)).into_response();
        }
        Ok(None) => {
            return (StatusCode::BAD_REQUEST, "user not found invalid request").into_response();
        }

        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn logout(cookie: CookieManager) -> impl IntoResponse {
    match cookie.get("access_token") {
        Some(c) => {
            cookie.remove(c.name());
            return (StatusCode::OK, "user logged out success").into_response();
        }
        None => return (StatusCode::UNAUTHORIZED, "you are not logged in").into_response(),
    };
}
