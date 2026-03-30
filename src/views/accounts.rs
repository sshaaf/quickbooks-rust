use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};

use crate::models::PaginationParams;
use crate::qbo::client::QboClient;
use crate::qbo::queries;
use crate::views::{format_money, pagination_prev_next_urls, render, unwrap_or_dash};
use crate::AppState;

#[allow(dead_code)]
pub struct AccountView {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub sub_type: String,
    pub classification: String,
    pub balance: String,
    pub active: bool,
}

impl From<&crate::qbo::types::Account> for AccountView {
    fn from(a: &crate::qbo::types::Account) -> Self {
        Self {
            id: unwrap_or_dash(&a.id),
            name: unwrap_or_dash(&a.name),
            account_type: unwrap_or_dash(&a.account_type),
            sub_type: unwrap_or_dash(&a.account_sub_type),
            classification: unwrap_or_dash(&a.classification),
            balance: format_money(a.current_balance),
            active: a.active.unwrap_or(true),
        }
    }
}

#[derive(Template)]
#[template(path = "accounts/list.html")]
struct AccountsListTemplate {
    active_nav: String,
    connected: bool,
    error_message: String,
    accounts: Vec<AccountView>,
    page: u32,
    total_count: Option<u32>,
    prev_url: String,
    next_url: String,
}

pub async fn list_page(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let connected = state.token_store.is_connected().await;
    if !connected {
        let base_path = "/accounts".to_string();
        let (prev_url, next_url) =
            pagination_prev_next_urls(&base_path, 1, 25, None);
        return render(&AccountsListTemplate {
            active_nav: "accounts".into(),
            connected: false,
            error_message: String::new(),
            accounts: vec![],
            page: 1,
            total_count: None,
            prev_url,
            next_url,
        });
    }

    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);
    let query_str = queries::list_query("Account", params.start_position(), params.per_page());

    match qbo.query(&query_str).await {
        Ok(resp) => {
            let qr = resp.query_response.unwrap_or_default();
            let accounts: Vec<AccountView> = qr
                .account
                .unwrap_or_default()
                .iter()
                .map(AccountView::from)
                .collect();

            let base_path = "/accounts".to_string();
            let page = params.page();
            let per_page = params.per_page();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, page, per_page, qr.total_count);
            render(&AccountsListTemplate {
                active_nav: "accounts".into(),
                connected: true,
                error_message: String::new(),
                accounts,
                page,
                total_count: qr.total_count,
                prev_url,
                next_url,
            })
        }
        Err(e) => {
            let base_path = "/accounts".to_string();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, 1, 25, None);
            render(&AccountsListTemplate {
                active_nav: "accounts".into(),
                connected: true,
                error_message: format!("{:?}", e),
                accounts: vec![],
                page: 1,
                total_count: None,
                prev_url,
                next_url,
            })
        }
    }
}
