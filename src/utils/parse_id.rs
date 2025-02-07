use axum::http::StatusCode;
use mongodb::bson::oid::ObjectId;

pub fn parse_object_id(id: String) -> Result<ObjectId, (StatusCode, String)> {
    let object_id = match ObjectId::parse_str(&id) {
        Ok(oid) => Ok(oid),
        Err(_) => Err((StatusCode::BAD_REQUEST, "Invalid Id".to_string())),
    };
    return object_id;
}
