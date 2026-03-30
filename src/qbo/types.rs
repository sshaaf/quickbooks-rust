use serde::{Deserialize, Serialize};

// -- Shared reference types --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameRef {
    pub value: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyRef {
    pub value: Option<String>,
    pub name: Option<String>,
}

// -- Customer --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Customer {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub company_name: Option<String>,
    pub primary_email_addr: Option<EmailAddr>,
    pub primary_phone: Option<PhoneNumber>,
    pub balance: Option<f64>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EmailAddr {
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhoneNumber {
    pub free_form_number: Option<String>,
}

// -- Invoice --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Invoice {
    pub id: Option<String>,
    pub doc_number: Option<String>,
    pub txn_date: Option<String>,
    pub due_date: Option<String>,
    pub total_amt: Option<f64>,
    pub balance: Option<f64>,
    pub customer_ref: Option<NameRef>,
    pub currency_ref: Option<CurrencyRef>,
    pub line: Option<Vec<InvoiceLine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoiceLine {
    pub id: Option<String>,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub detail_type: Option<String>,
}

// -- Payment --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payment {
    pub id: Option<String>,
    pub txn_date: Option<String>,
    pub total_amt: Option<f64>,
    pub customer_ref: Option<NameRef>,
    pub currency_ref: Option<CurrencyRef>,
    pub deposit_to_account_ref: Option<NameRef>,
}

// -- Account --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Account {
    pub id: Option<String>,
    pub name: Option<String>,
    pub account_type: Option<String>,
    pub account_sub_type: Option<String>,
    pub current_balance: Option<f64>,
    pub active: Option<bool>,
    pub classification: Option<String>,
}

// -- CompanyInfo --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompanyInfo {
    pub id: Option<String>,
    pub company_name: Option<String>,
    pub legal_name: Option<String>,
    pub country: Option<String>,
    pub fiscal_year_start_month: Option<String>,
}

// -- Generic query response wrappers from QBO --

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QboQueryResponse {
    pub query_response: Option<QueryResponseInner>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct QueryResponseInner {
    pub total_count: Option<u32>,
    pub start_position: Option<u32>,
    pub max_results: Option<u32>,

    #[serde(default)]
    pub customer: Option<Vec<Customer>>,
    #[serde(default)]
    pub invoice: Option<Vec<Invoice>>,
    #[serde(default)]
    pub payment: Option<Vec<Payment>>,
    #[serde(default)]
    pub account: Option<Vec<Account>>,
    #[serde(default)]
    pub company_info: Option<Vec<CompanyInfo>>,
}

// -- Single-entity read response --

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct QboSingleCustomer {
    pub customer: Customer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct QboSingleInvoice {
    pub invoice: Invoice,
}

// -- Report response (flexible) --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QboReport {
    pub header: Option<serde_json::Value>,
    pub columns: Option<serde_json::Value>,
    pub rows: Option<serde_json::Value>,
}
