use axum::routing::{get, post};
use axum::{middleware, Router};
use std::sync::Arc;

use crate::middlewares::auth_middleware::validate_user;
use crate::services::auth_service::logout;
use crate::{
    config::app_state::AppState,
    services::auth_service::{login, me},
};

pub fn auth_route(app_state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::<Arc<AppState>>::new()
        .route("/login", post(login))
        .route(
            "/me",
            get(me).layer(middleware::from_fn_with_state(
                app_state.clone(),
                validate_user,
            )),
        )
        .route(
            "/logout",
            post(logout).layer(middleware::from_fn_with_state(
                app_state.clone(),
                validate_user,
            )),
        )
}
