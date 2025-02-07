use std::sync::Arc;

use axum::Router;
use axum_cookie::CookieLayer;

use crate::config::app_state::AppState;

use super::{auth_route::auth_route, user_route::user_routes};

pub fn app(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/user", user_routes())
        .nest("/api/auth", auth_route())
        .with_state(app_state)
        .layer(CookieLayer::strict())
}
