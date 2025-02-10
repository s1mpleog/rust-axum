use std::sync::Arc;

use axum::routing::post;
use axum::{middleware, Router};

use crate::config::app_state::AppState;
use crate::middlewares::admin_middleware::is_admin;
use crate::services::product_service::*;

pub fn product_route(app_state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/create", post(create_products))
        .layer(middleware::from_fn_with_state(app_state.clone(), is_admin))
}
