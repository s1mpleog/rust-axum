use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use bcrypt::{BcryptError, DEFAULT_COST};
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Client, Collection, Database,
};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rust_dotenv::dotenv::DotEnv;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    name: String,
    email: String,
    password: String,
    age: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateUser {
    name: String,
    age: u8,
}

#[tokio::main]
async fn main() {
    init_logger();

    let client = connect_to_mongodb().await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app(client)).await.unwrap();

    println!("tokio");
}

async fn connect_to_mongodb() -> Client {
    let uri = match load_env("MONGODB_URI".to_string()) {
        Ok(env) => env.to_string(),
        Err(e) => e.to_string(),
    };

    let client = Client::with_uri_str(uri).await.unwrap();

    client
        .database("todo")
        .run_command(doc! {"ping": 1})
        .await
        .unwrap();

    tracing::info!("Pinged your databse. Successfully connected to mongodb");

    client
}

fn load_env(var: String) -> Result<String, &'static str> {
    let dotenv = DotEnv::new("");

    if let Some(env) = dotenv.get_var(var.to_string()) {
        return Ok(env);
    } else {
        tracing::error!("failed to load env");
        return Err("faled to load env");
    }
}

fn init_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn app(client: Client) -> Router {
    let db = client.database("todo");

    Router::new()
        .route("/", get(handler))
        .route("/user/create", post(create_user))
        .route(
            "/user/{id}",
            get(get_user_by_id).delete(delete_user).put(update_user),
        )
        .route("/user/all", get(get_all_users))
        .with_state(db)
}

fn parse_object_id(id: String) -> Result<ObjectId, (StatusCode, String)> {
    let object_id = match ObjectId::parse_str(&id) {
        Ok(oid) => Ok(oid),
        Err(_) => Err((StatusCode::BAD_REQUEST, "Invalid Id".to_string())),
    };
    return object_id;
}

fn hash_password(password: String) -> Result<String, BcryptError> {
    let hashed = match bcrypt::hash(&password, DEFAULT_COST) {
        Ok(password) => Ok(password.to_string()),
        Err(e) => Err(e),
    };
    return hashed;
}

#[allow(dead_code)] // just for development
fn verify_password(password: String, hashed_password: &str) -> Result<bool, BcryptError> {
    let result = match bcrypt::verify(password, hashed_password) {
        Ok(valid) => Ok(valid),
        Err(e) => Err(e),
    };
    return result;
}

async fn create_user(
    State(db): State<Database>,
    Json(mut input): Json<User>,
) -> Result<Json<InsertOneResult>, (StatusCode, String)> {
    let collection: Collection<User> = db.collection("users");
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

async fn get_all_users(
    State(db): State<Database>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let collection: Collection<User> = db.collection("users");

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

async fn get_user_by_id(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<User>, (StatusCode, String)> {
    let obj_id = match parse_object_id(id) {
        Ok(id) => id,
        Err(e) => return Err((e.0, e.1.to_string())),
    };

    let filter = doc! {"_id": obj_id};

    let collection: Collection<User> = db.collection("users");

    let user = collection
        .find_one(filter)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    match user {
        Some(value) => Ok(Json(value)),
        None => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    }
}

async fn update_user(
    State(db): State<Database>,
    Path(id): Path<String>,
    Json(input): Json<UpdateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<User> = db.collection("users");
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

async fn delete_user(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<String>, (StatusCode, String)> {
    let object_id = match parse_object_id(id) {
        Ok(id) => id,
        Err(e) => return Err((e.0, e.1.to_string())),
    };

    let collection: Collection<User> = db.collection("users");

    let filter = doc! {"_id": object_id};

    collection
        .find_one_and_delete(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(String::from("user deleted success")))
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello world!</h1>")
}
