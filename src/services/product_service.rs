use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use mongodb::{
    bson::{doc, Uuid},
    Collection,
};

use crate::{
    config::app_state::AppState,
    models::products_model::{ProductFilter, ProductPaginate, Products},
    utils::s3::upload_single,
};

pub async fn create_products(
    State(app_state): State<Arc<AppState>>,
    Json(mut data): Json<Products>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let collection: Collection<Products> = app_state.db.collection("products");

    let product_id = Uuid::new().to_string();

    data._id = Some(product_id);

    let result = collection.insert_one(data).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to create product".to_string(),
        )
    })?;

    return Ok(Json(result));
}

#[debug_handler]
pub async fn upload_product_image(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Mutlipart error {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read multipart fields".to_string(),
        )
    })? {
        let name = field.name().unwrap_or("unknown").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("unknown").to_string();
        let data = field.bytes().await.unwrap();

        if name == "images" {
            let image_name = file_name.clone();
            let image_type = content_type.clone();
            let image_bytes = data.to_vec();

            let collection: Collection<Products> = app_state.db.collection("products");

            let upload_result = upload_single(image_bytes, &image_type, &image_name)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to upload image".to_string(),
                    )
                })?;

            tracing::debug!("UPLOADED_URL: {upload_result}");

            let update = doc! {"$push": doc! {"image_url": upload_result }};
            let filter = doc! {"_id": &id};

            let product = collection
                .find_one_and_update(filter, update)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            match product {
                Some(_) => {
                    tracing::info!("PRODUCT UPDATED IMAGE UPLOADED");
                }
                None => return Err((StatusCode::NOT_FOUND, "Product not found".to_string())),
            };

            tracing::info!("PRODUCT_ID: {id}");
        }
    }
    Ok(StatusCode::OK)
}

#[debug_handler]
pub async fn get_all_products(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ProductPaginate>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let collection: Collection<Products> = app_state.db.collection("products");

    let mut products = vec![];
    let page = query.page.max(1);
    let limit_per_page = 5;

    tracing::debug!("QUERY:? {:?}", query);

    let pipeline = vec![
        doc! {"$skip": ((page - 1) * limit_per_page)},
        doc! {"$limit": limit_per_page},
    ];

    let mut cursor = collection.aggregate(pipeline).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch products here 1".to_string(),
        )
    })?;

    while cursor.advance().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch products here 2".to_string(),
        )
    })? {
        products.push(cursor.deserialize_current().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something went wrong".to_string(),
            )
        })?);
    }

    tracing::info!("{:?}", products);
    return Ok(Json(products));
}

#[debug_handler]
pub async fn filter_products(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<ProductFilter>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let collection: Collection<Products> = app_state.db.collection("products");

    let search_query = vec![
        query.title.unwrap_or_else(|| "".to_string()),
        query.brand.unwrap_or_else(|| "".to_string()),
        query.category.unwrap_or_else(|| "".to_string()),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>();

    tracing::info!("SEARCH_QUERY IS: {:?}", &search_query);

    let pipeline = vec![
        doc! {
            "$search": {
            "index": "default",
            "text": {
            "query": &search_query,
            "path": ["title", "brand", "category"]
        }
        }
        },
        doc! {"$limit": 5},
    ];

    let mut cursor = collection.aggregate(pipeline).await.map_err(|e| {
        tracing::debug!("ERROR: {}", e.to_string());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error while fetching products".to_string(),
        )
    })?;

    let mut products = Vec::new();

    while cursor.advance().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong".to_string(),
        )
    })? {
        products.push(
            cursor
                .deserialize_current()
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed".to_string()))?,
        )
    }

    Ok(Json(products))
}
