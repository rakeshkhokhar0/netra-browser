use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub total_bytes: i64,
    pub received_bytes: i64,
    pub state: String,
}

impl Download {
    pub fn new(id: String, url: String, file_name: String) -> Self {
        Self {
            id,
            url,
            file_name,
            total_bytes: 0,
            received_bytes: 0,
            state: "Pending".to_string(),
        }
    }
}
