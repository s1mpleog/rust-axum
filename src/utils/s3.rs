use aws_sdk_s3::{primitives::ByteStream, Client};
use mongodb::bson::Uuid;
use std::env;

pub async fn configure_s3() -> Client {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    client
}

pub async fn upload_single(
    file_bytes: Vec<u8>,
    file_type: &String,
    file_name: &String,
) -> Result<String, String> {
    let client = configure_s3().await;

    let bucket_name = env::var("AWS_BUCKET_NAME").expect("no env found");

    let region = env::var("AWS_REGION").expect("no env found");

    let uuid = Uuid::new().to_string();
    let key = format!("upload/{}.{}", &uuid, &file_name);

    let result = client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .content_type(file_type)
        .body(ByteStream::from(file_bytes))
        .send()
        .await;

    match result {
        Ok(_) => {
            let url = format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                &bucket_name, &region, &key
            );

            Ok(url)
        }
        Err(_) => Err("Failed to upload to s3".to_string()),
    }
}
