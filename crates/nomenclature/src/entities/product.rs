use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub requires_serial: bool,
    pub detection_pattern: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::serial_number::Entity")]
    SerialNumber,
}

impl Related<super::serial_number::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SerialNumber.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
