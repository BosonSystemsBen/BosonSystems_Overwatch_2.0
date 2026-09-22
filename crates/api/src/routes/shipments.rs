use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use common::AppError;
use pennylane::PennylaneClient;
use sendcloud::types::{
    Address, Integration, Measurement, Order, OrderDetails, OrderItem, OrderStatus,
    PaymentDetails, Price, ShippingDetails, Weight,
};
use serde::{Deserialize, Serialize};
use shipping::service::{self, NewSendcloudOrder, NewShipment, NewShipmentLine};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shipments", get(list))
        .route("/shipments/:id", get(get_one))
        .route("/shipments/:id/status", axum::routing::put(update_status))
        .route(
            "/shipments/:id/send-to-sendcloud",
            axum::routing::post(send_to_sendcloud),
        )
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
    sendcloud_order: Option<shipping::entities::sendcloud_order::Model>,
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ShipmentDetail>, ApiError> {
    let shipment = service::get_shipment(&state.db, id).await?;
    let lines = service::list_shipment_lines(&state.db, id).await?;
    let sendcloud_order = service::get_sendcloud_order_for_shipment(&state.db, id).await?;
    Ok(Json(ShipmentDetail {
        shipment,
        lines,
        sendcloud_order,
    }))
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
struct SendToSendcloudInput {
    weight_kg: f64,
}

/// Pushes the shipment to Sendcloud as an order for human review. This never
/// creates a label or incurs a carrier charge — a person completes customs
/// details (if any) and creates the label from the Sendcloud panel.
async fn send_to_sendcloud(
    State(state): State<AppState>,
    Path(shipment_id): Path<Uuid>,
    Json(input): Json<SendToSendcloudInput>,
) -> Result<Json<shipping::entities::sendcloud_order::Model>, ApiError> {
    let (client, integration_id) = state.sendcloud.as_ref().ok_or_else(|| {
        AppError::External(
            "Sendcloud n'est pas configuré (SENDCLOUD_PUBLIC_KEY/PRIVATE_KEY/INTEGRATION_ID manquants)".into(),
        )
    })?;

    if let Some(existing) = service::get_sendcloud_order_for_shipment(&state.db, shipment_id).await? {
        return Ok(Json(existing));
    }

    let shipment = service::get_shipment(&state.db, shipment_id).await?;
    let lines = service::list_shipment_lines(&state.db, shipment_id).await?;

    let order_items: Vec<OrderItem> = lines
        .iter()
        .map(|l| {
            let amount = l
                .amount_eur
                .as_deref()
                .and_then(|a| a.parse::<f64>().ok())
                .unwrap_or(0.0);
            let quantity = l.quantity.parse::<f64>().unwrap_or(1.0).round().max(1.0) as i32;
            OrderItem {
                name: l.label.clone(),
                quantity,
                total_price: Price::eur(amount),
            }
        })
        .collect();

    let total_price_eur: f64 = order_items.iter().map(|i| i.total_price.value).sum();

    let order = Order {
        order_id: shipment.id.to_string(),
        order_number: shipment.invoice_number.clone(),
        order_details: OrderDetails {
            integration: Integration { id: *integration_id },
            status: OrderStatus {
                code: "to_review",
                message: "À valider (douane) avant expédition",
            },
            order_created_at: shipment.created_at.to_rfc3339(),
            order_items,
        },
        payment_details: PaymentDetails {
            total_price: Price::eur(total_price_eur),
            status: OrderStatus {
                code: "n/a",
                message: "Statut de paiement non suivi par Overwatch",
            },
        },
        shipping_address: Address {
            name: shipment.customer_name.clone(),
            address_line_1: shipment.delivery_address.clone(),
            postal_code: shipment.delivery_postal_code.clone(),
            city: shipment.delivery_city.clone(),
            country_code: shipment.delivery_country.clone(),
            email: None,
            phone_number: None,
        },
        shipping_details: ShippingDetails {
            measurement: Measurement {
                weight: Weight::kg(input.weight_kg),
            },
        },
    };

    let response = client.create_order(&order).await.map_err(sendcloud_err)?;

    let sendcloud_order = service::record_sendcloud_order(
        &state.db,
        NewSendcloudOrder {
            shipment_id,
            sendcloud_order_id: response.id,
            order_number: response.order_number,
        },
    )
    .await?;

    service::update_shipment_status(
        &state.db,
        shipment_id,
        shipping::entities::shipment::status::SENT_TO_SENDCLOUD.to_string(),
    )
    .await?;

    Ok(Json(sendcloud_order))
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
                        amount_eur: Some(line.amount),
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
