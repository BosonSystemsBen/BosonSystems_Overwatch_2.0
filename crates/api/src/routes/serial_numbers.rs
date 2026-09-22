use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use nomenclature::service::{self, ListSerialNumbersFilter, NewSerialNumber};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/serial-numbers", get(list).post(create))
        .route("/serial-numbers/:id", axum::routing::delete(remove))
        .route("/serial-numbers/:id/assign", axum::routing::post(assign))
        .route("/serial-numbers/detect", axum::routing::post(detect))
}

#[derive(Debug, Deserialize)]
struct ListParams {
    product_id: Option<Uuid>,
    status: Option<String>,
    value: Option<String>,
    shipment_line_id: Option<Uuid>,
}

async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<nomenclature::entities::serial_number::Model>>, ApiError> {
    let items = service::list_serial_numbers(
        &state.db,
        ListSerialNumbersFilter {
            product_id: params.product_id,
            status: params.status,
            value: params.value,
            shipment_line_id: params.shipment_line_id,
        },
    )
    .await?;
    Ok(Json(items))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<NewSerialNumber>,
) -> Result<Json<nomenclature::entities::serial_number::Model>, ApiError> {
    let item = service::create_serial_number(&state.db, input).await?;
    Ok(Json(item))
}

async fn remove(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<(), ApiError> {
    service::delete_serial_number(&state.db, id).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct AssignInput {
    assigned_to: String,
}

async fn assign(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<AssignInput>,
) -> Result<Json<nomenclature::entities::serial_number::Model>, ApiError> {
    let item = service::assign_serial_number(&state.db, id, input.assigned_to).await?;
    Ok(Json(item))
}

#[derive(Debug, Deserialize)]
struct DetectInput {
    value: String,
}

async fn detect(
    State(state): State<AppState>,
    Json(input): Json<DetectInput>,
) -> Result<Json<Vec<nomenclature::entities::product::Model>>, ApiError> {
    let matches = service::detect_product_by_serial(&state.db, &input.value).await?;
    Ok(Json(matches))
}
