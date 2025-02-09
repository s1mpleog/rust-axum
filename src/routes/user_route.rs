use std::sync::Arc;

use crate::{config::app_state::AppState, services::user_service::*};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/register", post(register))
        .route("/verify", post(verify))
        .route(
            "/{id}",
            get(get_user_by_id).delete(delete_user).put(update_user),
        )
        .route("/all", get(get_all_users))
        .route("/avatar", post(test_multipart))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
}
