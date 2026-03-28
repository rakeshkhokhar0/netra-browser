#![allow(unexpected_cfgs)]

use std::ffi::{CStr, CString, c_char};

#[path = "../browser/mod.rs"]
pub mod browser;
#[path = "../core/mod.rs"]
pub mod core;
#[path = "../network/mod.rs"]
pub mod network;
#[path = "../platform/mod.rs"]
pub mod platform;
#[path = "../privacy/mod.rs"]
pub mod privacy;
#[path = "../services/mod.rs"]
pub mod services;
#[path = "../storage/mod.rs"]
pub mod storage;
#[path = "../utils/mod.rs"]
pub mod utils;

pub mod api;
pub mod native_control;

static NETRA_CONNECTION_MESSAGE: &[u8] = b"Netra Rust bridge connected\0";

/// Returns a simple integer used to prove the Rust DLL loaded successfully.
#[unsafe(no_mangle)]
pub extern "C" fn netra_connection_smoke_test() -> i32 {
    2026
}

/// Returns a static bridge message used by the Flutter smoke test.
#[unsafe(no_mangle)]
pub extern "C" fn netra_connection_message() -> *const c_char {
    NETRA_CONNECTION_MESSAGE.as_ptr().cast()
}

/// Registers the native Windows executor callbacks used by the Rust control
/// layer.
#[unsafe(no_mangle)]
pub extern "C" fn netra_register_native_executor(
    bindings: native_control::NativeExecutorBindings,
) -> i32 {
    match native_control::register(bindings) {
        Ok(()) => 1,
        Err(error) => {
            eprintln!("[FFI:C] netra_register_native_executor failed: {error}");
            0
        }
    }
}

/// Creates a Rust-owned browser tab and returns its identifier as an allocated
/// C string.
///
/// The caller must eventually release the returned pointer with
/// [netra_string_free].
#[unsafe(no_mangle)]
pub extern "C" fn netra_create_tab_id() -> *mut c_char {
    match api::engine_api::create_tab() {
        Ok(tab) => CString::new(tab.id)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut()),
        Err(error) => {
            eprintln!("[FFI:C] netra_create_tab_id failed: {error}");
            std::ptr::null_mut()
        }
    }
}

/// Closes the Rust-owned browser tab identified by the provided C string.
#[unsafe(no_mangle)]
pub extern "C" fn netra_close_tab(tab_id: *const c_char) -> i32 {
    with_string_arg(tab_id, api::engine_api::close_tab)
}

/// Switches the Rust browser core to the provided active tab identifier.
#[unsafe(no_mangle)]
pub extern "C" fn netra_set_active_tab(tab_id: *const c_char) -> i32 {
    with_string_arg(tab_id, api::engine_api::set_active_tab)
}

/// Loads a URL in the Rust-owned tab identified by the provided C strings.
#[unsafe(no_mangle)]
pub extern "C" fn netra_load_url(tab_id: *const c_char, url: *const c_char) -> i32 {
    with_two_string_args(tab_id, url, api::engine_api::load_url)
}

/// Requests backward navigation for the Rust-owned tab identified by the
/// provided C string.
#[unsafe(no_mangle)]
pub extern "C" fn netra_go_back(tab_id: *const c_char) -> i32 {
    with_string_arg(tab_id, api::engine_api::go_back)
}

/// Requests forward navigation for the Rust-owned tab identified by the
/// provided C string.
#[unsafe(no_mangle)]
pub extern "C" fn netra_go_forward(tab_id: *const c_char) -> i32 {
    with_string_arg(tab_id, api::engine_api::go_forward)
}

/// Reloads the Rust-owned tab identified by the provided C string.
#[unsafe(no_mangle)]
pub extern "C" fn netra_reload(tab_id: *const c_char) -> i32 {
    with_string_arg(tab_id, api::engine_api::reload)
}

/// Stops loading in the Rust-owned tab identified by the provided C string.
#[unsafe(no_mangle)]
pub extern "C" fn netra_stop_loading(tab_id: *const c_char) -> i32 {
    with_string_arg(tab_id, api::engine_api::stop_loading)
}

/// Routes a native browser event, encoded as JSON, into the Rust core.
#[unsafe(no_mangle)]
pub extern "C" fn netra_handle_event_json(event_json: *const c_char) -> i32 {
    let Some(event_json) = read_utf8_string(event_json) else {
        return 0;
    };

    let event = match decode_browser_event(&event_json) {
        Ok(event) => event,
        Err(error) => {
            eprintln!("[FFI:C] failed to decode browser event JSON: {error}");
            return 0;
        }
    };

    match api::engine_api::handle_event(event) {
        Ok(()) => 1,
        Err(error) => {
            eprintln!("[FFI:C] handle_event failed: {error}");
            0
        }
    }
}

/// Returns the current Rust-owned browser-state snapshot encoded as JSON.
///
/// The caller must eventually release the returned pointer with
/// [netra_string_free].
#[unsafe(no_mangle)]
pub extern "C" fn netra_get_browser_state_json() -> *mut c_char {
    let browser_state = match api::engine_api::get_browser_state() {
        Ok(browser_state) => browser_state,
        Err(error) => {
            eprintln!("[FFI:C] netra_get_browser_state_json failed: {error}");
            return std::ptr::null_mut();
        }
    };

    match serde_json::to_string(&browser_state)
        .ok()
        .and_then(|json| CString::new(json).ok())
    {
        Some(json) => json.into_raw(),
        None => {
            eprintln!("[FFI:C] failed to serialize browser state");
            std::ptr::null_mut()
        }
    }
}

/// Frees a C string previously allocated by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn netra_string_free(pointer: *mut c_char) {
    if pointer.is_null() {
        return;
    }

    let _ = unsafe { CString::from_raw(pointer) };
}

fn with_string_arg(
    raw_value: *const c_char,
    operation: impl FnOnce(String) -> Result<(), crate::core::error::NetraError>,
) -> i32 {
    let Some(value) = read_utf8_string(raw_value) else {
        return 0;
    };

    match operation(value) {
        Ok(()) => 1,
        Err(error) => {
            eprintln!("[FFI:C] operation failed: {error}");
            0
        }
    }
}

fn with_two_string_args(
    first_raw: *const c_char,
    second_raw: *const c_char,
    operation: impl FnOnce(String, String) -> Result<(), crate::core::error::NetraError>,
) -> i32 {
    let Some(first) = read_utf8_string(first_raw) else {
        return 0;
    };
    let Some(second) = read_utf8_string(second_raw) else {
        return 0;
    };

    match operation(first, second) {
        Ok(()) => 1,
        Err(error) => {
            eprintln!("[FFI:C] operation failed: {error}");
            0
        }
    }
}

fn read_utf8_string(raw_value: *const c_char) -> Option<String> {
    if raw_value.is_null() {
        eprintln!("[FFI:C] received null string pointer");
        return None;
    }

    let c_str = unsafe { CStr::from_ptr(raw_value) };
    match c_str.to_str() {
        Ok(value) => Some(value.to_string()),
        Err(error) => {
            eprintln!("[FFI:C] invalid UTF-8 string: {error}");
            None
        }
    }
}

fn decode_browser_event(
    event_json: &str,
) -> Result<crate::core::entities::browser_event::BrowserEvent, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(event_json)?;
    let event_type = value
        .get("type")
        .and_then(|field| field.as_str())
        .unwrap_or_default();
    let tab_id = value
        .get("tabId")
        .or_else(|| value.get("tab_id"))
        .and_then(|field| field.as_str())
        .unwrap_or_default()
        .to_string();

    use crate::core::entities::browser_event::BrowserEvent;

    let event = match event_type {
        "navigationStarting" => BrowserEvent::NavigationStarted {
            tab_id,
            url: string_field(&value, "url"),
            is_same_document: bool_field(&value, "isSameDocument"),
        },
        "TabCreated" => BrowserEvent::FrameCreated { tab_id },
        "TabClosed" => BrowserEvent::FrameDestroyed { tab_id },
        "contentLoading" => BrowserEvent::LoadStarted { tab_id },
        "navigationCompleted" => BrowserEvent::NavigationCompleted {
            tab_id,
            url: string_field(&value, "url"),
            is_same_document: bool_field(&value, "isSameDocument"),
        },
        "titleChanged" => BrowserEvent::TitleChanged {
            tab_id,
            title: string_field(&value, "title"),
        },
        "faviconChanged" => BrowserEvent::ConsoleMessage {
            tab_id,
            level: "info".to_string(),
            message: format!("favicon changed: {}", string_field(&value, "faviconUrl")),
            source_id: None,
            line_number: None,
        },
        "historyChanged" => BrowserEvent::HistoryStateChanged {
            tab_id,
            can_go_back: bool_field_alias(&value, "canGoBack", "can_go_back"),
            can_go_forward: bool_field_alias(&value, "canGoForward", "can_go_forward"),
        },
        "requestBlocked" => BrowserEvent::RequestBlocked {
            tab_id,
            url: string_field(&value, "url"),
            resource_type: string_field_alias(&value, "resourceType", "resource_type"),
        },
        "tabCrashed" => BrowserEvent::NavigationFailed {
            tab_id,
            url: string_field(&value, "url"),
            error_code: -1,
            description: "tab crashed".to_string(),
        },
        "engineReady" => BrowserEvent::FrameCreated { tab_id },
        "urlChanged" => BrowserEvent::UrlChanged {
            tab_id,
            url: string_field(&value, "url"),
        },
        _ => BrowserEvent::ConsoleMessage {
            tab_id,
            level: "debug".to_string(),
            message: format!("unmapped event: {event_type}"),
            source_id: None,
            line_number: None,
        },
    };

    Ok(event)
}

fn string_field(value: &serde_json::Value, field: &str) -> String {
    value
        .get(field)
        .and_then(|field| field.as_str())
        .unwrap_or_default()
        .to_string()
}

fn string_field_alias(value: &serde_json::Value, primary: &str, fallback: &str) -> String {
    value
        .get(primary)
        .or_else(|| value.get(fallback))
        .and_then(|field| field.as_str())
        .unwrap_or_default()
        .to_string()
}

fn bool_field(value: &serde_json::Value, field: &str) -> bool {
    value
        .get(field)
        .and_then(|field| field.as_bool())
        .unwrap_or(false)
}

fn bool_field_alias(value: &serde_json::Value, primary: &str, fallback: &str) -> bool {
    value
        .get(primary)
        .or_else(|| value.get(fallback))
        .and_then(|field| field.as_bool())
        .unwrap_or(false)
}
