use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

use crate::auth::oauth;
use crate::error::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
    #[serde(rename = "realmId")]
    pub realm_id: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatus {
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm_id: Option<String>,
}

pub async fn login(State(state): State<AppState>) -> impl IntoResponse {
    let csrf_state = uuid::Uuid::new_v4().to_string();
    state.token_store.store_state(csrf_state.clone()).await;

    let url = oauth::build_auth_url(&state.config, &csrf_state);

    (StatusCode::FOUND, [(header::LOCATION, url)])
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    if !state.token_store.validate_state(&params.state).await {
        return Err(AppError::Internal("Invalid OAuth state parameter".into()));
    }

    let token_resp =
        oauth::exchange_code(&state.config, &state.http_client, &params.code).await?;

    let token_data = oauth::token_response_to_data(token_resp, params.realm_id);
    state.token_store.set_tokens(token_data).await;

    Ok((StatusCode::FOUND, [(header::LOCATION, "/")]))
}

pub async fn status(State(state): State<AppState>) -> Json<AuthStatus> {
    let tokens = state.token_store.get_tokens().await;
    Json(AuthStatus {
        connected: tokens.is_some(),
        realm_id: tokens.map(|t| t.realm_id),
    })
}

pub async fn disconnect(State(state): State<AppState>) -> StatusCode {
    state.token_store.clear_tokens().await;
    StatusCode::OK
}
