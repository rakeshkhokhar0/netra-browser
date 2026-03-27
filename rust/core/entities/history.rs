use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    pub timestamp: i64,
}

impl HistoryEntry {
    pub fn new(id: String, url: String, title: String) -> Self {
        Self {
            id,
            url,
            title,
            timestamp: 0,
        }
    }
}
