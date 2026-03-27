use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::core::error::NetraError;

/// Coordinates high-level browser actions requested through the Rust FFI layer.
///
/// This restored controller is intentionally minimal. It preserves the stable
/// API surface that FFI calls delegate into while future navigation, privacy,
/// and storage logic is rebuilt around it.
pub struct BrowserController {
    tabs: Mutex<HashSet<String>>,
}

impl BrowserController {
    /// Creates a new controller instance.
    pub fn new() -> Self {
        Self {
            tabs: Mutex::new(HashSet::new()),
        }
    }

    /// Creates a new frame record for the supplied tab identifier.
    pub fn create_frame(&self, tab_id: String) -> Result<(), NetraError> {
        let mut tabs = self
            .tabs
            .lock()
            .map_err(|_| NetraError::OperationFailed("tab state lock poisoned".to_string()))?;
        tabs.insert(tab_id);
        Ok(())
    }

    /// Removes an existing frame record.
    pub fn destroy_frame(&self, tab_id: String) -> Result<(), NetraError> {
        let mut tabs = self
            .tabs
            .lock()
            .map_err(|_| NetraError::OperationFailed("tab state lock poisoned".to_string()))?;

        if tabs.remove(&tab_id) {
            Ok(())
        } else {
            Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")))
        }
    }

    /// Loads a URL for the supplied tab identifier.
    pub async fn load_url(&self, tab_id: String, _url: String) -> Result<(), NetraError> {
        self.ensure_tab_exists(&tab_id)
    }

    /// Requests back navigation for the supplied tab identifier.
    pub fn go_back(&self, tab_id: String) -> Result<(), NetraError> {
        self.ensure_tab_exists(&tab_id)
    }

    /// Requests forward navigation for the supplied tab identifier.
    pub fn go_forward(&self, tab_id: String) -> Result<(), NetraError> {
        self.ensure_tab_exists(&tab_id)
    }

    /// Requests reload for the supplied tab identifier.
    pub async fn reload(&self, tab_id: String) -> Result<(), NetraError> {
        self.ensure_tab_exists(&tab_id)
    }

    /// Requests stop-loading for the supplied tab identifier.
    pub fn stop_loading(&self, tab_id: String) -> Result<(), NetraError> {
        self.ensure_tab_exists(&tab_id)
    }

    /// Executes JavaScript for the supplied tab identifier.
    pub async fn execute_script(&self, tab_id: String, _js: String) -> Result<String, NetraError> {
        self.ensure_tab_exists(&tab_id)?;
        Ok(String::new())
    }

    fn ensure_tab_exists(&self, tab_id: &str) -> Result<(), NetraError> {
        let tabs = self
            .tabs
            .lock()
            .map_err(|_| NetraError::OperationFailed("tab state lock poisoned".to_string()))?;

        if tabs.contains(tab_id) {
            Ok(())
        } else {
            Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")))
        }
    }
}

/// Returns the shared browser controller used by FFI entry points.
pub fn get_browser_controller() -> &'static BrowserController {
    static CONTROLLER: OnceLock<BrowserController> = OnceLock::new();
    CONTROLLER.get_or_init(BrowserController::new)
}
