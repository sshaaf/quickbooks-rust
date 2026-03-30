use base64::Engine;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::auth::store::TokenData;
use crate::config::AppConfig;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    #[allow(dead_code)]
    pub token_type: String,
}

pub fn build_auth_url(config: &AppConfig, state: &str) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=com.intuit.quickbooks.accounting&state={}",
        config.qbo_auth_url,
        urlencoding::encode(&config.qbo_client_id),
        urlencoding::encode(&config.qbo_redirect_uri),
        urlencoding::encode(state),
    )
}

fn basic_auth_header(config: &AppConfig) -> String {
    let credentials = format!("{}:{}", config.qbo_client_id, config.qbo_client_secret);
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
    format!("Basic {encoded}")
}

pub async fn exchange_code(
    config: &AppConfig,
    http: &Client,
    code: &str,
) -> Result<TokenResponse, AppError> {
    let resp = http
        .post(&config.qbo_token_url)
        .header("Authorization", basic_auth_header(config))
        .header("Accept", "application/json")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", config.qbo_redirect_uri.as_str()),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::QboRequest(format!(
            "Token exchange failed: {body}"
        )));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse token response: {e}")))
}

pub async fn refresh_tokens(
    config: &AppConfig,
    http: &Client,
    refresh_token: &str,
) -> Result<TokenResponse, AppError> {
    let resp = http
        .post(&config.qbo_token_url)
        .header("Authorization", basic_auth_header(config))
        .header("Accept", "application/json")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::QboRequest(format!(
            "Token refresh failed: {body}"
        )));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse refresh response: {e}")))
}

pub fn token_response_to_data(resp: TokenResponse, realm_id: String) -> TokenData {
    TokenData {
        realm_id,
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at: Utc::now() + Duration::seconds(resp.expires_in),
    }
}
