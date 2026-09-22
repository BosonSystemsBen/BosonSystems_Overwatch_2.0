use chrono::Utc;
use common::{AppError, AppResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::{sendcloud_order, shipment, shipment_line};

#[derive(Debug, Deserialize)]
pub struct NewShipmentLine {
    pub pennylane_line_id: i64,
    pub product_id: Option<Uuid>,
    pub label: String,
    pub quantity: String,
    pub amount_eur: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewShipment {
    pub pennylane_invoice_id: i64,
    pub invoice_number: String,
    pub pennylane_customer_id: i64,
    pub customer_name: String,
    pub delivery_address: String,
    pub delivery_postal_code: String,
    pub delivery_city: String,
    pub delivery_country: String,
    pub lines: Vec<NewShipmentLine>,
}

/// Creates the shipment (and its lines) for a Pennylane invoice if it hasn't
/// been imported yet. Returns the existing shipment unchanged otherwise —
/// sync is idempotent, invoices are immutable once finalized on Pennylane's side.
pub async fn upsert_shipment(
    db: &DatabaseConnection,
    input: NewShipment,
) -> AppResult<shipment::Model> {
    if let Some(existing) = shipment::Entity::find()
        .filter(shipment::Column::PennylaneInvoiceId.eq(input.pennylane_invoice_id))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    let now = Utc::now();
    let shipment_id = Uuid::new_v4();
    let model = shipment::ActiveModel {
        id: Set(shipment_id),
        pennylane_invoice_id: Set(input.pennylane_invoice_id),
        invoice_number: Set(input.invoice_number),
        pennylane_customer_id: Set(input.pennylane_customer_id),
        customer_name: Set(input.customer_name),
        delivery_address: Set(input.delivery_address),
        delivery_postal_code: Set(input.delivery_postal_code),
        delivery_city: Set(input.delivery_city),
        delivery_country: Set(input.delivery_country),
        status: Set(shipment::status::PENDING.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let created = model.insert(db).await?;

    for line in input.lines {
        let line_model = shipment_line::ActiveModel {
            id: Set(Uuid::new_v4()),
            shipment_id: Set(shipment_id),
            pennylane_line_id: Set(line.pennylane_line_id),
            product_id: Set(line.product_id),
            label: Set(line.label),
            quantity: Set(line.quantity),
            amount_eur: Set(line.amount_eur),
            created_at: Set(now),
            updated_at: Set(now),
        };
        line_model.insert(db).await?;
    }

    Ok(created)
}

pub async fn list_shipments(
    db: &DatabaseConnection,
    status: Option<String>,
) -> AppResult<Vec<shipment::Model>> {
    let mut query = shipment::Entity::find();
    if let Some(status) = status {
        query = query.filter(shipment::Column::Status.eq(status));
    }
    Ok(query.all(db).await?)
}

pub async fn get_shipment(db: &DatabaseConnection, id: Uuid) -> AppResult<shipment::Model> {
    shipment::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn get_shipment_line(
    db: &DatabaseConnection,
    id: Uuid,
) -> AppResult<shipment_line::Model> {
    shipment_line::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list_shipment_lines(
    db: &DatabaseConnection,
    shipment_id: Uuid,
) -> AppResult<Vec<shipment_line::Model>> {
    Ok(shipment_line::Entity::find()
        .filter(shipment_line::Column::ShipmentId.eq(shipment_id))
        .all(db)
        .await?)
}

pub async fn update_shipment_status(
    db: &DatabaseConnection,
    id: Uuid,
    status: String,
) -> AppResult<shipment::Model> {
    let existing = get_shipment(db, id).await?;
    let mut active: shipment::ActiveModel = existing.into();
    active.status = Set(status);
    active.updated_at = Set(Utc::now());
    Ok(active.update(db).await?)
}

#[derive(Debug, Deserialize)]
pub struct NewSendcloudOrder {
    pub shipment_id: Uuid,
    pub sendcloud_order_id: i64,
    pub order_number: String,
}

pub async fn record_sendcloud_order(
    db: &DatabaseConnection,
    input: NewSendcloudOrder,
) -> AppResult<sendcloud_order::Model> {
    let now = Utc::now();
    let model = sendcloud_order::ActiveModel {
        id: Set(Uuid::new_v4()),
        shipment_id: Set(input.shipment_id),
        sendcloud_order_id: Set(input.sendcloud_order_id),
        order_number: Set(input.order_number),
        created_at: Set(now),
        updated_at: Set(now),
    };
    Ok(model.insert(db).await?)
}

pub async fn get_sendcloud_order_for_shipment(
    db: &DatabaseConnection,
    shipment_id: Uuid,
) -> AppResult<Option<sendcloud_order::Model>> {
    Ok(sendcloud_order::Entity::find()
        .filter(sendcloud_order::Column::ShipmentId.eq(shipment_id))
        .one(db)
        .await?)
}
