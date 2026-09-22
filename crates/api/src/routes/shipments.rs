use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use common::AppError;
use pennylane::PennylaneClient;
use sendcloud::types::{Address, FromAddress, Parcel, ShipWith, ShipmentRequest, Weight};
use serde::{Deserialize, Serialize};
use shipping::service::{self, NewSendcloudLabel, NewShipment, NewShipmentLine};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shipments", get(list))
        .route("/shipments/:id", get(get_one))
        .route("/shipments/:id/status", axum::routing::put(update_status))
        .route("/shipments/:id/create-label", axum::routing::post(create_label))
        .route("/pennylane/sync", axum::routing::post(sync))
        .route(
            "/shipment-lines/:id/link-serial-number",
            axum::routing::post(link_serial_number),
        )
}

async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<shipping::entities::shipment::Model>>, ApiError> {
    Ok(Json(service::list_shipments(&state.db, None).await?))
}

#[derive(Serialize)]
struct ShipmentDetail {
    shipment: shipping::entities::shipment::Model,
    lines: Vec<shipping::entities::shipment_line::Model>,
    label: Option<shipping::entities::sendcloud_label::Model>,
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ShipmentDetail>, ApiError> {
    let shipment = service::get_shipment(&state.db, id).await?;
    let lines = service::list_shipment_lines(&state.db, id).await?;
    let label = service::get_label_for_shipment(&state.db, id).await?;
    Ok(Json(ShipmentDetail { shipment, lines, label }))
}

#[derive(Deserialize)]
struct UpdateStatusInput {
    status: String,
}

async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateStatusInput>,
) -> Result<Json<shipping::entities::shipment::Model>, ApiError> {
    Ok(Json(
        service::update_shipment_status(&state.db, id, input.status).await?,
    ))
}

#[derive(Deserialize)]
struct LinkSerialNumberInput {
    serial_number_id: Uuid,
}

async fn link_serial_number(
    State(state): State<AppState>,
    Path(shipment_line_id): Path<Uuid>,
    Json(input): Json<LinkSerialNumberInput>,
) -> Result<Json<nomenclature::entities::serial_number::Model>, ApiError> {
    let line = service::get_shipment_line(&state.db, shipment_line_id).await?;
    let shipment = service::get_shipment(&state.db, line.shipment_id).await?;

    Ok(Json(
        nomenclature::service::link_serial_number_to_shipment_line(
            &state.db,
            input.serial_number_id,
            shipment_line_id,
            shipment.customer_name,
        )
        .await?,
    ))
}

#[derive(Serialize)]
struct SyncResult {
    invoices_processed: usize,
}

async fn sync(State(state): State<AppState>) -> Result<Json<SyncResult>, ApiError> {
    let client = state
        .pennylane
        .as_ref()
        .ok_or_else(|| AppError::External("Pennylane n'est pas configuré (PENNYLANE_API_TOKEN manquant)".into()))?;

    let count = sync_invoices(&state.db, client).await?;
    Ok(Json(SyncResult {
        invoices_processed: count,
    }))
}

fn pennylane_err(e: pennylane::PennylaneError) -> AppError {
    AppError::External(e.to_string())
}

fn sendcloud_err(e: sendcloud::SendcloudError) -> AppError {
    AppError::External(e.to_string())
}

#[derive(Deserialize)]
struct CreateLabelInput {
    weight_kg: f64,
}

#[derive(Serialize)]
struct CreateLabelResult {
    label: shipping::entities::sendcloud_label::Model,
    label_file_base64: Option<String>,
}

async fn create_label(
    State(state): State<AppState>,
    Path(shipment_id): Path<Uuid>,
    Json(input): Json<CreateLabelInput>,
) -> Result<Json<CreateLabelResult>, ApiError> {
    let (client, settings) = state.sendcloud.as_ref().ok_or_else(|| {
        AppError::External(
            "Sendcloud n'est pas configuré (SENDCLOUD_PUBLIC_KEY/PRIVATE_KEY/SENDER_ADDRESS_ID/SHIPPING_OPTION_CODE manquants)".into(),
        )
    })?;

    if let Some(existing) = service::get_label_for_shipment(&state.db, shipment_id).await? {
        return Ok(Json(CreateLabelResult {
            label: existing,
            label_file_base64: None,
        }));
    }

    let shipment = service::get_shipment(&state.db, shipment_id).await?;

    let request = ShipmentRequest {
        to_address: Address {
            name: shipment.customer_name.clone(),
            address_line_1: shipment.delivery_address.clone(),
            postal_code: shipment.delivery_postal_code.clone(),
            city: shipment.delivery_city.clone(),
            country_code: shipment.delivery_country.clone(),
            email: None,
            phone_number: None,
        },
        from_address: FromAddress {
            sender_address_id: settings.sender_address_id,
        },
        ship_with: ShipWith::shipping_option_code(
            settings.shipping_option_code.clone(),
            settings.contract_id,
        ),
        parcels: vec![Parcel {
            weight: Weight::kg(input.weight_kg),
        }],
        order_number: Some(shipment.invoice_number.clone()),
        external_reference_id: Some(shipment.id.to_string()),
    };

    let response = client.announce_shipment(&request).await.map_err(sendcloud_err)?;

    if !response.errors.is_empty() {
        let message = response
            .errors
            .into_iter()
            .filter_map(|e| e.detail)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AppError::External(format!("annonce du colis refusée par le transporteur: {message}")).into());
    }

    let parcel = response
        .parcels
        .into_iter()
        .next()
        .ok_or_else(|| AppError::External("Sendcloud n'a renvoyé aucun colis".into()))?;

    let label_link = parcel
        .documents
        .iter()
        .find(|d| d.document_type.as_deref() == Some("label"))
        .and_then(|d| d.link.clone());

    let label = service::record_label(
        &state.db,
        NewSendcloudLabel {
            shipment_id,
            sendcloud_shipment_id: response.id,
            sendcloud_parcel_id: parcel.id,
            tracking_number: parcel.tracking_number,
            tracking_url: parcel.tracking_url,
            label_link,
            status: parcel
                .status
                .map(|s| s.code)
                .unwrap_or_else(|| "unknown".to_string()),
        },
    )
    .await?;

    service::update_shipment_status(&state.db, shipment_id, shipping::entities::shipment::status::LABELED.to_string())
        .await?;

    Ok(Json(CreateLabelResult {
        label,
        label_file_base64: parcel.label_file,
    }))
}

async fn sync_invoices(
    db: &sea_orm::DatabaseConnection,
    client: &PennylaneClient,
) -> Result<usize, AppError> {
    let mut processed = 0usize;
    let mut cursor: Option<String> = None;

    loop {
        let page = client
            .list_customer_invoices(cursor.as_deref())
            .await
            .map_err(pennylane_err)?;

        for invoice in page.items {
            let Some(customer_ref) = &invoice.customer else {
                continue;
            };
            let customer = client
                .get_customer(customer_ref.id)
                .await
                .map_err(pennylane_err)?;

            let mut lines = Vec::new();
            let mut line_cursor: Option<String> = None;
            loop {
                let line_page = client
                    .list_invoice_lines(invoice.id, line_cursor.as_deref())
                    .await
                    .map_err(pennylane_err)?;

                for line in line_page.items {
                    let Some(product_ref) = &line.product else {
                        continue;
                    };
                    let product_id =
                        nomenclature::service::find_product_by_pennylane_id(db, product_ref.id)
                            .await?
                            .map(|p| p.id);

                    lines.push(NewShipmentLine {
                        pennylane_line_id: line.id,
                        product_id,
                        label: line.label,
                        quantity: line.quantity,
                    });
                }

                if !line_page.has_more {
                    break;
                }
                line_cursor = line_page.next_cursor;
            }

            service::upsert_shipment(
                db,
                NewShipment {
                    pennylane_invoice_id: invoice.id,
                    invoice_number: invoice.invoice_number,
                    pennylane_customer_id: customer.id,
                    customer_name: customer.name,
                    delivery_address: customer.delivery_address.address,
                    delivery_postal_code: customer.delivery_address.postal_code,
                    delivery_city: customer.delivery_address.city,
                    delivery_country: customer.delivery_address.country_alpha2,
                    lines,
                },
            )
            .await?;
            processed += 1;
        }

        if !page.has_more {
            break;
        }
        cursor = page.next_cursor;
    }

    Ok(processed)
}
