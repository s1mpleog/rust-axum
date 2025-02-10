use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use axum_cookie::CookieManager;
use axum_macros::debug_middleware;
use mongodb::{bson::doc, Collection};

use crate::{config::app_state::AppState, models::user_model::User, utils::jwt::decode_token};

#[debug_middleware]
pub async fn is_admin(
    cookie: CookieManager,
    State(app_state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let token = cookie
        .get("access_token")
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "UNAUTHORIZED".to_string()))?;

    let payload = decode_token(token.value()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to decode token".to_string(),
        )
    })?;

    let collection: Collection<User> = app_state.db.collection("users");
    let filter = doc! {"_id": payload.claims.user_id};

    let user = collection
        .find_one(filter)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch user".to_string(),
            )
        })?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "UNAUTHORIZED".to_string()))?;

    let role = user.role.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "You are not allowed for this request".to_string(),
        )
    })?;

    tracing::debug!("ROLE: {}", role);
    if role != "admin" {
        return Err((StatusCode::UNAUTHORIZED, "UNAUTHORIZED".to_string()));
    }

    Ok(next.run(request).await)
}
