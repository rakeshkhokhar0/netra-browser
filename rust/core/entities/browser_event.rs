use serde::{Deserialize, Serialize};

/// A strongly typed event representing state changes within an active browser tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserEvent {
    /// Fired when an initial frame is successfully allocated.
    FrameCreated {
        tab_id: String,
    },
    /// Fired when a frame is completely decommissioned.
    FrameDestroyed {
        tab_id: String,
    },
    /// Fired when the visual window title of a frame transitions.
    TitleChanged {
        tab_id: String,
        title: String,
    },
    /// Fired when the URL of a frame changes natively.
    UrlChanged {
        tab_id: String,
        url: String,
    },
    /// Fired when a navigation lifecycle sequence begins.
    NavigationStarted {
        tab_id: String,
        url: String,
        is_same_document: bool,
    },
    /// Fired when a navigation lifecycle sequence completes successfully.
    NavigationCompleted {
        tab_id: String,
        url: String,
        is_same_document: bool,
    },
    /// Fired when a navigation event throws an internal or native framework error.
    NavigationFailed {
        tab_id: String,
        url: String,
        error_code: i32,
        description: String,
    },
    /// Fired when a frame begins downloading page resources.
    LoadStarted {
        tab_id: String,
    },
    /// Fired when a frame completes downloading page resources.
    LoadFinished {
        tab_id: String,
    },
    /// Fired when the native frame history stack transitions its bounds.
    HistoryStateChanged {
        tab_id: String,
        can_go_back: bool,
        can_go_forward: bool,
    },
    /// Fired when the engine decides to reject an interceptable request sequence.
    RequestBlocked {
        tab_id: String,
        url: String,
        resource_type: String,
    },
    /// Low-level debug or native logging propagation.
    ConsoleMessage {
        tab_id: String,
        level: String,
        message: String,
        source_id: Option<String>,
        line_number: Option<i32>,
    },
}
