mod database;
use std::sync::Arc;

use self::database::mongo;
mod config;
mod logger;
mod middlewares;
mod routes;
mod services;
mod utils;

use config::app_state::AppState;
use logger::init_logger::init_logger;
use routes::app::app;
mod models;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect(".env file not found");
    init_logger();

    let db = mongo::connect_to_mongodb().await;

    let app_state = Arc::new(AppState { db });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app(app_state)).await.unwrap();
}
