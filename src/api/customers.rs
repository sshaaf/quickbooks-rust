use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::error::AppError;
use crate::models::{ApiResponse, PaginationMeta, PaginationParams};
use crate::qbo::client::QboClient;
use crate::qbo::queries;
use crate::qbo::types::Customer;
use crate::AppState;

pub async fn list_customers(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<Vec<Customer>>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let query_str = queries::list_query("Customer", params.start_position(), params.per_page());
    let resp = qbo.query(&query_str).await?;

    let qr = resp.query_response.unwrap_or_default();
    let customers = qr.customer.unwrap_or_default();

    Ok(Json(ApiResponse {
        data: customers,
        meta: Some(PaginationMeta {
            page: params.page(),
            per_page: params.per_page(),
            total_count: qr.total_count,
        }),
    }))
}

pub async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Customer>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let value = qbo.get_entity(&format!("customer/{id}")).await?;

    let customer: Customer = serde_json::from_value(
        value
            .get("Customer")
            .cloned()
            .ok_or_else(|| AppError::NotFound("Customer not found".into()))?,
    )
    .map_err(|e| AppError::Internal(format!("Parse error: {e}")))?;

    Ok(Json(ApiResponse {
        data: customer,
        meta: None,
    }))
}
