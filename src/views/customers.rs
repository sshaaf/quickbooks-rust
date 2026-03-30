use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::Html,
};

use crate::models::PaginationParams;
use crate::qbo::client::QboClient;
use crate::qbo::queries;
use crate::views::{format_money, pagination_prev_next_urls, render, unwrap_or_dash};
use crate::AppState;

// -- View models --

pub struct CustomerView {
    pub id: String,
    pub display_name: String,
    pub company_name: String,
    pub email: String,
    pub phone: String,
    pub balance: String,
    pub active: bool,
}

impl From<&crate::qbo::types::Customer> for CustomerView {
    fn from(c: &crate::qbo::types::Customer) -> Self {
        Self {
            id: unwrap_or_dash(&c.id),
            display_name: unwrap_or_dash(&c.display_name),
            company_name: unwrap_or_dash(&c.company_name),
            email: c
                .primary_email_addr
                .as_ref()
                .and_then(|e| e.address.clone())
                .unwrap_or_else(|| "—".into()),
            phone: c
                .primary_phone
                .as_ref()
                .and_then(|p| p.free_form_number.clone())
                .unwrap_or_else(|| "—".into()),
            balance: format_money(c.balance),
            active: c.active.unwrap_or(true),
        }
    }
}

// -- Templates --

#[derive(Template)]
#[template(path = "customers/list.html")]
struct CustomersListTemplate {
    active_nav: String,
    connected: bool,
    error_message: String,
    customers: Vec<CustomerView>,
    page: u32,
    total_count: Option<u32>,
    prev_url: String,
    next_url: String,
}

#[derive(Template)]
#[template(path = "customers/detail.html")]
struct CustomerDetailTemplate {
    customer: CustomerView,
}

// -- Handlers --

pub async fn list_page(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let connected = state.token_store.is_connected().await;
    if !connected {
        let base_path = "/customers".to_string();
        let (prev_url, next_url) =
            pagination_prev_next_urls(&base_path, 1, 25, None);
        return render(&CustomersListTemplate {
            active_nav: "customers".into(),
            connected: false,
            error_message: String::new(),
            customers: vec![],
            page: 1,
            total_count: None,
            prev_url,
            next_url,
        });
    }

    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);
    let query_str = queries::list_query("Customer", params.start_position(), params.per_page());

    match qbo.query(&query_str).await {
        Ok(resp) => {
            let qr = resp.query_response.unwrap_or_default();
            let customers: Vec<CustomerView> = qr
                .customer
                .unwrap_or_default()
                .iter()
                .map(CustomerView::from)
                .collect();

            let base_path = "/customers".to_string();
            let page = params.page();
            let per_page = params.per_page();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, page, per_page, qr.total_count);
            render(&CustomersListTemplate {
                active_nav: "customers".into(),
                connected: true,
                error_message: String::new(),
                customers,
                page,
                total_count: qr.total_count,
                prev_url,
                next_url,
            })
        }
        Err(e) => {
            let base_path = "/customers".to_string();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, 1, 25, None);
            render(&CustomersListTemplate {
                active_nav: "customers".into(),
                connected: true,
                error_message: format!("{:?}", e),
                customers: vec![],
                page: 1,
                total_count: None,
                prev_url,
                next_url,
            })
        }
    }
}

pub async fn detail_fragment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Html<String> {
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    match qbo.get_entity(&format!("customer/{id}")).await {
        Ok(value) => {
            if let Some(raw) = value.get("Customer") {
                if let Ok(c) = serde_json::from_value::<crate::qbo::types::Customer>(raw.clone()) {
                    return render(&CustomerDetailTemplate {
                        customer: CustomerView::from(&c),
                    });
                }
            }
            Html("<p class='text-sm text-red-500'>Failed to parse customer data.</p>".into())
        }
        Err(e) => Html(format!(
            "<p class='text-sm text-red-500'>Error: {:?}</p>",
            e
        )),
    }
}
