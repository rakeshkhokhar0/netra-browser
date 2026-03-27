/// Internal browser events used for synchronous module-to-module communication.
///
/// This enum defines the shared event surface for core browser workflows.
/// Each variant carries only the payload required by subscribers to react
/// deterministically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Emitted when a new tab is created.
    TabCreated(String),
    /// Emitted when an existing tab is closed.
    TabClosed(String),
    /// Emitted when a tab is suspended.
    TabSuspended(String),
    /// Emitted when a previously suspended tab is resumed.
    TabResumed(String),
    /// Emitted when a navigation operation completes.
    ///
    /// Payload order:
    /// - tab id
    /// - resolved URL
    NavigationCompleted(String, String),
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
pub struct EventBus {
    /// Subscriber registry for event handlers.
    ///
    /// Each handler is an opaque closure that receives events by reference.
    pub subscribers: Vec<Box<dyn Fn(&Event)>>,
}

impl EventBus {
    /// Creates an empty event bus with no subscribers.
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    /// Registers a new subscriber closure.
    ///
    /// The handler is appended to the registry and will receive all future
    /// published events.
    pub fn subscribe(&mut self, handler: Box<dyn Fn(&Event)>) {
        self.subscribers.push(handler);
    }

    /// Publishes an event to all registered subscribers.
    ///
    /// Dispatch semantics:
    /// - events are delivered synchronously
    /// - handlers are invoked in registration order
    /// - each handler receives a shared reference to the same event instance
    pub fn publish(&self, event: Event) {
        for subscriber in &self.subscribers {
            subscriber(&event);
        }
    }
}
