use serde::{Deserialize, Serialize};

/// Represents an entity modeling a distinct browser tab state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

impl Tab {
    pub fn new(id: String) -> Self {
        Self {
            id,
            title: String::new(),
            url: "about:blank".to_string(),
            is_active: false,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
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
