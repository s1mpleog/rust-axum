use std::{str::FromStr, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use axum_macros::debug_handler;
use mongodb::bson;
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

    let user_id = match user.id {
        Some(id) => id,
        None => return Err((StatusCode::UNAUTHORIZED, "UNAUTHORIZED".to_string())),
    };

    match ObjectId::from_str(input.product_id.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };

    let filter = doc! { "user_id": &user_id };

    let result = collection
        .find_one(filter.clone()) // Clone filter for later use
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string()))?;

    match result {
        Some(mut cart) => {
            if let Some(item) = cart
                .products
                .iter_mut()
                .find(|p| p.product_id == input.product_id)
            {
                item.quantity += input.quantity;
            } else {
                cart.products.push(input);
            }

            let update = doc! {
                "$set": { "products": bson::to_bson(&cart.products).unwrap() }
            };

            collection
                .update_one(filter, update)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok((StatusCode::OK, "Updated cart".to_string()))
        }
        None => {
            let new_cart = Cart {
                _id: Some(ObjectId::new()),
                products: vec![input],
                user_id,
                total_price: None,
            };

            collection
                .insert_one(new_cart)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string()))?;

            Ok((StatusCode::OK, "CREATED".to_string()))
        }
    }
}
