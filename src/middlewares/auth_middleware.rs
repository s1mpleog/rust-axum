use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_cookie::CookieManager;
use axum_macros::debug_middleware;
use mongodb::{bson::doc, Collection};
use serde::Serialize;

use crate::{config::app_state::AppState, models::user_model::User, utils::jwt::decode_token};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

#[debug_middleware]
pub async fn validate_user(
    cookie: CookieManager,
    State(app_state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = match cookie.get("access_token") {
        Some(c) => c,
        None => {
            let error_response = ErrorResponse {
                status: "fail",
                message: "You are not logged in".to_string(),
            };
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    let decoded = match decode_token(token.value()) {
        Ok(data) => data,
        Err(_) => {
            let error_response = ErrorResponse {
                status: "fail",
                message: "your token is invalid please login again".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    let user_id = decoded.claims.user_id;

    let collection: Collection<User> = app_state.db.collection("users");

    let filter = doc! {"_id": &user_id};

    let mut error_response = ErrorResponse {
        status: "fail",
        message: "your token is invalid please login again".to_string(),
    };

    let user = collection
        .find_one(filter)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR));

    match user {
        Ok(Some(usr)) => {
            request.extensions_mut().insert(usr);
            Ok(next.run(request).await)
        }
        Err(_) => {
            error_response.message = "Internal Server Error".to_string();
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
        Ok(None) => {
            error_response.message = "Invalid Token user not found".to_string();
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}
