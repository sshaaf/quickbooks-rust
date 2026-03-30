pub mod accounts;
pub mod company;
pub mod customers;
pub mod home;
pub mod invoices;
pub mod payments;
pub mod reports;

use axum::response::Html;

fn format_money(amount: Option<f64>) -> String {
    match amount {
        Some(a) => format!("${:.2}", a),
        None => String::from("—"),
    }
}

fn unwrap_or_dash(val: &Option<String>) -> String {
    val.clone().unwrap_or_else(|| String::from("—"))
}

fn render<T: askama::Template>(tmpl: &T) -> Html<String> {
    match tmpl.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!(
            "<h1>Template render error</h1><pre>{}</pre>",
            e
        )),
    }
}

pub fn pagination_url(base: &str, page: u32, per_page: u32) -> String {
    format!("{}?page={}&per_page={}", base, page, per_page)
}

/// Empty string means no link for that direction.
pub fn pagination_prev_next_urls(
    base: &str,
    page: u32,
    per_page: u32,
    total_count: Option<u32>,
) -> (String, String) {
    let prev_url = if page > 1 {
        pagination_url(base, page - 1, per_page)
    } else {
        String::new()
    };
    let total = total_count.unwrap_or(0);
    let next_url = if page * per_page < total {
        pagination_url(base, page + 1, per_page)
    } else {
        String::new()
    };
    (prev_url, next_url)
}
