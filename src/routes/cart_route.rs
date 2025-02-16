use std::sync::Arc;

use axum::{middleware, Router};

use axum::routing::post;

use crate::middlewares::auth_middleware::validate_user;
use crate::{config::app_state::AppState, services::cart_service::*};

pub fn cart_route(app_state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/create", post(add_to_cart))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            validate_user,
        ))
}
