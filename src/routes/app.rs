use std::sync::Arc;

use axum::Router;

use crate::config::app_state::AppState;

use super::user_route::user_routes;

pub fn app(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/api/user", user_routes())
        .with_state(app_state)
}
