use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterList {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub rules_count: i32,
}

impl FilterList {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            enabled: true,
            rules_count: 0,
        }
    }
}
