use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::error::AppError;
use crate::models::{ApiResponse, PaginationMeta, PaginationParams};
use crate::qbo::client::QboClient;
use crate::qbo::queries;
use crate::qbo::types::Invoice;
use crate::AppState;

pub async fn list_invoices(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<Vec<Invoice>>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let query_str = queries::list_query("Invoice", params.start_position(), params.per_page());
    let resp = qbo.query(&query_str).await?;

    let qr = resp.query_response.unwrap_or_default();
    let invoices = qr.invoice.unwrap_or_default();

    Ok(Json(ApiResponse {
        data: invoices,
        meta: Some(PaginationMeta {
            page: params.page(),
            per_page: params.per_page(),
            total_count: qr.total_count,
        }),
    }))
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Invoice>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let value = qbo.get_entity(&format!("invoice/{id}")).await?;

    let invoice: Invoice = serde_json::from_value(
        value
            .get("Invoice")
            .cloned()
            .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?,
    )
    .map_err(|e| AppError::Internal(format!("Parse error: {e}")))?;

    Ok(Json(ApiResponse {
        data: invoice,
        meta: None,
    }))
}
