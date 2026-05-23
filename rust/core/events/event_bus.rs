use std::sync::{Arc, Mutex};

use crate::core::entities::browser_event::BrowserEvent;
use crate::core::entities::tab::Tab;

/// Internal browser events used for synchronous module-to-module communication.
///
/// This enum defines the shared event surface for core browser workflows.
/// Each variant carries only the payload required by subscribers to react
/// deterministically.
#[derive(Debug, Clone)]
pub enum Event {
    /// Emitted when a native browser event is routed into the Rust core.
    ///
    /// Payload: `(sequence_number, BrowserEvent, current_tab_state)`
    BrowserEvent(u32, BrowserEvent, Option<Tab>),
    /// Emitted when a new tab is created.
    TabCreated(String),
    /// Emitted when an existing tab is closed.
    TabClosed(String),
    /// Emitted when a tab is suspended.
    TabSuspended(String),
    /// Emitted when a previously suspended tab is resumed.
    TabResumed(String),
    /// Emitted when a network request is blocked.
    ///
    /// Payload:
    /// - blocked request URL
    BlockedRequest(String),
}

/// Synchronous publish/subscribe event bus for internal browser communication.
///
/// The bus stores subscriber closures and delivers each published event to all
/// registered handlers in subscription order. Dispatch is synchronous and
/// deterministic: each handler is invoked immediately on the calling thread.
#[derive(Clone)]
pub struct EventBus {
    /// Subscriber registry for event handlers.
    ///
    /// Each handler is an opaque closure that receives events by reference.
    pub subscribers: Arc<Mutex<Vec<Arc<dyn Fn(&Event) + Send + Sync>>>>,
}

impl EventBus {
    /// Creates an empty event bus with no subscribers.
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Registers a new subscriber closure.
    ///
    /// The handler is appended to the registry and will receive all future
    /// published events.
    pub fn subscribe(&self, handler: Box<dyn Fn(&Event) + Send + Sync>) {
        let handler: Arc<dyn Fn(&Event) + Send + Sync> = Arc::new(move |event| handler(event));

        match self.subscribers.lock() {
            Ok(mut subscribers) => {
                subscribers.push(handler);
            }
            Err(error) => {
                eprintln!("[EventBus] subscribers mutex poisoned during subscribe: {error}");
            }
        }
    }

    /// Publishes an event to all registered subscribers.
    ///
    /// Dispatch semantics:
    /// - events are delivered synchronously
    /// - handlers are invoked in registration order
    /// - each handler receives a shared reference to the same event instance
    pub fn publish(&self, event: Event) {
        let subscribers = match self.subscribers.lock() {
            Ok(subscribers) => subscribers.clone(),
            Err(error) => {
                eprintln!("[EventBus] subscribers mutex poisoned during publish: {error}");
                return;
            }
        };

        for subscriber in subscribers {
            subscriber(&event);
        }
    }
}
