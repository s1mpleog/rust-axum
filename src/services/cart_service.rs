use std::{str::FromStr, sync::Arc};

use axum::{
    extract::State,
    http::{Extensions, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_macros::debug_handler;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};

use crate::{
    config::app_state::AppState,
    models::{
        cart_model::{Cart, CartItem},
        user_model::User,
    },
};

#[debug_handler]
pub async fn add_to_cart(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(input): Json<CartItem>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let collection: Collection<Cart> = app_state.db.collection("cart");

    tracing::info!("USER FROM CART: {:?}", &user.id);

    tracing::info!("PRODUCT_ID: {:?}", &input.product_id);

    let object_id = match ObjectId::from_str(input.product_id.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };

    let filter = doc! {"_id": &object_id};

    let result = collection.find_one(filter).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch products".to_string(),
        )
    })?;

    match result {
        Some(cart) => {
            tracing::debug!("cartId?: {:?}", cart.products);
            return Ok(StatusCode::OK);
        }

        None => {
            let user_id = match user.id {
                Some(id) => id,
                None => return Err((StatusCode::UNAUTHORIZED, "UNAUTHORIZED".to_string())),
            };

            let products = vec![input];

            let new_cart = Cart {
                products,
                user_id,
                total_price: None,
            };

            collection.insert_one(new_cart).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Server Error".to_string(),
                )
            })?;

            Ok(StatusCode::OK)
        }
    }
}
