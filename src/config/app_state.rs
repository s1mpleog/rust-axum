use mongodb::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub secret: String,
}
