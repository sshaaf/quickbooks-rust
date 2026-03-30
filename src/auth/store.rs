use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TokenData {
    pub realm_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

impl TokenData {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn needs_refresh(&self) -> bool {
        Utc::now() >= self.expires_at - chrono::Duration::minutes(5)
    }
}

#[derive(Debug, Clone)]
pub struct TokenStore {
    tokens: Arc<RwLock<Option<TokenData>>>,
    pending_states: Arc<RwLock<HashMap<String, ()>>>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(None)),
            pending_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_tokens(&self, data: TokenData) {
        let mut lock = self.tokens.write().await;
        *lock = Some(data);
    }

    pub async fn get_tokens(&self) -> Option<TokenData> {
        let lock = self.tokens.read().await;
        lock.clone()
    }

    pub async fn clear_tokens(&self) {
        let mut lock = self.tokens.write().await;
        *lock = None;
    }

    pub async fn is_connected(&self) -> bool {
        let lock = self.tokens.read().await;
        lock.is_some()
    }

    pub async fn store_state(&self, state: String) {
        let mut lock = self.pending_states.write().await;
        lock.insert(state, ());
    }

    pub async fn validate_state(&self, state: &str) -> bool {
        let mut lock = self.pending_states.write().await;
        lock.remove(state).is_some()
    }
}
