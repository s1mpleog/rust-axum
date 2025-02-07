use mongodb::bson::doc;
use mongodb::{Client, Database};
use std::env;

pub async fn connect_to_mongodb() -> Database {
    dotenvy::dotenv().expect(".env file not found");

    let uri = match env::var("MONGODB_URI") {
        Ok(env) => env,
        Err(e) => e.to_string(),
    };

    let database_name = match env::var("DATABASE_NAME") {
        Ok(name) => name,
        Err(e) => e.to_string(),
    };

    let client = Client::with_uri_str(uri.to_string())
        .await
        .expect("failed to connect_to_mongodb");

    let database = client.database(&database_name);

    database
        .run_command(doc! {"ping": 1})
        .await
        .expect("failed to ping databse");

    println!("successfully connected to mongodb");

    database
}
