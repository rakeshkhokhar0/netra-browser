use crate::browser::browser_controller::{get_browser_controller, BrowserController};
use crate::core::entities::browser_event::BrowserEvent;
use crate::core::entities::browser_state::BrowserState;
use crate::core::entities::tab::Tab;
use crate::core::error::NetraError;

/// Creates a new browser tab through the shared [BrowserController].
///
/// This FFI entry point performs no business logic. It logs the call and
/// delegates creation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn create_tab() -> Result<Tab, NetraError> {
    println!("[FFI] create_tab called");
    with_controller(BrowserController::create_tab)
}

/// Closes an existing browser tab addressed by [tab_id].
///
/// This FFI entry point validates the incoming tab identifier, logs the call,
/// and delegates the close operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn close_tab(tab_id: String) -> Result<(), NetraError> {
    println!("[FFI] close_tab called: {tab_id}");
    validate_tab_id(&tab_id)?;
    with_controller(|controller| controller.close_tab(tab_id))
}

/// Marks the provided tab as the active browser tab.
///
/// This FFI entry point validates the incoming tab identifier, logs the call,
/// and delegates active-tab switching to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn set_active_tab(tab_id: String) -> Result<(), NetraError> {
    println!("[FFI] set_active_tab called: {tab_id}");
    validate_tab_id(&tab_id)?;
    with_controller(|controller| controller.set_active_tab(tab_id))
}

/// Loads the provided [url] in the tab identified by [tab_id].
///
/// This FFI entry point validates the incoming arguments, logs the call, and
/// delegates navigation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn load_url(tab_id: String, url: String) -> Result<(), NetraError> {
    println!("[FFI] load_url called: {url}");
    validate_tab_id(&tab_id)?;
    validate_url(&url)?;
    with_controller(|controller| controller.load_url(tab_id, url))
}

/// Requests backward navigation for the tab identified by [tab_id].
///
/// This FFI entry point validates the incoming tab identifier, logs the call,
/// and delegates the action to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn go_back(tab_id: String) -> Result<(), NetraError> {
    println!("[FFI] go_back called: {tab_id}");
    validate_tab_id(&tab_id)?;
    with_controller(|controller| controller.go_back_for_tab(tab_id))
}

/// Requests forward navigation for the tab identified by [tab_id].
///
/// This FFI entry point validates the incoming tab identifier, logs the call,
/// and delegates the action to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn go_forward(tab_id: String) -> Result<(), NetraError> {
    println!("[FFI] go_forward called: {tab_id}");
    validate_tab_id(&tab_id)?;
    with_controller(|controller| controller.go_forward_for_tab(tab_id))
}

/// Reloads the currently visible document in the tab identified by [tab_id].
///
/// This FFI entry point validates the incoming tab identifier, logs the call,
/// and delegates the action to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn reload(tab_id: String) -> Result<(), NetraError> {
    println!("[FFI] reload called: {tab_id}");
    validate_tab_id(&tab_id)?;
    with_controller(|controller| controller.reload_tab(tab_id))
}

/// Stops the current loading operation in the tab identified by [tab_id].
///
/// This FFI entry point validates the incoming tab identifier, logs the call,
/// and delegates the action to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn stop_loading(tab_id: String) -> Result<(), NetraError> {
    println!("[FFI] stop_loading called: {tab_id}");
    validate_tab_id(&tab_id)?;
    with_controller(|controller| controller.stop_loading(tab_id))
}

/// Routes a typed native browser event into the Rust browser controller.
///
/// This FFI entry point performs no business logic. It logs the received event
/// and delegates event publication to the shared browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn handle_event(event: BrowserEvent) -> Result<(), NetraError> {
    println!("[RUST EVENT] {:?}", event);
    with_controller(|controller| {
        controller.handle_browser_event(event);
        Ok(())
    })
}

/// Returns the full browser-state snapshot owned by the Rust core.
///
/// Flutter uses this read-only snapshot to render tab state without mutating
/// a duplicated local browser model.
#[flutter_rust_bridge::frb(sync)]
pub fn get_browser_state() -> Result<BrowserState, NetraError> {
    println!("[FFI] get_browser_state called");
    with_controller(|controller| Ok(controller.get_browser_state()))
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

/// Executes a closure against the shared browser controller mutex.
fn with_controller<T>(
    operation: impl FnOnce(&mut BrowserController) -> Result<T, NetraError>,
) -> Result<T, NetraError> {
    let controller = get_browser_controller();
    let mut controller = controller
        .lock()
        .map_err(|_| NetraError::OperationFailed("browser controller lock poisoned".to_string()))?;
    operation(&mut controller)
}
