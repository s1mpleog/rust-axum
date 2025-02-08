use std::sync::Arc;

use crate::{config::app_state::AppState, services::user_service::*};
use axum::{routing::get, routing::post, Router};

pub fn user_routes() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/register", post(register))
        .route("/verify", post(verify))
        .route(
            "/{id}",
            get(get_user_by_id).delete(delete_user).put(update_user),
        )
        .route("/all", get(get_all_users))
}
