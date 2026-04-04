use crate::browser::browser_controller::BrowserController;
use crate::core::entities::browser_event::BrowserEvent;
use crate::core::entities::browser_state::BrowserState;
use crate::core::entities::tab::Tab;
use crate::core::error::NetraError;

/// Creates a new browser tab through the shared [BrowserController].
///
/// This FFI-facing function is intentionally thin:
/// - validates no additional business rules
/// - does not access browser state directly
/// - does not acquire the browser-controller mutex
/// - delegates creation to [BrowserController::create_tab_safe]
///
/// The created [Tab] is returned so existing bridge layers can reuse the
/// Rust-generated tab identifier without generating their own.
#[flutter_rust_bridge::frb(sync)]
pub fn create_tab() -> Result<Tab, NetraError> {
    BrowserController::create_tab_safe()
}

/// Closes the browser tab identified by [tab_id].
///
/// This function performs only input validation before delegating to
/// [BrowserController::close_tab_safe].
#[flutter_rust_bridge::frb(sync)]
pub fn close_tab(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    BrowserController::close_tab_safe(tab_id)
}

/// Marks the browser tab identified by [tab_id] as active.
///
/// This function performs only input validation before delegating to
/// [BrowserController::set_active_tab_safe].
#[flutter_rust_bridge::frb(sync)]
pub fn set_active_tab(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    BrowserController::set_active_tab_safe(tab_id)
}

/// Navigates the browser tab identified by [tab_id] to [url].
///
/// This function performs only input validation before delegating to
/// [BrowserController::navigate_safe].
#[flutter_rust_bridge::frb(sync)]
pub fn navigate(tab_id: String, url: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    validate_url(&url)?;
    BrowserController::navigate_safe(tab_id, url)
}

/// Compatibility wrapper retained for C-boundary callers that still use the
/// historical `load_url` name.
///
/// This wrapper remains a pure delegation layer and forwards directly to
/// [navigate].
#[flutter_rust_bridge::frb(sync)]
pub fn load_url(tab_id: String, url: String) -> Result<(), NetraError> {
    navigate(tab_id, url)
}

/// Requests backward navigation for the browser tab identified by [tab_id].
///
/// This function performs only input validation before delegating to
/// [BrowserController::go_back_safe].
#[flutter_rust_bridge::frb(sync)]
pub fn go_back(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    BrowserController::go_back_safe(tab_id)
}

/// Requests forward navigation for the browser tab identified by [tab_id].
///
/// This function performs only input validation before delegating to
/// [BrowserController::go_forward_safe].
#[flutter_rust_bridge::frb(sync)]
pub fn go_forward(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    BrowserController::go_forward_safe(tab_id)
}

/// Reloads the currently visible document in the browser tab identified by
/// [tab_id].
///
/// This function performs only input validation before delegating to
/// [BrowserController::reload_safe].
#[flutter_rust_bridge::frb(sync)]
pub fn reload(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    BrowserController::reload_safe(tab_id)
}

/// Stops the current loading operation in the browser tab identified by
/// [tab_id].
///
/// This function performs only input validation before delegating to
/// [BrowserController::stop_loading_safe].
#[flutter_rust_bridge::frb(sync)]
pub fn stop_loading(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    BrowserController::stop_loading_safe(tab_id)
}

/// Routes a typed native browser event into the Rust browser controller.
///
/// This function is a thin handoff to
/// [BrowserController::handle_browser_event_safe] and does not interpret or
/// transform event semantics locally.
#[flutter_rust_bridge::frb(sync)]
pub fn handle_event(event: BrowserEvent) -> Result<(), NetraError> {
    BrowserController::handle_browser_event_safe(event)
}

/// Returns the full browser-state snapshot owned by the Rust core.
///
/// This read-only FFI function delegates directly to
/// [BrowserController::get_browser_state_safe] so Flutter can render the
/// authoritative Rust state without mutating a duplicate local model.
#[flutter_rust_bridge::frb(sync)]
pub fn get_browser_state() -> Result<BrowserState, NetraError> {
    BrowserController::get_browser_state_safe()
}

/// Validates that a tab identifier is present before delegation.
fn validate_tab_id(tab_id: &str) -> Result<(), NetraError> {
    if tab_id.trim().is_empty() {
        return Err(NetraError::InvalidInput("tab_id must not be empty".to_string()));
    }

    Ok(())
}

/// Validates that a URL is present before delegation.
fn validate_url(url: &str) -> Result<(), NetraError> {
    if url.trim().is_empty() {
        return Err(NetraError::InvalidInput("url must not be empty".to_string()));
    }

    Ok(())
}
