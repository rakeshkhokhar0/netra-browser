use crate::browser::browser_controller::get_browser_controller;
use crate::core::error::NetraError;

/// Creates a new browser frame using the provided tab identifier.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn create_frame(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    get_browser_controller().create_frame(tab_id)
}

/// Destroys an existing browser frame using the provided tab identifier.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn destroy_frame(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    get_browser_controller().destroy_frame(tab_id)
}

/// Loads a URL into the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb]
pub async fn load_url(tab_id: String, url: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    validate_url(&url)?;
    get_browser_controller().load_url(tab_id, url).await
}

/// Requests backward navigation for the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn go_back(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    get_browser_controller().go_back(tab_id)
}

/// Requests forward navigation for the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn go_forward(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    get_browser_controller().go_forward(tab_id)
}

/// Requests a reload of the current page in the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb]
pub async fn reload(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    get_browser_controller().reload(tab_id).await
}

/// Requests that the specified browser frame stop its current loading work.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn stop_loading(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    get_browser_controller().stop_loading(tab_id)
}

/// Executes JavaScript within the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb]
pub async fn execute_script(tab_id: String, js: String) -> Result<String, NetraError> {
    validate_tab_id(&tab_id)?;
    get_browser_controller().execute_script(tab_id, js).await
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
