use askama::Template;
use axum::{extract::State, response::Html};

use crate::views::render;
use crate::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    active_nav: String,
    connected: bool,
    error_message: String,
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let connected = state.token_store.is_connected().await;
    render(&IndexTemplate {
        active_nav: "home".into(),
        connected,
        error_message: String::new(),
    })
}
