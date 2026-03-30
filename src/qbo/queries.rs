pub fn list_query(entity: &str, start_position: u32, max_results: u32) -> String {
    format!(
        "SELECT * FROM {entity} STARTPOSITION {start_position} MAXRESULTS {max_results}"
    )
}

#[allow(dead_code)]
pub fn count_query(entity: &str) -> String {
    format!("SELECT COUNT(*) FROM {entity}")
}
