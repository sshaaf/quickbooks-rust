use axum::{extract::State, Json};

use crate::error::AppError;
use crate::models::ApiResponse;
use crate::qbo::client::QboClient;
use crate::qbo::types::CompanyInfo;
use crate::AppState;

pub async fn get_company(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<CompanyInfo>>, AppError> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    let tokens = state
        .token_store
        .get_tokens()
        .await
        .ok_or(AppError::NotConnected)?;

    let value = qbo
        .get_entity(&format!("companyinfo/{}", tokens.realm_id))
        .await?;

    let info: CompanyInfo = serde_json::from_value(
        value
            .get("CompanyInfo")
            .cloned()
            .ok_or_else(|| AppError::Internal("Missing CompanyInfo in response".into()))?,
    )
    .map_err(|e| AppError::Internal(format!("Parse error: {e}")))?;

    Ok(Json(ApiResponse {
        data: info,
        meta: None,
    }))
}
