use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::browser::browser_state::{BrowserState as BrowserRuntimeState, WindowBounds};
use crate::core::entities::browser_event::BrowserEvent;
use crate::core::entities::browser_state::BrowserState;
use crate::core::error::NetraError;
use crate::core::events::event_bus::{Event, EventBus};
use crate::core::entities::tab::{Tab, TabId};
use crate::native_control;

/// Central synchronous orchestrator for the browser core.
///
/// This controller exposes a single entry point for high-level browser
/// operations. It owns the global [BrowserState] coordinator and the internal
/// [EventBus], delegating all stateful behavior to the state layer while
/// publishing shared internal events after state-changing operations.
///
/// Architectural constraints:
/// - no UI logic
/// - no direct WebView or native bridge interaction
/// - no duplicated logic from lower browser modules
/// - no async execution or threading
pub struct BrowserController {
    /// Global browser state coordinator.
    pub state: BrowserRuntimeState,
    /// Internal synchronous event bus.
    pub event_bus: EventBus,
}

/// Shared global browser controller instance used by FFI-facing entry points.
static BROWSER_CONTROLLER: Lazy<Arc<Mutex<BrowserController>>> =
    Lazy::new(|| Arc::new(Mutex::new(BrowserController::new())));

/// Returns the shared browser controller singleton.
pub fn get_browser_controller() -> Arc<Mutex<BrowserController>> {
    Arc::clone(&BROWSER_CONTROLLER)
}

impl BrowserController {
    /// Creates a new browser controller with default state and an empty event
    /// bus.
    pub fn new() -> Self {
        Self {
            state: BrowserRuntimeState::new(),
            event_bus: EventBus::new(),
        }
    }

    /// Creates a new tab through [BrowserState] and then publishes
    /// [Event::TabCreated] with the created tab identifier.
    pub fn create_tab(&mut self) -> Result<Tab, NetraError> {
        println!("[RUST] create_tab");
        let tab = self.state.create_tab();
        if let Err(error) = native_control::create_tab(&tab.id) {
            self.state.close_tab(tab.id.clone());
            return Err(error);
        }
        self.event_bus.publish(Event::TabCreated(tab.id.clone()));
        Ok(tab)
    }

    /// Closes an existing tab through [BrowserState] and then publishes
    /// [Event::TabClosed] with the closed tab identifier.
    pub fn close_tab(&mut self, tab_id: TabId) -> Result<(), NetraError> {
        self.ensure_tab_exists(&tab_id)?;
        native_control::close_tab(&tab_id)?;
        self.state.close_tab(tab_id.clone());
        self.event_bus.publish(Event::TabClosed(tab_id));
        Ok(())
    }

    /// Delegates active-tab switching to [BrowserState].
    ///
    /// This method intentionally does not publish an additional event-bus
    /// event because the shared internal event surface defined for this phase
    /// does not include an active-tab-change variant.
    pub fn set_active_tab(&mut self, tab_id: TabId) -> Result<(), NetraError> {
        if !self.state.contains_tab(&tab_id) {
            return Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")));
        }

        native_control::set_active_tab(&tab_id)?;
        if self.state.set_active_tab(tab_id.clone()) {
            Ok(())
        } else {
            Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")))
        }
    }

    /// Navigates the active tab through [BrowserState].
    ///
    /// Returns the normalized URL when an active tab exists, otherwise `None`.
    pub fn navigate(&mut self, input: String) -> Option<String> {
        let tab_id = self.get_active_tab_id()?;
        let url = self.state.navigate(input)?;
        self.state.set_tab_loading_state(&tab_id, true);
        self.event_bus
            .publish(Event::NavigationCompleted(tab_id, url.clone()));
        Some(url)
    }

    /// Requests backward navigation through [BrowserState].
    ///
    /// Returns the resolved URL when backward navigation is possible,
    /// otherwise `None`.
    pub fn go_back(&mut self) -> Option<String> {
        let tab_id = self.get_active_tab_id()?;
        let url = self.state.go_back()?;
        self.state.set_tab_loading_state(&tab_id, true);
        self.event_bus
            .publish(Event::NavigationCompleted(tab_id, url.clone()));
        Some(url)
    }

    /// Requests forward navigation through [BrowserState].
    ///
    /// Returns the resolved URL when forward navigation is possible,
    /// otherwise `None`.
    pub fn go_forward(&mut self) -> Option<String> {
        let tab_id = self.get_active_tab_id()?;
        let url = self.state.go_forward()?;
        self.state.set_tab_loading_state(&tab_id, true);
        self.event_bus
            .publish(Event::NavigationCompleted(tab_id, url.clone()));
        Some(url)
    }

    /// Activates the requested tab and loads the supplied URL.
    pub fn load_url(&mut self, tab_id: TabId, url: String) -> Result<(), NetraError> {
        println!("[RUST] load_url: tab_id={tab_id}, url={url}");
        self.set_active_tab(tab_id.clone())?;
        let normalized_url = self
            .navigate(url)
            .ok_or_else(|| NetraError::OperationFailed("failed to load URL".to_string()))?;
        native_control::load_url(&tab_id, &normalized_url)?;
        Ok(())
    }

    /// Activates the requested tab and navigates backward in its history.
    pub fn go_back_for_tab(&mut self, tab_id: TabId) -> Result<(), NetraError> {
        self.set_active_tab(tab_id.clone())?;
        let _ = self
            .go_back()
            .ok_or_else(|| NetraError::OperationFailed("back navigation is not available".to_string()))?;
        native_control::go_back(&tab_id)?;
        Ok(())
    }

    /// Activates the requested tab and navigates forward in its history.
    pub fn go_forward_for_tab(&mut self, tab_id: TabId) -> Result<(), NetraError> {
        self.set_active_tab(tab_id.clone())?;
        let _ = self
            .go_forward()
            .ok_or_else(|| NetraError::OperationFailed("forward navigation is not available".to_string()))?;
        native_control::go_forward(&tab_id)?;
        Ok(())
    }

    /// Activates the requested tab and reuses its current URL as a reload target.
    pub fn reload_tab(&mut self, tab_id: TabId) -> Result<(), NetraError> {
        self.set_active_tab(tab_id.clone())?;
        let current_url = self
            .state
            .get_tab(&tab_id)
            .map(|tab| tab.url)
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| NetraError::OperationFailed("reload is not available for an empty tab".to_string()))?;

        native_control::reload(&tab_id)?;
        self.state.set_tab_loading_state(&tab_id, true);
        self.event_bus
            .publish(Event::NavigationCompleted(tab_id, current_url));
        Ok(())
    }

    /// Marks the requested tab as no longer loading.
    pub fn stop_loading(&mut self, tab_id: TabId) -> Result<(), NetraError> {
        self.ensure_tab_exists(&tab_id)?;
        native_control::stop_loading(&tab_id)?;
        self.state.set_tab_loading_state(&tab_id, false);
        Ok(())
    }

    /// Returns the current tab snapshot from [BrowserState].
    pub fn get_tabs(&self) -> Vec<Tab> {
        self.state.get_tabs()
    }

    /// Returns the currently active tab identifier, if any.
    pub fn get_active_tab_id(&self) -> Option<TabId> {
        self.state.get_active_tab_id()
    }

    /// Returns the current Rust-owned browser-state snapshot for FFI readers.
    pub fn get_browser_state(&self) -> BrowserState {
        self.state.snapshot()
    }

    /// Updates window bounds through [BrowserState].
    pub fn set_window_bounds(&mut self, bounds: WindowBounds) {
        self.state.set_window_bounds(bounds);
    }

    /// Registers a synchronous event subscriber by delegating to [EventBus].
    pub fn subscribe(&mut self, handler: Box<dyn Fn(&Event) + Send + Sync>) {
        self.event_bus.subscribe(handler);
    }

    /// Publishes a typed native browser event into the shared Rust event bus.
    pub fn handle_browser_event(&mut self, event: BrowserEvent) {
        self.state.apply_browser_event(&event);
        self.event_bus.publish(Event::BrowserEvent(event));
    }

    fn ensure_tab_exists(&self, tab_id: &str) -> Result<(), NetraError> {
        if self.state.contains_tab(tab_id) {
            Ok(())
        } else {
            Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")))
        }
    }
}

impl Default for BrowserController {
    fn default() -> Self {
        Self::new()
    }
}
