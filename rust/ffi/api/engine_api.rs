use crate::browser::browser_controller::get_browser_controller;
use crate::core::error::NetraError;

/// Creates a new browser frame using the provided tab identifier.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn create_frame(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    Err(NetraError::OperationFailed(
        "create_frame requires external tab-id injection support and is deferred to a later phase"
            .to_string(),
    ))
}

/// Destroys an existing browser frame using the provided tab identifier.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn destroy_frame(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    with_controller(|controller| {
        ensure_tab_exists(controller, &tab_id)?;
        controller.close_tab(tab_id);
        Ok(())
    })
}

/// Loads a URL into the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb]
pub async fn load_url(tab_id: String, url: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    validate_url(&url)?;
    with_controller(|controller| {
        ensure_tab_exists(controller, &tab_id)?;
        controller.set_active_tab(tab_id);
        controller.navigate(url).map(|_| ()).ok_or_else(|| {
            NetraError::OperationFailed(
                "failed to navigate active tab using current browser-state pipeline".to_string(),
            )
        })
    })
}

/// Requests backward navigation for the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn go_back(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    with_controller(|controller| {
        ensure_tab_exists(controller, &tab_id)?;
        controller.set_active_tab(tab_id);
        controller.go_back().map(|_| ()).ok_or_else(|| {
            NetraError::OperationFailed(
                "back navigation is not currently available for the active tab".to_string(),
            )
        })
    })
}

/// Requests forward navigation for the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn go_forward(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    with_controller(|controller| {
        ensure_tab_exists(controller, &tab_id)?;
        controller.set_active_tab(tab_id);
        controller.go_forward().map(|_| ()).ok_or_else(|| {
            NetraError::OperationFailed(
                "forward navigation is not currently available for the active tab".to_string(),
            )
        })
    })
}

/// Requests a reload of the current page in the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb]
pub async fn reload(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    Err(NetraError::OperationFailed(
        "reload is not implemented in the current browser core phase".to_string(),
    ))
}

/// Requests that the specified browser frame stop its current loading work.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb(sync)]
pub fn stop_loading(tab_id: String) -> Result<(), NetraError> {
    validate_tab_id(&tab_id)?;
    Err(NetraError::OperationFailed(
        "stop_loading is not implemented in the current browser core phase".to_string(),
    ))
}

/// Executes JavaScript within the specified browser frame.
///
/// This FFI entry point performs only minimal input validation and delegates
/// the operation to the browser controller.
#[flutter_rust_bridge::frb]
pub async fn execute_script(tab_id: String, js: String) -> Result<String, NetraError> {
    validate_tab_id(&tab_id)?;
    let _ = js;
    Err(NetraError::OperationFailed(
        "execute_script is not implemented in the current browser core phase".to_string(),
    ))
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
    operation: impl FnOnce(&mut crate::browser::browser_controller::BrowserController) -> Result<T, NetraError>,
) -> Result<T, NetraError> {
    let controller = get_browser_controller();
    let mut controller = controller
        .lock()
        .map_err(|_| NetraError::OperationFailed("browser controller lock poisoned".to_string()))?;
    operation(&mut controller)
}

/// Ensures a tab exists in the current browser-state snapshot before a
/// tab-addressed FFI operation proceeds.
fn ensure_tab_exists(
    controller: &crate::browser::browser_controller::BrowserController,
    tab_id: &str,
) -> Result<(), NetraError> {
    if controller.get_tabs().iter().any(|tab| tab.id == tab_id) {
        Ok(())
    } else {
        Err(NetraError::NotFound(format!("tab `{tab_id}` was not found")))
    }
}
