use axum::{
    extract::{Query, State},
    Json,
};

use crate::error::AppError;
use crate::models::{ApiResponse, ReportParams};
use crate::qbo::client::QboClient;
use crate::qbo::types::QboReport;
use crate::AppState;

pub async fn profit_and_loss(
    State(state): State<AppState>,
    Query(params): Query<ReportParams>,
) -> Result<Json<ApiResponse<QboReport>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let report = qbo
        .get_report(
            "ProfitAndLoss",
            params.start_date.as_deref(),
            params.end_date.as_deref(),
        )
        .await?;

    Ok(Json(ApiResponse {
        data: report,
        meta: None,
    }))
}

pub async fn balance_sheet(
    State(state): State<AppState>,
    Query(params): Query<ReportParams>,
) -> Result<Json<ApiResponse<QboReport>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let report = qbo
        .get_report(
            "BalanceSheet",
            params.start_date.as_deref(),
            params.end_date.as_deref(),
        )
        .await?;

    Ok(Json(ApiResponse {
        data: report,
        meta: None,
    }))
}
