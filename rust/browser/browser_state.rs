use crate::browser::navigation_controller::NavigationController;
use crate::browser::tab_manager::{Tab, TabId, TabManager};

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

/// High-level browser-state events emitted by [BrowserState].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserEvent {
    /// Emitted when the active tab changes.
    ActiveTabChanged(TabId),
    /// Emitted when the set of tabs changes.
    TabsUpdated,
    /// Emitted when tracked window bounds are updated.
    WindowResized(WindowBounds),
}

/// Emits a browser-state event.
///
/// This is a temporary integration stub. A future event bus will consume and
/// dispatch these events to interested subsystems.
fn emit_event(_event: BrowserEvent) {}

/// Global, synchronous browser state coordinator.
///
/// This type is the state-layer entry point for tab and navigation operations.
/// It delegates tab ownership to [TabManager] and navigation ownership to
/// [NavigationController], while exposing a deterministic API for higher-level
/// browser orchestration.
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

    /// Creates a new tab through [TabManager], then emits:
    /// - [BrowserEvent::TabsUpdated]
    /// - [BrowserEvent::ActiveTabChanged]
    pub fn create_tab(&mut self) -> Tab {
        let tab = self.tab_manager.create_tab();
        emit_event(BrowserEvent::TabsUpdated);
        emit_event(BrowserEvent::ActiveTabChanged(tab.id.clone()));
        tab
    }

    /// Closes a tab through [TabManager], emits [BrowserEvent::TabsUpdated],
    /// and emits [BrowserEvent::ActiveTabChanged] if the active tab changed to
    /// another tab.
    pub fn close_tab(&mut self, tab_id: TabId) {
        let active_before = self.get_active_tab_id();
        self.tab_manager.close_tab(tab_id);
        emit_event(BrowserEvent::TabsUpdated);

        let active_after = self.get_active_tab_id();
        if active_before != active_after {
            if let Some(active_tab_id) = active_after {
                emit_event(BrowserEvent::ActiveTabChanged(active_tab_id));
            }
        }
    }

    /// Sets the active tab through [TabManager], then emits
    /// [BrowserEvent::ActiveTabChanged] for the resulting active tab.
    pub fn set_active_tab(&mut self, tab_id: TabId) {
        self.tab_manager.set_active_tab(tab_id);
        if let Some(active_tab_id) = self.get_active_tab_id() {
            emit_event(BrowserEvent::ActiveTabChanged(active_tab_id));
        }
    }

    /// Navigates the active tab using [NavigationController].
    ///
    /// Returns the normalized URL when an active tab exists, otherwise `None`.
    pub fn navigate(&mut self, input: String) -> Option<String> {
        let active_tab_id = self.get_active_tab_id()?;
        Some(self.navigation_controller.navigate(active_tab_id, input))
    }

    /// Requests backward navigation on the active tab via
    /// [NavigationController].
    ///
    /// Returns the resolved URL when possible, otherwise `None`.
    pub fn go_back(&mut self) -> Option<String> {
        let active_tab_id = self.get_active_tab_id()?;
        self.navigation_controller.go_back(active_tab_id)
    }

    /// Requests forward navigation on the active tab via
    /// [NavigationController].
    ///
    /// Returns the resolved URL when possible, otherwise `None`.
    pub fn go_forward(&mut self) -> Option<String> {
        let active_tab_id = self.get_active_tab_id()?;
        self.navigation_controller.go_forward(active_tab_id)
    }

    /// Returns the current tab snapshot from [TabManager].
    pub fn get_tabs(&self) -> Vec<Tab> {
        self.tab_manager.get_all_tabs()
    }

    /// Returns the currently active tab identifier, if any.
    ///
    /// Tab activity is derived directly from [TabManager] state.
    pub fn get_active_tab_id(&self) -> Option<TabId> {
        self.tab_manager.get_active_tab_id()
    }

    /// Updates tracked window bounds and emits [BrowserEvent::WindowResized].
    pub fn set_window_bounds(&mut self, bounds: WindowBounds) {
        self.window_bounds = bounds;
        emit_event(BrowserEvent::WindowResized(bounds));
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self::new()
    }
}
