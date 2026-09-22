use pennylane::PennylaneClient;
use sea_orm::DatabaseConnection;
use sendcloud::SendcloudClient;

#[derive(Clone)]
pub struct SendcloudSettings {
    pub sender_address_id: i64,
    pub shipping_option_code: String,
    pub contract_id: Option<i64>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub pennylane: Option<PennylaneClient>,
    pub sendcloud: Option<(SendcloudClient, SendcloudSettings)>,
}
