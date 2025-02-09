use std::sync::Arc;

use crate::{
    models::user_model::{TempUser, VerifyOtpInput},
    utils::{
        bcrypt::hash_password,
        generate_otp::create_otp,
        s3::{configure_s3, upload_single},
        send_email::send_mail,
    },
};
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_cookie::{cookie::Cookie, prelude::SameSite, CookieManager};
use axum_macros::debug_handler;
use chrono::{Duration, Utc};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use std::env;
use uuid::Uuid;

use crate::{
    config::app_state::AppState,
    models::user_model::{UpdateUser, User},
    utils::parse_id::parse_object_id,
};

#[debug_handler]
pub async fn register(
    State(app_state): State<Arc<AppState>>,
    cookie: CookieManager,
    Json(input): Json<TempUser>,
) -> Result<Json<String>, (StatusCode, String)> {
    let user_collection: Collection<User> = app_state.db.collection("users");
    let filter = doc! {"email": &input.email.to_lowercase()};
    let temp_user_collection: Collection<TempUser> = app_state.db.collection("temp-user");

    let is_user_exists = user_collection
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
            let id = Uuid::new_v4().to_string();

            let otp = create_otp();

            match send_mail(&input.name, &input.email, &otp).await {
                Ok(_) => true,
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to send email".to_string(),
                    ))
                }
            };

            let hashed = match hash_password(input.password) {
                Ok(hashed) => hashed,
                Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            };

            let temp_user = TempUser {
                _id: id.clone(),
                email: input.email.to_lowercase(),
                password: hashed,
                otp: Some(otp.to_string()),
                name: input.name,
                expires_at: Utc::now() + Duration::minutes(5),
            };

            temp_user_collection
                .insert_one(temp_user)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let mut session_token = Cookie::new("session_token", id);
            session_token.set_http_only(true);
            session_token.set_same_site(SameSite::Strict);

            cookie.set(session_token);

            return Ok(Json(
                "A 6 digit otp has been sent to your gmail".to_string(),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "server error".to_string(),
            ))
        }
    }
}

#[debug_handler]
pub async fn verify(
    State(app_state): State<Arc<AppState>>,
    cookie: CookieManager,
    Json(input): Json<VerifyOtpInput>,
) -> impl IntoResponse {
    let secret_token = match cookie.get("session_token") {
        Some(cookie) => cookie,
        None => return Err((StatusCode::BAD_REQUEST, "your session have been expired")),
    };

    let temp_user_collection: Collection<TempUser> = app_state.db.collection("temp-user");

    let temp_user = temp_user_collection
        .find_one(doc! {"_id": secret_token.value()})
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Server Error"));

    match temp_user {
        Ok(Some(usr)) => {
            let stored_otp = match &usr.otp {
                Some(otp) => otp,
                None => return Err((StatusCode::BAD_REQUEST, "your session is expired")),
            };

            if stored_otp.to_string() != input.otp.to_string() {
                return Err((StatusCode::BAD_REQUEST, "Invalid OTP"));
            }

            let user_id = Some(ObjectId::new());

            let user = User {
                id: user_id,
                email: usr.email.to_lowercase(),
                password: usr.password,
                name: usr.name,
            };

            let user_collection: Collection<User> = app_state.db.collection("users");

            let result = user_collection
                .insert_one(user)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Server Error"));

            match result {
                Ok(_) => {
                    let result = temp_user_collection
                        .delete_one(doc! {"_id": secret_token.value()})
                        .await
                        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "server error"));

                    match result {
                        Ok(_) => {
                            cookie.remove("session_token");
                            return Ok((StatusCode::CREATED, "User created successfully"));
                        }
                        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "server error")),
                    };
                }
                Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server Error")),
            };
        }
        Ok(None) => return Err((StatusCode::BAD_REQUEST, "your session is expired")),
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server error")),
    };
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

#[debug_handler]
pub async fn test_multipart(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("unknown").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("unknown").to_string();
        let data = field.bytes().await.unwrap();

        if name == "avatar" {
            let avatar_type = content_type.clone();
            let avatar_name = file_name.clone();

            let result = match upload_single(data.to_vec(), &avatar_type, &avatar_name).await {
                Ok(url) => url,
                Err(_) => {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, "falied to upload avatar"))
                }
            };

            return Ok(Json(result));
        }
    }

    return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server Error"));
}
