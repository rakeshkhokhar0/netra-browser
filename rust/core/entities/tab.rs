use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Stable identifier used to address tabs across the Rust browser core.
pub type TabId = String;

/// Represents an entity modeling a distinct browser tab state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    /// Stable identifier used across Rust, Flutter, and the native host.
    pub id: TabId,
    /// Current document title associated with the tab.
    pub title: String,
    /// Current favicon URL associated with the tab, when provided by WebView2.
    pub favicon_url: Option<String>,
    /// Current visible URL for the tab.
    pub url: String,
    /// Whether this tab is currently the active tab.
    pub is_active: bool,
    /// Whether the tab is currently loading content.
    pub is_loading: bool,
    /// Whether backward navigation is currently available.
    pub can_go_back: bool,
    /// Whether forward navigation is currently available.
    pub can_go_forward: bool,
    /// Whether the tab has been suspended by background-tab policy.
    pub is_suspended: bool,
    /// Number of intercepted requests blocked for this tab.
    ///
    /// This counter is updated from native request-blocked events routed back
    /// through the Rust event pipeline.
    pub blocked_count: u32,
    /// Monotonically increasing sequence number for this tab's events.
    pub sequence_number: u32,
    #[serde(skip, default = "Instant::now")]
    pub last_accessed: Instant,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            title: String::new(),
            favicon_url: None,
            url: "about:blank".to_string(),
            is_active: false,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
            is_suspended: false,
            blocked_count: 0,
            sequence_number: 0,
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

    pub fn update_favicon_url(&mut self, favicon_url: Option<String>) {
        self.favicon_url = favicon_url;
    }

    pub fn increment_blocked_count(&mut self) {
        self.blocked_count = self.blocked_count.saturating_add(1);
    }
}
