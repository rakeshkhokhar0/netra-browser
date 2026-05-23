use crate::browser::navigation_controller::NavigationController;
use crate::browser::tab_manager::TabManager;
use crate::core::entities::browser_event::BrowserEvent;
use crate::core::entities::browser_state::BrowserState as BrowserStateSnapshot;
use crate::core::entities::tab::{Tab, TabId};
use crate::core::error::NetraError;

/// Represents native window geometry tracked by the Rust browser core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowBounds {
    /// Window width in physical pixels.
    pub width: u32,
    /// Window height in physical pixels.
    pub height: u32,
    /// Left position in screen coordinates.
    pub x: i32,
    /// Top position in screen coordinates.
    pub y: i32,
}

/// Global, synchronous browser state coordinator.
///
/// This type is the state-layer entry point for tab and navigation operations.
/// It delegates tab ownership to [TabManager] and navigation ownership to
/// [NavigationController], while exposing a deterministic API for higher-level
/// browser orchestration.
#[derive(Clone)]
pub struct BrowserState {
    /// Sole owner of tab lifecycle and tab-state data.
    pub tab_manager: TabManager,
    /// Sole owner of navigation history and URL normalization behavior.
    pub navigation_controller: NavigationController,
    /// Current native window bounds tracked by Rust state.
    pub window_bounds: WindowBounds,
}

impl BrowserState {
    /// Creates a new browser state with default window size and empty tab/nav
    /// state.
    ///
    /// Defaults:
    /// - width: 800
    /// - height: 600
    /// - x: 0
    /// - y: 0
    pub fn new() -> Self {
        Self {
            tab_manager: TabManager::new(),
            navigation_controller: NavigationController::new(),
            window_bounds: WindowBounds {
                width: 800,
                height: 600,
                x: 0,
                y: 0,
            },
        }
    }

    /// Creates a new tab through [TabManager].
    pub fn create_tab(&mut self) -> Result<Tab, NetraError> {
        self.tab_manager.create_tab()
    }

    /// Closes a tab through [TabManager].
    pub fn close_tab(&mut self, tab_id: TabId) {
        let _ = self.tab_manager.close_tab(tab_id);
    }

    /// Sets the active tab through [TabManager].
    pub fn set_active_tab(&mut self, tab_id: TabId) -> bool {
        self.tab_manager.set_active_tab(tab_id)
    }

    /// Navigates the active tab using [NavigationController].
    ///
    /// Returns the normalized URL when an active tab exists, otherwise `None`.
    pub fn navigate(&mut self, input: String) -> Option<String> {
        let active_tab_id = self.get_active_tab_id()?;
        let url = self.navigation_controller.navigate(input);
        let current_tab = self.get_tab(&active_tab_id);
        self.tab_manager.update_navigation_state(
            &active_tab_id,
            url.clone(),
            current_tab
                .as_ref()
                .map(|tab| tab.can_go_back)
                .unwrap_or(false),
            current_tab
                .as_ref()
                .map(|tab| tab.can_go_forward)
                .unwrap_or(false),
        );
        Some(url)
    }

    /// Returns the current tab snapshot from [TabManager].
    pub fn get_tabs(&self) -> Vec<Tab> {
        self.tab_manager.get_all_tabs()
    }

    /// Returns the requested tab snapshot from [TabManager], if it exists.
    pub fn get_tab(&self, tab_id: &str) -> Option<Tab> {
        self.tab_manager.get_tab(tab_id)
    }

    /// Returns whether a tab exists in the current browser state.
    pub fn contains_tab(&self, tab_id: &str) -> bool {
        self.tab_manager.contains_tab(tab_id)
    }

    /// Returns the currently active tab identifier, if any.
    ///
    /// Tab activity is derived directly from [TabManager] state.
    pub fn get_active_tab_id(&self) -> Option<TabId> {
        self.tab_manager.get_active_tab_id()
    }

    /// Updates tracked window bounds.
    pub fn set_window_bounds(&mut self, bounds: WindowBounds) {
        self.window_bounds = bounds;
    }

    /// Updates the loading state flag for a specific tab.
    pub fn set_tab_loading_state(&mut self, tab_id: &str, is_loading: bool) {
        self.tab_manager.set_loading_state(tab_id, is_loading);
    }

    /// Returns a serializable browser-state snapshot for FFI consumers.
    pub fn snapshot(&self) -> BrowserStateSnapshot {
        BrowserStateSnapshot {
            active_tab_id: self.get_active_tab_id(),
            tabs: self.get_tabs(),
        }
    }

    /// Applies a typed browser event to the Rust-owned state snapshot.
    ///
    /// Native WebView2 events flow into Rust through this path so the browser
    /// core remains the single source of truth for the current tab list,
    /// loading state, URL/title state, history affordances, and blocked
    /// request counts.
    pub fn apply_browser_event(&mut self, event: &BrowserEvent) {
        match event {
            BrowserEvent::FrameCreated { .. } => {}
            BrowserEvent::FrameDestroyed { tab_id } => {
                self.close_tab(tab_id.clone());
            }
            BrowserEvent::TitleChanged { tab_id, title } => {
                self.tab_manager.update_title(tab_id, title.clone());
            }
            BrowserEvent::FaviconChanged { tab_id, favicon_url } => {
                let next_favicon_url = if favicon_url.trim().is_empty() {
                    None
                } else {
                    Some(favicon_url.clone())
                };
                self.tab_manager.update_favicon_url(tab_id, next_favicon_url);
            }
            BrowserEvent::UrlChanged { tab_id, url } => {
                self.tab_manager.update_url(tab_id, url.clone());
            }
            BrowserEvent::NavigationStarted { tab_id, url, .. } => {
                let tab = self.get_tab(tab_id);
                self.tab_manager.update_navigation_state(
                    tab_id,
                    url.clone(),
                    tab.as_ref().map(|current| current.can_go_back).unwrap_or(false),
                    tab.as_ref()
                        .map(|current| current.can_go_forward)
                        .unwrap_or(false),
                );
                self.tab_manager.set_loading_state(tab_id, true);
            }
            BrowserEvent::NavigationCompleted { tab_id, url, .. } => {
                let tab = self.get_tab(tab_id);
                self.tab_manager.update_navigation_state(
                    tab_id,
                    url.clone(),
                    tab.as_ref().map(|current| current.can_go_back).unwrap_or(false),
                    tab.as_ref()
                        .map(|current| current.can_go_forward)
                        .unwrap_or(false),
                );
                self.tab_manager.set_loading_state(tab_id, false);
            }
            BrowserEvent::NavigationFailed { tab_id, url, .. } => {
                if !url.trim().is_empty() {
                    self.tab_manager.update_url(tab_id, url.clone());
                }
                self.tab_manager.set_loading_state(tab_id, false);
            }
            BrowserEvent::TabCrashed { tab_id } => {
                self.tab_manager.set_loading_state(tab_id, false);
            }
            BrowserEvent::LoadStarted { tab_id } => {
                self.tab_manager.set_loading_state(tab_id, true);
            }
            BrowserEvent::LoadFinished { tab_id } => {
                self.tab_manager.set_loading_state(tab_id, false);
            }
            BrowserEvent::HistoryStateChanged {
                tab_id,
                can_go_back,
                can_go_forward,
            } => {
                let current_url = self
                    .get_tab(tab_id)
                    .map(|tab| tab.url)
                    .unwrap_or_default();
                self.tab_manager.update_navigation_state(
                    tab_id,
                    current_url,
                    *can_go_back,
                    *can_go_forward,
                );
            }
            BrowserEvent::RequestBlocked { tab_id, .. } => {
                self.tab_manager.increment_blocked_count(tab_id);
            }
            BrowserEvent::ConsoleMessage { .. } => {}
        }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_tab_removes_closed_tab_from_state() {
        let mut state = BrowserState::new();
        let first_tab = state.create_tab().expect("first tab should be created");
        let second_tab = state.create_tab().expect("second tab should be created");

        let _ = state.set_active_tab(first_tab.id.clone());
        let _ = state.navigate("example.com".to_string());
        let _ = state.set_active_tab(second_tab.id.clone());
        let _ = state.navigate("openai.com".to_string());

        state.close_tab(first_tab.id.clone());

        assert!(!state.contains_tab(&first_tab.id));
        assert!(state.contains_tab(&second_tab.id));
    }
}
