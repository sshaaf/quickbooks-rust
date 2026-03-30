use axum::{
    extract::{Query, State},
    Json,
};

use crate::error::AppError;
use crate::models::{ApiResponse, PaginationMeta, PaginationParams};
use crate::qbo::client::QboClient;
use crate::qbo::queries;
use crate::qbo::types::Account;
use crate::AppState;

pub async fn list_accounts(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ApiResponse<Vec<Account>>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let query_str = queries::list_query("Account", params.start_position(), params.per_page());
    let resp = qbo.query(&query_str).await?;

    let qr = resp.query_response.unwrap_or_default();
    let accounts = qr.account.unwrap_or_default();

    Ok(Json(ApiResponse {
        data: accounts,
        meta: Some(PaginationMeta {
            page: params.page(),
            per_page: params.per_page(),
            total_count: qr.total_count,
        }),
    }))
}
