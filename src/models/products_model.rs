use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Products {
    pub _id: Option<ObjectId>,
    pub title: String,
    pub description: String,
    pub price: f32,
    pub offer_price: Option<f32>,
    pub category: String,
    pub image_url: Option<Vec<String>>,
    pub brand: String,
}

#[derive(Debug, Deserialize)]
pub struct ProductPaginate {
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct ProductFilter {
    pub category: Option<String>,
    pub title: Option<String>,
    pub brand: Option<String>,
}
