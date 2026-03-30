use reqwest::Client;

use crate::auth::oauth;
use crate::auth::store::TokenStore;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::qbo::types::{QboQueryResponse, QboReport};

pub struct QboClient<'a> {
    config: &'a AppConfig,
    http: &'a Client,
    store: &'a TokenStore,
}

impl<'a> QboClient<'a> {
    pub fn new(config: &'a AppConfig, http: &'a Client, store: &'a TokenStore) -> Self {
        Self {
            config,
            http,
            store,
        }
    }

    async fn ensure_valid_token(&self) -> Result<(String, String), AppError> {
        let tokens = self
            .store
            .get_tokens()
            .await
            .ok_or(AppError::NotConnected)?;

        if tokens.needs_refresh() {
            let refreshed =
                oauth::refresh_tokens(self.config, self.http, &tokens.refresh_token).await?;
            let new_data =
                oauth::token_response_to_data(refreshed, tokens.realm_id.clone());
            let access = new_data.access_token.clone();
            let realm = new_data.realm_id.clone();
            self.store.set_tokens(new_data).await;
            Ok((access, realm))
        } else {
            Ok((tokens.access_token, tokens.realm_id))
        }
    }

    fn base_url(&self, realm_id: &str) -> String {
        format!("{}/{}", self.config.qbo_base_url, realm_id)
    }

    pub async fn query(&self, query_str: &str) -> Result<QboQueryResponse, AppError> {
        let (token, realm) = self.ensure_valid_token().await?;
        let url = format!("{}/query", self.base_url(&realm));

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .query(&[("query", query_str)])
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::TokenExpired);
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::QboRequest(format!("QBO query failed: {body}")));
        }

        resp.json::<QboQueryResponse>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse QBO response: {e}")))
    }

    pub async fn get_entity(&self, entity_path: &str) -> Result<serde_json::Value, AppError> {
        let (token, realm) = self.ensure_valid_token().await?;
        let url = format!("{}/{}", self.base_url(&realm), entity_path);

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::TokenExpired);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::NOT_FOUND
                || body.contains("Object Not Found")
            {
                return Err(AppError::NotFound(format!(
                    "Entity not found: {entity_path}"
                )));
            }
            return Err(AppError::QboRequest(format!(
                "QBO request failed ({status}): {body}"
            )));
        }

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse QBO response: {e}")))
    }

    pub async fn get_report(
        &self,
        report_name: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<QboReport, AppError> {
        let (token, realm) = self.ensure_valid_token().await?;
        let url = format!("{}/reports/{}", self.base_url(&realm), report_name);

        let mut req = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "application/json");

        if let Some(sd) = start_date {
            req = req.query(&[("start_date", sd)]);
        }
        if let Some(ed) = end_date {
            req = req.query(&[("end_date", ed)]);
        }

        let resp = req.send().await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::TokenExpired);
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::QboRequest(format!(
                "QBO report request failed: {body}"
            )));
        }

        resp.json::<QboReport>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse report response: {e}")))
    }
}
