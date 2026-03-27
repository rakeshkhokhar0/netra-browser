use crate::browser::browser_state::{BrowserState, WindowBounds};
use crate::browser::tab_manager::{Tab, TabId};
use crate::core::events::event_bus::{Event, EventBus};

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
    pub state: BrowserState,
    /// Internal synchronous event bus.
    pub event_bus: EventBus,
}

impl BrowserController {
    /// Creates a new browser controller with default state and an empty event
    /// bus.
    pub fn new() -> Self {
        Self {
            state: BrowserState::new(),
            event_bus: EventBus::new(),
        }
    }

    /// Creates a new tab through [BrowserState] and then publishes
    /// [Event::TabCreated] with the created tab identifier.
    pub fn create_tab(&mut self) -> Tab {
        let tab = self.state.create_tab();
        self.event_bus.publish(Event::TabCreated(tab.id.clone()));
        tab
    }

    /// Closes an existing tab through [BrowserState] and then publishes
    /// [Event::TabClosed] with the closed tab identifier.
    pub fn close_tab(&mut self, tab_id: TabId) {
        self.state.close_tab(tab_id.clone());
        self.event_bus.publish(Event::TabClosed(tab_id));
    }

    /// Delegates active-tab switching to [BrowserState].
    ///
    /// This method intentionally does not publish an additional event-bus
    /// event because the shared internal event surface defined for this phase
    /// does not include an active-tab-change variant.
    pub fn set_active_tab(&mut self, tab_id: TabId) {
        self.state.set_active_tab(tab_id);
    }

    /// Navigates the active tab through [BrowserState].
    ///
    /// Returns the normalized URL when an active tab exists, otherwise `None`.
    pub fn navigate(&mut self, input: String) -> Option<String> {
        let tab_id = self.get_active_tab_id()?;
        let url = self.state.navigate(input)?;
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
        self.event_bus
            .publish(Event::NavigationCompleted(tab_id, url.clone()));
        Some(url)
    }

    /// Returns the current tab snapshot from [BrowserState].
    pub fn get_tabs(&self) -> Vec<Tab> {
        self.state.get_tabs()
    }

    /// Returns the currently active tab identifier, if any.
    pub fn get_active_tab_id(&self) -> Option<TabId> {
        self.state.get_active_tab_id()
    }

    /// Updates window bounds through [BrowserState].
    pub fn set_window_bounds(&mut self, bounds: WindowBounds) {
        self.state.set_window_bounds(bounds);
    }

    /// Registers a synchronous event subscriber by delegating to [EventBus].
    pub fn subscribe(&mut self, handler: Box<dyn Fn(&Event)>) {
        self.event_bus.subscribe(handler);
    }
}

impl Default for BrowserController {
    fn default() -> Self {
        Self::new()
    }
}
