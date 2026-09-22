use pennylane::PennylaneClient;
use sea_orm::DatabaseConnection;
use sendcloud::SendcloudClient;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub pennylane: Option<PennylaneClient>,
    pub sendcloud: Option<(SendcloudClient, i64)>,
}
