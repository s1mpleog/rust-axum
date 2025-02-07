use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use crate::services::auth_service::logout;
use crate::{
    config::app_state::AppState,
    services::auth_service::{login, me},
};

pub fn auth_route() -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/logout", post(logout))
}
