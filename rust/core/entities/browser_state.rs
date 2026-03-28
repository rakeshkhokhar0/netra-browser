use serde::{Deserialize, Serialize};

use crate::core::entities::tab::Tab;

/// Serializable snapshot of the browser state owned by the Rust core.
///
/// This entity is designed specifically for FFI and read-only consumers such
/// as the Flutter shell. It captures the current active tab identifier and the
/// full ordered tab list without exposing any controller, bridge, or runtime
/// synchronization details.
///
/// The struct intentionally contains no behavior. It exists only as a stable
/// data contract that can be serialized and sent across process or language
/// boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    /// Identifier of the tab currently marked active by the Rust core.
    ///
    /// A value of `None` indicates that the browser currently has no active
    /// tab, which can occur before the first tab is created or after all tabs
    /// have been closed.
    pub active_tab_id: Option<String>,

    /// Ordered snapshot of every known tab owned by the browser core.
    ///
    /// Each tab record contains the state that the Flutter shell needs to
    /// render tab chrome, URL state, loading state, navigation affordances,
    /// and blocked-request counts.
    pub tabs: Vec<Tab>,
}
