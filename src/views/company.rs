use askama::Template;
use axum::{extract::State, response::Html};

use crate::qbo::client::QboClient;
use crate::views::{render, unwrap_or_dash};
use crate::AppState;

#[derive(Template)]
#[template(path = "company.html")]
struct CompanyTemplate {
    active_nav: String,
    connected: bool,
    error_message: String,
    company_name: String,
    legal_name: String,
    country: String,
    fiscal_year_start: String,
}

pub async fn page(State(state): State<AppState>) -> Html<String> {
    let connected = state.token_store.is_connected().await;
    if !connected {
        return render(&CompanyTemplate {
            active_nav: "company".into(),
            connected: false,
            error_message: String::new(),
            company_name: String::new(),
            legal_name: String::new(),
            country: String::new(),
            fiscal_year_start: String::new(),
        });
    }

    let tokens = state.token_store.get_tokens().await.unwrap();
    let qbo = QboClient::new(&state.config, &state.http_client, &state.token_store);

    match qbo
        .get_entity(&format!("companyinfo/{}", tokens.realm_id))
        .await
    {
        Ok(value) => {
            if let Some(raw) = value.get("CompanyInfo") {
                if let Ok(info) =
                    serde_json::from_value::<crate::qbo::types::CompanyInfo>(raw.clone())
                {
                    return render(&CompanyTemplate {
                        active_nav: "company".into(),
                        connected: true,
                        error_message: String::new(),
                        company_name: unwrap_or_dash(&info.company_name),
                        legal_name: unwrap_or_dash(&info.legal_name),
                        country: unwrap_or_dash(&info.country),
                        fiscal_year_start: unwrap_or_dash(&info.fiscal_year_start_month),
                    });
                }
            }
            render(&CompanyTemplate {
                active_nav: "company".into(),
                connected: true,
                error_message: "Failed to parse company info.".into(),
                company_name: String::new(),
                legal_name: String::new(),
                country: String::new(),
                fiscal_year_start: String::new(),
            })
        }
        Err(e) => render(&CompanyTemplate {
            active_nav: "company".into(),
            connected: true,
            error_message: format!("{:?}", e),
            company_name: String::new(),
            legal_name: String::new(),
            country: String::new(),
            fiscal_year_start: String::new(),
        }),
    }
}
