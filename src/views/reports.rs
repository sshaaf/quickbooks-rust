use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};

use crate::models::ReportParams;
use crate::qbo::client::QboClient;
use crate::views::render;
use crate::AppState;

pub struct ReportRowView {
    pub cells: Vec<String>,
    pub is_section_header: bool,
    pub is_total: bool,
    pub depth: usize,
}

#[derive(Template)]
#[template(path = "reports/report.html")]
struct ReportTemplate {
    active_nav: String,
    connected: bool,
    error_message: String,
    report_title: String,
    period: String,
    start_date: String,
    end_date: String,
    columns: Vec<String>,
    rows: Vec<ReportRowView>,
}

fn extract_columns(report: &serde_json::Value) -> Vec<String> {
    report
        .get("Columns")
        .and_then(|c| c.get("Column"))
        .and_then(|c| c.as_array())
        .map(|cols| {
            cols.iter()
                .map(|c| {
                    c.get("ColTitle")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn flatten_rows(rows_val: &serde_json::Value, depth: usize, out: &mut Vec<ReportRowView>) {
    let rows = match rows_val.get("Row").and_then(|r| r.as_array()) {
        Some(r) => r,
        None => return,
    };

    for row in rows {
        let row_type = row
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Data");

        if let Some(header) = row.get("Header") {
            let cells = extract_col_data(header);
            if !cells.is_empty() && cells.iter().any(|c| !c.is_empty()) {
                out.push(ReportRowView {
                    cells,
                    is_section_header: true,
                    is_total: false,
                    depth,
                });
            }
        }

        if let Some(col_data) = row.get("ColData") {
            let cells: Vec<String> = col_data
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|c| {
                            c.get("value")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string()
                        })
                        .collect()
                })
                .unwrap_or_default();

            let is_total = row_type == "Section" && row.get("Summary").is_some();
            if !cells.is_empty() {
                out.push(ReportRowView {
                    cells,
                    is_section_header: false,
                    is_total,
                    depth,
                });
            }
        }

        if let Some(inner_rows) = row.get("Rows") {
            flatten_rows(inner_rows, depth + 1, out);
        }

        if let Some(summary) = row.get("Summary") {
            let cells = extract_col_data(summary);
            if !cells.is_empty() {
                out.push(ReportRowView {
                    cells,
                    is_section_header: false,
                    is_total: true,
                    depth,
                });
            }
        }
    }
}

fn extract_col_data(node: &serde_json::Value) -> Vec<String> {
    node.get("ColData")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .map(|c| {
                    c.get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn extract_period(report: &serde_json::Value) -> String {
    let header = match report.get("Header") {
        Some(h) => h,
        None => return String::new(),
    };
    let start = header
        .get("StartPeriod")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let end = header
        .get("EndPeriod")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if start.is_empty() && end.is_empty() {
        String::new()
    } else {
        format!("{} to {}", start, end)
    }
}

async fn render_report(
    state: &AppState,
    report_name: &str,
    display_title: &str,
    nav: &str,
    params: &ReportParams,
) -> Html<String> {
    let connected = state.token_store.is_connected().await;
    let empty = || ReportTemplate {
        active_nav: nav.into(),
        connected,
        error_message: String::new(),
        report_title: display_title.into(),
        period: String::new(),
        start_date: params.start_date.clone().unwrap_or_default(),
        end_date: params.end_date.clone().unwrap_or_default(),
        columns: vec![],
        rows: vec![],
    };

    if !connected {
        return render(&empty());
    }

    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    match qbo
        .get_report(
            report_name,
            params.start_date.as_deref(),
            params.end_date.as_deref(),
        )
        .await
    {
        Ok(report) => {
            let raw = serde_json::to_value(&report).unwrap_or_default();
            let columns = extract_columns(&raw);
            let period = extract_period(&raw);
            let mut rows = Vec::new();
            if let Some(rows_val) = raw.get("Rows") {
                flatten_rows(rows_val, 0, &mut rows);
            }

            render(&ReportTemplate {
                active_nav: nav.into(),
                connected: true,
                error_message: String::new(),
                report_title: display_title.into(),
                period,
                start_date: params.start_date.clone().unwrap_or_default(),
                end_date: params.end_date.clone().unwrap_or_default(),
                columns,
                rows,
            })
        }
        Err(e) => {
            let mut tmpl = empty();
            tmpl.error_message = format!("{:?}", e);
            render(&tmpl)
        }
    }
}

pub async fn profit_and_loss_page(
    State(state): State<AppState>,
    Query(params): Query<ReportParams>,
) -> Html<String> {
    render_report(&state, "ProfitAndLoss", "Profit & Loss", "profit-and-loss", &params).await
}

pub async fn balance_sheet_page(
    State(state): State<AppState>,
    Query(params): Query<ReportParams>,
) -> Html<String> {
    render_report(&state, "BalanceSheet", "Balance Sheet", "balance-sheet", &params).await
}
