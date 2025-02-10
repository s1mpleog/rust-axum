use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Products {
    pub _id: Option<String>,
    pub title: String,
    pub description: String,
    pub price: f32,
    pub offer_price: Option<f32>,
    pub category: String,
    pub image_url: Option<Vec<String>>,
    pub brand: String,
}
