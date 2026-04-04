use std::sync::Arc;

use crate::core::entities::browser_event::BrowserEvent;
use crate::core::entities::tab::Tab;
use crate::core::events::event_bus::{Event, EventBus};

/// Bridges internal browser events into a structured formatted callback
/// suitable for FFI forwarding.
///
/// This dispatcher owns only the callback handler used to forward events
/// outward. It does not store application state, interpret business meaning,
/// or perform asynchronous work.
pub struct EventDispatcher {
    /// Structured callback invoked for every forwarded event.
    pub handler: Arc<dyn Fn(u32, BrowserEvent, Option<Tab>) + Send + Sync>,
}

impl EventDispatcher {
    /// Creates a new event dispatcher with the provided forwarding handler.
    pub fn new(handler: Arc<dyn Fn(u32, BrowserEvent, Option<Tab>) + Send + Sync>) -> Self {
        Self { handler }
    }

    /// Registers this dispatcher with the provided [EventBus].
    pub fn register(&self, event_bus: &EventBus) {
        let handler = Arc::clone(&self.handler);

        event_bus.subscribe(Box::new(move |event| {
            if let Event::BrowserEvent(seq, browser_event, current_tab) = event {
                handler(*seq, browser_event.clone(), current_tab.clone());
            }
        }));
    }
}
