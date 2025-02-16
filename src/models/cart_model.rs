use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cart {
    pub _id: Option<ObjectId>,
    pub products: Vec<CartItem>,
    pub user_id: ObjectId,
    pub total_price: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCart {
    pub product_id: String,
}
