use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub name: String,
    pub age: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TempUser {
    #[serde(skip_deserializing)]
    pub _id: String,
    pub otp: Option<String>,
    pub email: String,
    pub password: String,
    pub name: String,
    #[serde(skip_deserializing)]
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyOtpInput {
    pub otp: String,
}
