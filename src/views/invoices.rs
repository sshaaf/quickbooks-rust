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

pub struct InvoiceView {
    pub id: String,
    pub doc_number: String,
    pub date: String,
    pub due_date: String,
    pub customer_name: String,
    pub total: String,
    pub balance: String,
    pub lines: Vec<InvoiceLineView>,
}

pub struct InvoiceLineView {
    pub description: String,
    pub amount: String,
}

impl From<&crate::qbo::types::Invoice> for InvoiceView {
    fn from(inv: &crate::qbo::types::Invoice) -> Self {
        Self {
            id: unwrap_or_dash(&inv.id),
            doc_number: unwrap_or_dash(&inv.doc_number),
            date: unwrap_or_dash(&inv.txn_date),
            due_date: unwrap_or_dash(&inv.due_date),
            customer_name: inv
                .customer_ref
                .as_ref()
                .and_then(|r| r.name.clone())
                .unwrap_or_else(|| "—".into()),
            total: format_money(inv.total_amt),
            balance: format_money(inv.balance),
            lines: inv
                .line
                .as_ref()
                .map(|lines| {
                    lines
                        .iter()
                        .filter(|l| l.detail_type.as_deref() != Some("SubTotalLineDetail"))
                        .map(|l| InvoiceLineView {
                            description: unwrap_or_dash(&l.description),
                            amount: format_money(l.amount),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

// -- Templates --

#[derive(Template)]
#[template(path = "invoices/list.html")]
struct InvoicesListTemplate {
    active_nav: String,
    connected: bool,
    error_message: String,
    invoices: Vec<InvoiceView>,
    page: u32,
    total_count: Option<u32>,
    prev_url: String,
    next_url: String,
}

#[derive(Template)]
#[template(path = "invoices/detail.html")]
struct InvoiceDetailTemplate {
    invoice: InvoiceView,
}

// -- Handlers --

pub async fn list_page(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let connected = state.token_store.is_connected().await;
    if !connected {
        let base_path = "/invoices".to_string();
        let (prev_url, next_url) =
            pagination_prev_next_urls(&base_path, 1, 25, None);
        return render(&InvoicesListTemplate {
            active_nav: "invoices".into(),
            connected: false,
            error_message: String::new(),
            invoices: vec![],
            page: 1,
            total_count: None,
            prev_url,
            next_url,
        });
    }

    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);
    let query_str = queries::list_query("Invoice", params.start_position(), params.per_page());

    match qbo.query(&query_str).await {
        Ok(resp) => {
            let qr = resp.query_response.unwrap_or_default();
            let invoices: Vec<InvoiceView> = qr
                .invoice
                .unwrap_or_default()
                .iter()
                .map(InvoiceView::from)
                .collect();

            let base_path = "/invoices".to_string();
            let page = params.page();
            let per_page = params.per_page();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, page, per_page, qr.total_count);
            render(&InvoicesListTemplate {
                active_nav: "invoices".into(),
                connected: true,
                error_message: String::new(),
                invoices,
                page,
                total_count: qr.total_count,
                prev_url,
                next_url,
            })
        }
        Err(e) => {
            let base_path = "/invoices".to_string();
            let (prev_url, next_url) =
                pagination_prev_next_urls(&base_path, 1, 25, None);
            render(&InvoicesListTemplate {
                active_nav: "invoices".into(),
                connected: true,
                error_message: format!("{:?}", e),
                invoices: vec![],
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

    match qbo.get_entity(&format!("invoice/{id}")).await {
        Ok(value) => {
            if let Some(raw) = value.get("Invoice") {
                if let Ok(inv) = serde_json::from_value::<crate::qbo::types::Invoice>(raw.clone())
                {
                    return render(&InvoiceDetailTemplate {
                        invoice: InvoiceView::from(&inv),
                    });
                }
            }
            Html("<p class='text-sm text-red-500'>Failed to parse invoice data.</p>".into())
        }
        Err(e) => Html(format!(
            "<p class='text-sm text-red-500'>Error: {:?}</p>",
            e
        )),
    }
}
