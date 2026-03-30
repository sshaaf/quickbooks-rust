mod api;
mod auth;
mod config;
mod error;
mod models;
mod qbo;
mod views;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub config: config::AppConfig,
    pub http_client: reqwest::Client,
    pub token_store: auth::store::TokenStore,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::AppConfig::from_env().expect("Failed to load configuration");

    let state = AppState {
        config: config.clone(),
        http_client: reqwest::Client::new(),
        token_store: auth::store::TokenStore::new(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // JSON API routes (unchanged)
    let auth_routes = Router::new()
        .route("/login", get(auth::handlers::login))
        .route("/callback", get(auth::handlers::callback))
        .route("/status", get(auth::handlers::status))
        .route("/disconnect", post(auth::handlers::disconnect));

    let api_routes = Router::new()
        .route("/customers", get(api::customers::list_customers))
        .route("/customers/{id}", get(api::customers::get_customer))
        .route("/invoices", get(api::invoices::list_invoices))
        .route("/invoices/{id}", get(api::invoices::get_invoice))
        .route("/payments", get(api::payments::list_payments))
        .route("/accounts", get(api::accounts::list_accounts))
        .route("/company", get(api::company::get_company))
        .route("/profit-and-loss", get(api::reports::profit_and_loss))
        .route("/balance-sheet", get(api::reports::balance_sheet));

    // HTML view routes
    let view_routes = Router::new()
        .route("/", get(views::home::index))
        .route("/customers", get(views::customers::list_page))
        .route("/customers/{id}", get(views::customers::detail_fragment))
        .route("/invoices", get(views::invoices::list_page))
        .route("/invoices/{id}", get(views::invoices::detail_fragment))
        .route("/payments", get(views::payments::list_page))
        .route("/accounts", get(views::accounts::list_page))
        .route("/company", get(views::company::page))
        .route("/reports/profit-and-loss", get(views::reports::profit_and_loss_page))
        .route("/reports/balance-sheet", get(views::reports::balance_sheet_page));

    let app = Router::new()
        .merge(view_routes)
        .nest("/api/auth", auth_routes)
        .nest("/api", api_routes)
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("Vega backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
