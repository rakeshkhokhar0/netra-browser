use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Stable identifier used to address tabs across the Rust browser core.
pub type TabId = String;

/// Represents an entity modeling a distinct browser tab state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_suspended: bool,
    #[serde(skip, default = "Instant::now")]
    pub last_accessed: Instant,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            title: String::new(),
            url: "about:blank".to_string(),
            is_active: false,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
            is_suspended: false,
            last_accessed: Instant::now(),
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn update_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn set_loading_state(&mut self, is_loading: bool) {
        self.is_loading = is_loading;
    }

    pub fn set_navigation_state(&mut self, can_go_back: bool, can_go_forward: bool) {
        self.can_go_back = can_go_back;
        self.can_go_forward = can_go_forward;
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
    }
}
