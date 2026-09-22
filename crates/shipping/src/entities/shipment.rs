use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shipments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pennylane_invoice_id: i64,
    pub invoice_number: String,
    pub pennylane_customer_id: i64,
    pub customer_name: String,
    pub delivery_address: String,
    pub delivery_postal_code: String,
    pub delivery_city: String,
    pub delivery_country: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::shipment_line::Entity")]
    ShipmentLine,
}

impl Related<super::shipment_line::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShipmentLine.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub mod status {
    pub const PENDING: &str = "pending";
    pub const READY: &str = "ready";
    /// Pushed to Sendcloud as an order for human review — no label created yet.
    pub const SENT_TO_SENDCLOUD: &str = "sent_to_sendcloud";
    pub const SHIPPED: &str = "shipped";
}
