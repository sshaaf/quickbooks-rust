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

pub struct PaymentView {
    pub id: String,
    pub date: String,
    pub customer_name: String,
    pub amount: String,
    pub deposit_account: String,
}

impl From<&crate::qbo::types::Payment> for PaymentView {
    fn from(p: &crate::qbo::types::Payment) -> Self {
        Self {
            id: unwrap_or_dash(&p.id),
            date: unwrap_or_dash(&p.txn_date),
            customer_name: p
                .customer_ref
                .as_ref()
                .and_then(|r| r.name.clone())
                .unwrap_or_else(|| "—".into()),
            amount: format_money(p.total_amt),
            deposit_account: p
                .deposit_to_account_ref
                .as_ref()
                .and_then(|r| r.name.clone())
                .unwrap_or_else(|| "—".into()),
        }
    }
}

#[derive(Template)]
#[template(path = "payments/list.html")]
struct PaymentsListTemplate {
    active_nav: String,
    connected: bool,
    error_message: String,
    payments: Vec<PaymentView>,
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
        let base_path = "/payments".to_string();
        let (prev_url, next_url) =
            pagination_prev_next_urls(&base_path, 1, 25, None);
        return render(&PaymentsListTemplate {
            active_nav: "payments".into(),
            connected: false,
            error_message: String::new(),
            payments: vec![],
            page: 1,
            total_count: None,
            prev_url,
            next_url,
        });
    }

    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);
    let query_str = queries::list_query("Payment", params.start_position(), params.per_page());

    match qbo.query(&query_str).await {
        Ok(resp) => {
            let qr = resp.query_response.unwrap_or_default();
            let payments: Vec<PaymentView> = qr
                .payment
                .unwrap_or_default()
                .iter()
                .map(PaymentView::from)
                .collect();

            let base_path = "/payments".to_string();
            let page = params.page();
            let per_page = params.per_page();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, page, per_page, qr.total_count);
            render(&PaymentsListTemplate {
                active_nav: "payments".into(),
                connected: true,
                error_message: String::new(),
                payments,
                page,
                total_count: qr.total_count,
                prev_url,
                next_url,
            })
        }
        Err(e) => {
            let base_path = "/payments".to_string();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, 1, 25, None);
            render(&PaymentsListTemplate {
                active_nav: "payments".into(),
                connected: true,
                error_message: format!("{:?}", e),
                payments: vec![],
                page: 1,
                total_count: None,
                prev_url,
                next_url,
            })
        }
    }
}
