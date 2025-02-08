use std::sync::Arc;

use crate::utils::bcrypt::hash_password;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::{bson::doc, bson::oid::ObjectId, results::InsertOneResult, Collection};

use crate::{
    config::app_state::AppState,
    models::user_model::{UpdateUser, User},
    utils::parse_id::parse_object_id,
};

pub async fn create_user(
    State(app_state): State<Arc<AppState>>,
    Json(mut input): Json<User>,
) -> Result<Json<InsertOneResult>, (StatusCode, String)> {
    let collection: Collection<User> = app_state.db.collection("users");
    let filter = doc! {"email": &input.email.to_lowercase()};

    let is_user_exists = collection
        .find_one(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));

    match is_user_exists {
        Ok(Some(_)) => {
            return Err((
                StatusCode::CONFLICT,
                "user with this email already exists".to_string(),
            ));
        }
        Ok(None) => {
            input.id = Some(ObjectId::new());
            input.email = input.email.to_lowercase();

            match hash_password(input.password) {
                Ok(hashed) => input.password = hashed,
                Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            };

            let result = collection
                .insert_one(input)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            return Ok(Json(result));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "server error".to_string(),
            ))
        }
    }
}

pub async fn get_all_users(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let collection: Collection<User> = app_state.db.collection("users");

    let mut users: Vec<User> = vec![];

    let mut cursor = collection
        .find(doc! {})
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    while cursor
        .advance()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        users.push(
            cursor
                .deserialize_current()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        );
    }
    Ok(Json(users))
}

pub async fn get_user_by_id(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<User>, (StatusCode, String)> {
    let obj_id = match parse_object_id(id) {
        Ok(id) => id,
        Err(e) => return Err((e.0, e.1.to_string())),
    };

    let filter = doc! {"_id": obj_id};

    let collection: Collection<User> = app_state.db.collection("users");

    let user = collection
        .find_one(filter)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    match user {
        Some(value) => Ok(Json(value)),
        None => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    }
}

pub async fn update_user(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<User> = app_state.db.collection("users");
    let object_id = match parse_object_id(id) {
        Ok(oid) => oid,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let filter = doc! {"_id": object_id};
    let update = doc! {"$set": doc! {"name": &input.name, "age": input.age as i32}};

    let result = collection
        .update_one(filter, update)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.matched_count == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_user(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<String>, (StatusCode, String)> {
    let object_id = match parse_object_id(id) {
        Ok(id) => id,
        Err(e) => return Err((e.0, e.1.to_string())),
    };

    let collection: Collection<User> = app_state.db.collection("users");

    let filter = doc! {"_id": object_id};

    collection
        .find_one_and_delete(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(String::from("user deleted success")))
}
