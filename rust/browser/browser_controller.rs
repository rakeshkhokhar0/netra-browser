use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::browser::browser_state::{BrowserState as BrowserRuntimeState, WindowBounds};
use crate::core::entities::browser_event::BrowserEvent;
use crate::core::entities::browser_state::BrowserState;
use crate::core::entities::tab::{Tab, TabId};
use crate::core::error::NetraError;
use crate::core::events::event_bus::{Event, EventBus};
use crate::native_control;

/// Central synchronous orchestrator for browser-core operations.
///
/// Architectural role:
/// - sole coordinator for Rust-owned browser state mutation
/// - bridge between state orchestration and the native control layer
/// - source of internal event publication after successful operations
///
/// Safety rules enforced by this controller:
/// - public mutating operations never require `&mut self`
/// - each public mutating operation locks the global controller only for Rust
///   state mutation
/// - all `native_control::*` calls run after the controller mutex has been
///   released
/// - internal events are published through a cloned [EventBus] after the
///   controller mutex has been released
/// - no UI logic or FFI bindings are stored here
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

    /// Creates a new tab using three isolated phases:
    /// 1. lock controller and update Rust state
    /// 2. unlock controller and call the native layer
    /// 3. publish the resulting event without the controller lock held
    ///
    /// Failure handling:
    /// - native tab creation still uses a narrow tab-specific cleanup path
    ///   because the tab has not been materialized natively yet
    pub fn create_tab_safe() -> Result<Tab, NetraError> {
        println!("[RUST] create_tab_safe");

        let (tab_id, event_bus, browser_event) = Self::with_locked_controller(|controller| {
            let created_tab = controller.state.create_tab()?;
            let sequence_number = controller
                .state
                .tab_manager
                .increment_sequence(&created_tab.id);
            let authoritative_tab = controller
                .state
                .get_tab(&created_tab.id)
                .unwrap_or(created_tab.clone());
            let browser_event = Event::BrowserEvent(
                sequence_number,
                BrowserEvent::FrameCreated {
                    tab_id: created_tab.id.clone(),
                },
                Some(authoritative_tab.clone()),
            );

            Ok((authoritative_tab.id, controller.event_bus.clone(), browser_event))
        })?;

        if let Err(error) = native_control::create_tab(&tab_id) {
            let rollback_result = Self::with_locked_controller(|controller| {
                controller.state.close_tab(tab_id.clone());
                Ok(())
            });

            if let Err(rollback_error) = rollback_result {
                eprintln!("[RUST] create_tab_safe rollback failed: {rollback_error}");
            }

            return Err(error);
        }

        let tab = Self::with_locked_controller(|controller| {
            controller
                .state
                .get_tab(&tab_id)
                .ok_or_else(|| NetraError::NotFound(format!("tab `{tab_id}` was not found")))
        })?;

        event_bus.publish(Event::TabCreated(tab.id.clone()));
        event_bus.publish(browser_event);
        Ok(tab)
    }

    /// Closes an existing tab using three isolated phases:
    /// 1. lock controller and update Rust state
    /// 2. unlock controller and call the native layer
    /// 3. publish the resulting event without the controller lock held
    ///
    /// Failure handling:
    /// - native failures are returned directly
    /// - no full-state rollback is attempted, which prevents stale snapshots
    ///   from overwriting concurrent state changes
    pub fn close_tab_safe(tab_id: TabId) -> Result<(), NetraError> {
        let (event_bus, browser_event) = Self::with_locked_controller(|controller| {
            controller.ensure_tab_exists(&tab_id)?;
            controller.state.close_tab(tab_id.clone());

            let next_active_tab = controller
                .state
                .get_active_tab_id()
                .and_then(|active_tab_id| controller.state.get_tab(&active_tab_id));
            let sequence_number = next_active_tab
                .as_ref()
                .map(|tab| controller.state.tab_manager.increment_sequence(&tab.id))
                .unwrap_or(0);
            let authoritative_tab = next_active_tab
                .as_ref()
                .and_then(|tab| controller.state.get_tab(&tab.id));
            let browser_event = Event::BrowserEvent(
                sequence_number,
                BrowserEvent::FrameDestroyed {
                    tab_id: tab_id.clone(),
                },
                authoritative_tab,
            );

            Ok((controller.event_bus.clone(), browser_event))
        })?;

        native_control::close_tab(&tab_id)?;

        event_bus.publish(Event::TabClosed(tab_id.clone()));
        event_bus.publish(browser_event);
        Ok(())
    }

    /// Activates the requested tab using three isolated phases:
    /// 1. lock controller and update Rust state
    /// 2. unlock controller and call the native layer
    /// 3. complete without publishing while unlocked
    ///
    /// Failure handling:
    /// - native failures are returned directly
    /// - no full-state rollback is attempted
    pub fn set_active_tab_safe(tab_id: TabId) -> Result<(), NetraError> {
        let should_reload = Self::with_locked_controller(|controller| {
            controller.ensure_tab_exists(&tab_id)?;
            let was_suspended = controller.state.tab_manager.is_tab_suspended(&tab_id);

            if controller.state.set_active_tab(tab_id.clone()) {
                let current_url = controller
                    .state
                    .get_tab(&tab_id)
                    .map(|tab| tab.url)
                    .unwrap_or_default();
                let should_reload = was_suspended
                    && !current_url.trim().is_empty()
                    && current_url.trim() != "about"
                    && current_url.trim() != "about:blank";

                if should_reload {
                    controller.state.set_tab_loading_state(&tab_id, true);
                }

                Ok(should_reload)
            } else {
                Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")))
            }
        })?;

        let native_result = native_control::set_active_tab(&tab_id).and_then(|_| {
            if should_reload {
                native_control::reload(&tab_id)
            } else {
                Ok(())
            }
        });

        native_result
    }

    /// Activates the requested tab, normalizes the supplied URL, updates Rust
    /// navigation state, and then forwards the command to the native layer.
    ///
    /// This method keeps navigation behavior unchanged while ensuring the
    /// controller mutex is never held during `native_control::*` execution or
    /// during event publication.
    ///
    /// Failure handling:
    /// - native failures are returned directly
    /// - no full-state rollback is attempted
    pub fn navigate_safe(tab_id: TabId, url: String) -> Result<(), NetraError> {
        println!("[RUST] navigate_safe: tab_id={tab_id}, url={url}");

        let (event_bus, normalized_url) =
            Self::with_locked_controller(|controller| {
                controller.ensure_tab_exists(&tab_id)?;

                if !controller.state.set_active_tab(tab_id.clone()) {
                    return Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")));
                }

                let normalized_url = controller
                    .state
                    .navigate(url.clone())
                    .ok_or_else(|| {
                        NetraError::OperationFailed("failed to load URL".to_string())
                    })?;

                controller.state.set_tab_loading_state(&tab_id, true);

                Ok((controller.event_bus.clone(), normalized_url))
            })?;

        native_control::set_active_tab(&tab_id)
            .and_then(|_| native_control::load_url(&tab_id, &normalized_url))?;

        event_bus.publish(Event::NavigationCompleted(
            tab_id,
            normalized_url.clone(),
        ));
        Ok(())
    }

    /// Activates the requested tab, moves its history cursor backward in Rust
    /// state, and then forwards the back command to the native layer.
    ///
    /// Failure handling:
    /// - native failures are returned directly
    /// - no full-state rollback is attempted
    pub fn go_back_safe(tab_id: TabId) -> Result<(), NetraError> {
        let (event_bus, resolved_url) =
            Self::with_locked_controller(|controller| {
                controller.ensure_tab_exists(&tab_id)?;

                if !controller.state.set_active_tab(tab_id.clone()) {
                    return Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")));
                }

                let resolved_url = controller.state.go_back().ok_or_else(|| {
                    NetraError::OperationFailed(
                        "back navigation is not available".to_string(),
                    )
                })?;

                controller.state.set_tab_loading_state(&tab_id, true);

                Ok((controller.event_bus.clone(), resolved_url))
            })?;

        native_control::set_active_tab(&tab_id)
            .and_then(|_| native_control::go_back(&tab_id))?;

        event_bus.publish(Event::NavigationCompleted(
            tab_id,
            resolved_url.clone(),
        ));
        Ok(())
    }

    /// Activates the requested tab, moves its history cursor forward in Rust
    /// state, and then forwards the forward command to the native layer.
    ///
    /// Failure handling:
    /// - native failures are returned directly
    /// - no full-state rollback is attempted
    pub fn go_forward_safe(tab_id: TabId) -> Result<(), NetraError> {
        let (event_bus, resolved_url) =
            Self::with_locked_controller(|controller| {
                controller.ensure_tab_exists(&tab_id)?;

                if !controller.state.set_active_tab(tab_id.clone()) {
                    return Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")));
                }

                let resolved_url = controller.state.go_forward().ok_or_else(|| {
                    NetraError::OperationFailed(
                        "forward navigation is not available".to_string(),
                    )
                })?;

                controller.state.set_tab_loading_state(&tab_id, true);

                Ok((controller.event_bus.clone(), resolved_url))
            })?;

        native_control::set_active_tab(&tab_id)
            .and_then(|_| native_control::go_forward(&tab_id))?;

        event_bus.publish(Event::NavigationCompleted(
            tab_id,
            resolved_url.clone(),
        ));
        Ok(())
    }

    /// Activates the requested tab and triggers a native reload while Rust
    /// keeps the current URL and loading state authoritative.
    ///
    /// Failure handling:
    /// - native failures are returned directly
    /// - no full-state rollback is attempted
    pub fn reload_safe(tab_id: TabId) -> Result<(), NetraError> {
        let (event_bus, current_url) =
            Self::with_locked_controller(|controller| {
                controller.ensure_tab_exists(&tab_id)?;

                if !controller.state.set_active_tab(tab_id.clone()) {
                    return Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")));
                }

                let current_url = controller
                    .state
                    .get_tab(&tab_id)
                    .map(|tab| tab.url)
                    .filter(|url| !url.trim().is_empty())
                    .ok_or_else(|| {
                        NetraError::OperationFailed(
                            "reload is not available for an empty tab".to_string(),
                        )
                    })?;

                controller.state.set_tab_loading_state(&tab_id, true);

                Ok((controller.event_bus.clone(), current_url))
            })?;

        native_control::set_active_tab(&tab_id)
            .and_then(|_| native_control::reload(&tab_id))?;

        event_bus.publish(Event::NavigationCompleted(tab_id, current_url));
        Ok(())
    }

    /// Stops loading in the requested tab while keeping the controller mutex
    /// released during the native stop call.
    ///
    /// Failure handling:
    /// - native failures are returned directly
    /// - no full-state rollback is attempted
    pub fn stop_loading_safe(tab_id: TabId) -> Result<(), NetraError> {
        Self::with_locked_controller(|controller| {
            controller.ensure_tab_exists(&tab_id)?;
            controller.state.set_tab_loading_state(&tab_id, false);
            Ok(())
        })?;

        native_control::stop_loading(&tab_id)
    }

    /// Applies an authoritative native browser event to Rust-owned state and
    /// publishes the derived internal event after releasing the controller
    /// mutex.
    pub fn handle_browser_event_safe(event: BrowserEvent) -> Result<(), NetraError> {
        let (event_bus, published_event, sequence_number) =
            Self::with_locked_controller(|controller| {
                println!("[Rust] Event received: {:?}", event);
                let tab_id = event.tab_id().to_string();
                controller.state.apply_browser_event(&event);
                println!("[Rust] State updated for tab: {tab_id}");

                let sequence_tab_id = controller
                    .state
                    .get_tab(&tab_id)
                    .map(|tab| tab.id)
                    .or_else(|| {
                        controller.state.get_active_tab_id().and_then(|active_tab_id| {
                            controller.state.get_tab(&active_tab_id).map(|tab| tab.id)
                        })
                    })
                    .unwrap_or(tab_id);

                let sequence_number = controller
                    .state
                    .tab_manager
                    .increment_sequence(&sequence_tab_id);
                let current_tab = controller.state.get_tab(&sequence_tab_id);
                let published_event =
                    Event::BrowserEvent(sequence_number, event.clone(), current_tab);

                Ok((controller.event_bus.clone(), published_event, sequence_number))
            })?;

        event_bus.publish(published_event);
        println!("[Rust] Event published with sequence: {sequence_number}");
        Ok(())
    }

    /// Returns a read-only browser-state snapshot using the shared global
    /// controller lock.
    pub fn get_browser_state_safe() -> Result<BrowserState, NetraError> {
        Self::with_locked_controller(|controller| Ok(controller.state.snapshot()))
    }

    /// Updates tracked window bounds through the shared global controller lock.
    pub fn set_window_bounds_safe(bounds: WindowBounds) -> Result<(), NetraError> {
        Self::with_locked_controller(|controller| {
            controller.state.set_window_bounds(bounds);
            Ok(())
        })
    }

    /// Registers a synchronous event subscriber by delegating to [EventBus].
    pub fn subscribe(&self, handler: Box<dyn Fn(&Event) + Send + Sync>) {
        self.event_bus.subscribe(handler);
    }

    /// Registers a synchronous event subscriber on the shared global event bus
    /// without holding the browser-controller mutex during subscription.
    ///
    /// Locking model:
    /// 1. briefly lock the shared controller only to clone the event bus
    /// 2. release the controller mutex
    /// 3. register the subscriber on the cloned event bus
    ///
    /// This keeps event-bus subscriber management isolated from the main
    /// browser-controller lock and prevents future lock layering between
    /// controller access and subscriber registration.
    pub fn subscribe_safe(handler: Box<dyn Fn(&Event) + Send + Sync>) -> Result<(), NetraError> {
        let event_bus = Self::with_locked_controller(|controller| {
            Ok(controller.event_bus.clone())
        })?;

        event_bus.subscribe(handler);
        Ok(())
    }

    /// Returns the current Rust-owned browser-state snapshot for FFI readers.
    pub fn get_browser_state(&self) -> BrowserState {
        self.state.snapshot()
    }

    fn ensure_tab_exists(&self, tab_id: &str) -> Result<(), NetraError> {
        if self.state.contains_tab(tab_id) {
            Ok(())
        } else {
            Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")))
        }
    }

    fn with_locked_controller<T>(
        operation: impl FnOnce(&mut BrowserController) -> Result<T, NetraError>,
    ) -> Result<T, NetraError> {
        let controller = get_browser_controller();
        let mut controller = controller.lock().map_err(|_| {
            NetraError::OperationFailed("browser controller lock poisoned".to_string())
        })?;
        operation(&mut controller)
    }
}

impl Default for BrowserController {
    fn default() -> Self {
        Self::new()
    }
}
