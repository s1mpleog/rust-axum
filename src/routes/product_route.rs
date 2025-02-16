use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post, put};
use axum::{middleware, Router};
use tower_http::limit::RequestBodyLimitLayer;

use crate::config::app_state::AppState;
use crate::middlewares::admin_middleware::is_admin;
use crate::services::product_service::*;

pub fn product_route(app_state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/create", post(create_products))
        .layer(middleware::from_fn_with_state(app_state.clone(), is_admin))
        .route("/image/{id}", put(upload_product_image))
        .layer(middleware::from_fn_with_state(app_state.clone(), is_admin))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(15 * 1024 * 1024))
        .route("/all", get(get_all_products))
        .layer(middleware::from_fn_with_state(app_state.clone(), is_admin))
        .route("/filter", get(filter_products))
}
