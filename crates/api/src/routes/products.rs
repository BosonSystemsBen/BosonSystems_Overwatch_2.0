use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use nomenclature::service::{self, NewProduct, UpdateProduct};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/products", get(list).post(create))
        .route("/products/:id", get(get_one).put(update).delete(remove))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<nomenclature::entities::product::Model>>, ApiError> {
    let products = service::list_products(&state.db).await?;
    Ok(Json(products))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<NewProduct>,
) -> Result<Json<nomenclature::entities::product::Model>, ApiError> {
    let product = service::create_product(&state.db, input).await?;
    Ok(Json(product))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<nomenclature::entities::product::Model>, ApiError> {
    let product = service::get_product(&state.db, id).await?;
    Ok(Json(product))
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProduct>,
) -> Result<Json<nomenclature::entities::product::Model>, ApiError> {
    let product = service::update_product(&state.db, id, input).await?;
    Ok(Json(product))
}

async fn remove(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<(), ApiError> {
    service::delete_product(&state.db, id).await?;
    Ok(())
}
