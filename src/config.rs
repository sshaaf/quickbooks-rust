use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub qbo_client_id: String,
    pub qbo_client_secret: String,
    pub qbo_redirect_uri: String,
    pub qbo_base_url: String,
    pub qbo_auth_url: String,
    pub qbo_token_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenvy::dotenv().ok();

        let environment = env::var("QBO_ENVIRONMENT").unwrap_or_else(|_| "sandbox".into());

        let qbo_base_url = match environment.as_str() {
            "production" => "https://quickbooks.api.intuit.com/v3/company".into(),
            _ => "https://sandbox-quickbooks.api.intuit.com/v3/company".into(),
        };

        Ok(Self {
            qbo_client_id: env::var("QBO_CLIENT_ID")?,
            qbo_client_secret: env::var("QBO_CLIENT_SECRET")?,
            qbo_redirect_uri: env::var("QBO_REDIRECT_URI")?,
            qbo_base_url,
            qbo_auth_url: "https://appcenter.intuit.com/connect/oauth2".into(),
            qbo_token_url: "https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer".into(),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
        })
    }
}
