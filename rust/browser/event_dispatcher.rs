use std::sync::Arc;

use crate::core::events::event_bus::{Event, EventBus};

/// Bridges internal browser events into a simple string-based callback format
/// suitable for FFI forwarding.
///
/// This dispatcher owns only the callback handler used to forward events
/// outward. It does not store application state, interpret business meaning,
/// or perform asynchronous work.
pub struct EventDispatcher {
    /// String-based callback invoked for every forwarded event.
    ///
    /// Callback arguments:
    /// - event type
    /// - payload
    pub handler: Arc<dyn Fn(String, String) + Send + Sync>,
}

impl EventDispatcher {
    /// Creates a new event dispatcher with the provided forwarding handler.
    pub fn new(handler: Arc<dyn Fn(String, String) + Send + Sync>) -> Self {
        Self { handler }
    }

    /// Registers this dispatcher with the provided [EventBus].
    ///
    /// Registration is synchronous and appends a subscriber closure to the
    /// bus. Each received internal event is mapped into a `(event_type,
    /// payload)` pair and immediately forwarded through the stored handler.
    pub fn register(&self, event_bus: &mut EventBus) {
        let handler = Arc::clone(&self.handler);

        event_bus.subscribe(Box::new(move |event| {
            let (event_type, payload) = Self::map_event(event);
            handler(event_type, payload);
        }));
    }

    /// Converts an internal [Event] into a string event type and payload pair.
    ///
    /// Mapping rules are intentionally simple and stable so downstream FFI
    /// consumers can deserialize them without Rust-specific knowledge.
    fn map_event(event: &Event) -> (String, String) {
        match event {
            Event::TabCreated(id) => ("TabCreated".to_string(), id.clone()),
            Event::TabClosed(id) => ("TabClosed".to_string(), id.clone()),
            Event::TabSuspended(id) => ("TabSuspended".to_string(), id.clone()),
            Event::TabResumed(id) => ("TabResumed".to_string(), id.clone()),
            Event::NavigationCompleted(tab_id, url) => (
                "NavigationCompleted".to_string(),
                format!("{tab_id}|{url}"),
            ),
            Event::BlockedRequest(url) => ("BlockedRequest".to_string(), url.clone()),
        }
    }
}
