use chrono::Utc;
use common::{AppError, AppResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::{product, serial_number};

#[derive(Debug, Deserialize)]
pub struct NewProduct {
    pub sku: String,
    pub name: String,
    pub requires_serial: bool,
    pub detection_pattern: Option<String>,
    #[serde(default)]
    pub pennylane_product_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub requires_serial: Option<bool>,
    pub detection_pattern: Option<Option<String>>,
    #[serde(default)]
    pub pennylane_product_id: Option<Option<i64>>,
}

fn validate_pattern(pattern: &Option<String>) -> AppResult<()> {
    if let Some(p) = pattern {
        regex::Regex::new(p)
            .map_err(|e| AppError::Validation(format!("motif de détection invalide: {e}")))?;
    }
    Ok(())
}

pub async fn create_product(db: &DatabaseConnection, input: NewProduct) -> AppResult<product::Model> {
    if input.sku.trim().is_empty() || input.name.trim().is_empty() {
        return Err(AppError::Validation("sku et name sont requis".into()));
    }
    validate_pattern(&input.detection_pattern)?;

    let now = Utc::now();
    let model = product::ActiveModel {
        id: Set(Uuid::new_v4()),
        sku: Set(input.sku),
        name: Set(input.name),
        requires_serial: Set(input.requires_serial),
        detection_pattern: Set(input.detection_pattern),
        created_at: Set(now),
        updated_at: Set(now),
        pennylane_product_id: Set(input.pennylane_product_id),
    };
    Ok(model.insert(db).await?)
}

pub async fn find_product_by_pennylane_id(
    db: &DatabaseConnection,
    pennylane_product_id: i64,
) -> AppResult<Option<product::Model>> {
    Ok(product::Entity::find()
        .filter(product::Column::PennylaneProductId.eq(pennylane_product_id))
        .one(db)
        .await?)
}

pub async fn list_products(db: &DatabaseConnection) -> AppResult<Vec<product::Model>> {
    Ok(product::Entity::find().all(db).await?)
}

pub async fn get_product(db: &DatabaseConnection, id: Uuid) -> AppResult<product::Model> {
    product::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn update_product(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateProduct,
) -> AppResult<product::Model> {
    if let Some(p) = &input.detection_pattern {
        validate_pattern(p)?;
    }
    let existing = get_product(db, id).await?;
    let mut active: product::ActiveModel = existing.into();

    if let Some(sku) = input.sku {
        active.sku = Set(sku);
    }
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    if let Some(requires_serial) = input.requires_serial {
        active.requires_serial = Set(requires_serial);
    }
    if let Some(detection_pattern) = input.detection_pattern {
        active.detection_pattern = Set(detection_pattern);
    }
    if let Some(pennylane_product_id) = input.pennylane_product_id {
        active.pennylane_product_id = Set(pennylane_product_id);
    }
    active.updated_at = Set(Utc::now());

    Ok(active.update(db).await?)
}

pub async fn delete_product(db: &DatabaseConnection, id: Uuid) -> AppResult<()> {
    let existing = get_product(db, id).await?;
    let active: product::ActiveModel = existing.into();
    active.delete(db).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct NewSerialNumber {
    pub product_id: Uuid,
    pub value: String,
    pub assigned_to: Option<String>,
}

pub async fn create_serial_number(
    db: &DatabaseConnection,
    input: NewSerialNumber,
) -> AppResult<serial_number::Model> {
    if input.value.trim().is_empty() {
        return Err(AppError::Validation("value est requis".into()));
    }
    // Product must exist.
    get_product(db, input.product_id).await?;

    let now = Utc::now();
    let status = if input.assigned_to.is_some() {
        serial_number::status::ASSIGNED
    } else {
        serial_number::status::IN_STOCK
    };

    let model = serial_number::ActiveModel {
        id: Set(Uuid::new_v4()),
        product_id: Set(input.product_id),
        value: Set(input.value),
        status: Set(status.to_string()),
        assigned_to: Set(input.assigned_to),
        created_at: Set(now),
        updated_at: Set(now),
        shipment_line_id: Set(None),
    };
    Ok(model.insert(db).await?)
}

pub async fn link_serial_number_to_shipment_line(
    db: &DatabaseConnection,
    serial_number_id: Uuid,
    shipment_line_id: Uuid,
    assigned_to: String,
) -> AppResult<serial_number::Model> {
    let existing = serial_number::Entity::find_by_id(serial_number_id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active: serial_number::ActiveModel = existing.into();
    active.shipment_line_id = Set(Some(shipment_line_id));
    active.assigned_to = Set(Some(assigned_to));
    active.status = Set(serial_number::status::ASSIGNED.to_string());
    active.updated_at = Set(Utc::now());
    Ok(active.update(db).await?)
}

pub async fn list_serial_numbers(
    db: &DatabaseConnection,
    product_id: Option<Uuid>,
    status: Option<String>,
) -> AppResult<Vec<serial_number::Model>> {
    let mut query = serial_number::Entity::find();
    if let Some(pid) = product_id {
        query = query.filter(serial_number::Column::ProductId.eq(pid));
    }
    if let Some(s) = status {
        query = query.filter(serial_number::Column::Status.eq(s));
    }
    Ok(query.all(db).await?)
}

pub async fn assign_serial_number(
    db: &DatabaseConnection,
    id: Uuid,
    assigned_to: String,
) -> AppResult<serial_number::Model> {
    let existing = serial_number::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut active: serial_number::ActiveModel = existing.into();
    active.assigned_to = Set(Some(assigned_to));
    active.status = Set(serial_number::status::ASSIGNED.to_string());
    active.updated_at = Set(Utc::now());
    Ok(active.update(db).await?)
}

pub async fn delete_serial_number(db: &DatabaseConnection, id: Uuid) -> AppResult<()> {
    let existing = serial_number::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;
    let active: serial_number::ActiveModel = existing.into();
    active.delete(db).await?;
    Ok(())
}

/// Detects which product(s) a scanned serial number matches, based on each
/// product's detection_pattern regex. Several products can match the same
/// value (ambiguous patterns) — the caller decides how to present that.
pub async fn detect_product_by_serial(
    db: &DatabaseConnection,
    value: &str,
) -> AppResult<Vec<product::Model>> {
    let products = product::Entity::find()
        .filter(product::Column::DetectionPattern.is_not_null())
        .all(db)
        .await?;

    let matches = products
        .into_iter()
        .filter(|p| {
            p.detection_pattern
                .as_deref()
                .and_then(|pattern| regex::Regex::new(pattern).ok())
                .map(|re| re.is_match(value))
                .unwrap_or(false)
        })
        .collect();

    Ok(matches)
}
