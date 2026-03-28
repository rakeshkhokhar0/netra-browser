use std::ffi::{CString, c_char};

use once_cell::sync::OnceCell;

use crate::core::error::NetraError;

/// Native callback signature used to execute a single-string control command.
pub type NativeStringOperation = unsafe extern "C" fn(*const c_char) -> i32;

/// Native callback signature used to execute a two-string control command.
pub type NativeTwoStringOperation =
    unsafe extern "C" fn(*const c_char, *const c_char) -> i32;

/// Collection of native executor callbacks registered by the Windows runner.
///
/// Rust stores these callbacks once during process startup and then uses them
/// as the direct control surface for WebView2 operations. This allows the Rust
/// DLL to remain the real executor without requiring Flutter to forward every
/// browser control command through a MethodChannel first.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeExecutorBindings {
    /// Creates a native WebView-backed tab for the provided identifier.
    pub create_tab: Option<NativeStringOperation>,
    /// Closes the native tab identified by the provided identifier.
    pub close_tab: Option<NativeStringOperation>,
    /// Marks the provided tab as the active native tab.
    pub set_active_tab: Option<NativeStringOperation>,
    /// Loads the provided URL in the specified native tab.
    pub load_url: Option<NativeTwoStringOperation>,
    /// Requests backward navigation for the specified native tab.
    pub go_back: Option<NativeStringOperation>,
    /// Requests forward navigation for the specified native tab.
    pub go_forward: Option<NativeStringOperation>,
    /// Reloads the specified native tab.
    pub reload: Option<NativeStringOperation>,
    /// Stops loading in the specified native tab.
    pub stop_loading: Option<NativeStringOperation>,
}

static NATIVE_EXECUTOR_BINDINGS: OnceCell<NativeExecutorBindings> = OnceCell::new();

/// Registers the native executor callbacks exactly once for the process.
pub fn register(bindings: NativeExecutorBindings) -> Result<(), NetraError> {
    if NATIVE_EXECUTOR_BINDINGS.get().is_some() {
        return Ok(());
    }

    let required_callbacks = [
        bindings.create_tab.is_some(),
        bindings.close_tab.is_some(),
        bindings.set_active_tab.is_some(),
        bindings.load_url.is_some(),
        bindings.go_back.is_some(),
        bindings.go_forward.is_some(),
        bindings.reload.is_some(),
        bindings.stop_loading.is_some(),
    ];

    if required_callbacks.iter().any(|callback_present| !callback_present) {
        return Err(NetraError::InvalidInput(
            "native executor registration is missing one or more callbacks".to_string(),
        ));
    }

    NATIVE_EXECUTOR_BINDINGS.set(bindings).map_err(|_| {
        NetraError::OperationFailed(
            "native executor callbacks were already registered".to_string(),
        )
    })
}

/// Creates a native tab through the registered Windows executor.
pub fn create_tab(tab_id: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] create_tab({tab_id})");
    invoke_string_operation("create_tab", tab_id, |bindings| bindings.create_tab)
}

/// Closes a native tab through the registered Windows executor.
pub fn close_tab(tab_id: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] close_tab({tab_id})");
    invoke_string_operation("close_tab", tab_id, |bindings| bindings.close_tab)
}

/// Marks a native tab as active through the registered Windows executor.
pub fn set_active_tab(tab_id: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] set_active_tab({tab_id})");
    invoke_string_operation(
        "set_active_tab",
        tab_id,
        |bindings| bindings.set_active_tab,
    )
}

/// Loads a URL in a native tab through the registered Windows executor.
pub fn load_url(tab_id: &str, url: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] load_url({tab_id}, {url})");
    let bindings = bindings()?;
    let operation = bindings.load_url.ok_or_else(|| {
        NetraError::OperationFailed("native load_url callback is not registered".to_string())
    })?;

    let tab_id = CString::new(tab_id).map_err(|_| {
        NetraError::InvalidInput("tab_id contains an interior null byte".to_string())
    })?;
    let url = CString::new(url)
        .map_err(|_| NetraError::InvalidInput("url contains an interior null byte".to_string()))?;

    let status = unsafe { operation(tab_id.as_ptr(), url.as_ptr()) };
    if status == 1 {
        Ok(())
    } else {
        Err(NetraError::OperationFailed(
            "native load_url operation failed".to_string(),
        ))
    }
}

/// Requests backward navigation through the registered Windows executor.
pub fn go_back(tab_id: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] go_back({tab_id})");
    invoke_string_operation("go_back", tab_id, |bindings| bindings.go_back)
}

/// Requests forward navigation through the registered Windows executor.
pub fn go_forward(tab_id: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] go_forward({tab_id})");
    invoke_string_operation("go_forward", tab_id, |bindings| bindings.go_forward)
}

/// Reloads a native tab through the registered Windows executor.
pub fn reload(tab_id: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] reload({tab_id})");
    invoke_string_operation("reload", tab_id, |bindings| bindings.reload)
}

/// Stops loading in a native tab through the registered Windows executor.
pub fn stop_loading(tab_id: &str) -> Result<(), NetraError> {
    println!("[RUST -> C++] stop_loading({tab_id})");
    invoke_string_operation("stop_loading", tab_id, |bindings| bindings.stop_loading)
}

fn bindings() -> Result<&'static NativeExecutorBindings, NetraError> {
    NATIVE_EXECUTOR_BINDINGS.get().ok_or_else(|| {
        NetraError::OperationFailed(
            "native executor callbacks were not registered".to_string(),
        )
    })
}

fn invoke_string_operation(
    operation_name: &str,
    value: &str,
    resolve: impl FnOnce(&NativeExecutorBindings) -> Option<NativeStringOperation>,
) -> Result<(), NetraError> {
    let bindings = bindings()?;
    let operation = resolve(bindings).ok_or_else(|| {
        NetraError::OperationFailed(format!(
            "native {operation_name} callback is not registered"
        ))
    })?;
    let value = CString::new(value).map_err(|_| {
        NetraError::InvalidInput(format!(
            "{operation_name} argument contains an interior null byte"
        ))
    })?;

    let status = unsafe { operation(value.as_ptr()) };
    if status == 1 {
        Ok(())
    } else {
        Err(NetraError::OperationFailed(format!(
            "native {operation_name} operation failed"
        )))
    }
}
