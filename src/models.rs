use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl PaginationParams {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(25).clamp(1, 100)
    }

    pub fn start_position(&self) -> u32 {
        (self.page() - 1) * self.per_page() + 1
    }
}

#[derive(Debug, Deserialize)]
pub struct ReportParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
