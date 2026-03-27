use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub folder: Option<String>,
    pub created_at: i64,
}

impl Bookmark {
    pub fn new(id: String, title: String, url: String) -> Self {
        Self {
            id,
            title,
            url,
            folder: None,
            created_at: 0,
        }
    }
}
