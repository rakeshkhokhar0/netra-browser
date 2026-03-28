use std::sync::Arc;

use crate::core::entities::browser_event::BrowserEvent;
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
            Event::BrowserEvent(browser_event) => Self::map_browser_event(browser_event),
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

    /// Converts a typed native [BrowserEvent] into a stable outward-facing
    /// string event type and payload pair.
    fn map_browser_event(event: &BrowserEvent) -> (String, String) {
        match event {
            BrowserEvent::FrameCreated { tab_id } => {
                ("FrameCreated".to_string(), tab_id.clone())
            }
            BrowserEvent::FrameDestroyed { tab_id } => {
                ("FrameDestroyed".to_string(), tab_id.clone())
            }
            BrowserEvent::TitleChanged { tab_id, title } => {
                ("TitleChanged".to_string(), format!("{tab_id}|{title}"))
            }
            BrowserEvent::UrlChanged { tab_id, url } => {
                ("UrlChanged".to_string(), format!("{tab_id}|{url}"))
            }
            BrowserEvent::NavigationStarted {
                tab_id,
                url,
                is_same_document,
            } => (
                "NavigationStarted".to_string(),
                format!("{tab_id}|{url}|{is_same_document}"),
            ),
            BrowserEvent::NavigationCompleted {
                tab_id,
                url,
                is_same_document,
            } => (
                "NavigationCompletedNative".to_string(),
                format!("{tab_id}|{url}|{is_same_document}"),
            ),
            BrowserEvent::NavigationFailed {
                tab_id,
                url,
                error_code,
                description,
            } => (
                "NavigationFailed".to_string(),
                format!("{tab_id}|{url}|{error_code}|{description}"),
            ),
            BrowserEvent::LoadStarted { tab_id } => {
                ("LoadStarted".to_string(), tab_id.clone())
            }
            BrowserEvent::LoadFinished { tab_id } => {
                ("LoadFinished".to_string(), tab_id.clone())
            }
            BrowserEvent::HistoryStateChanged {
                tab_id,
                can_go_back,
                can_go_forward,
            } => (
                "HistoryStateChanged".to_string(),
                format!("{tab_id}|{can_go_back}|{can_go_forward}"),
            ),
            BrowserEvent::RequestBlocked {
                tab_id,
                url,
                resource_type,
            } => (
                "RequestBlockedNative".to_string(),
                format!("{tab_id}|{url}|{resource_type}"),
            ),
            BrowserEvent::ConsoleMessage {
                tab_id,
                level,
                message,
                source_id,
                line_number,
            } => (
                "ConsoleMessage".to_string(),
                format!(
                    "{tab_id}|{level}|{message}|{}|{}",
                    source_id.clone().unwrap_or_default(),
                    line_number.map(|value| value.to_string()).unwrap_or_default()
                ),
            ),
        }
    }
}
