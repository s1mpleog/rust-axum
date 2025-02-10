use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::{bson::Uuid, Collection};

use crate::{config::app_state::AppState, models::products_model::Products};

pub async fn create_products(
    State(app_state): State<Arc<AppState>>,
    Json(mut data): Json<Products>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let collection: Collection<Products> = app_state.db.collection("products");

    let product_id = Uuid::new().to_string();

    data._id = Some(product_id);

    let result = collection.insert_one(data).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to create product".to_string(),
        )
    })?;

    return Ok(Json(result));
}
