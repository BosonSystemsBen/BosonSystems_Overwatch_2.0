use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sendcloud_labels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub sendcloud_shipment_id: String,
    pub sendcloud_parcel_id: i64,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub label_link: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::shipment::Entity",
        from = "Column::ShipmentId",
        to = "super::shipment::Column::Id"
    )]
    Shipment,
}

impl Related<super::shipment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Shipment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
