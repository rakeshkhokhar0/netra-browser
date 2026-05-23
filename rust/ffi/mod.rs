#![allow(unexpected_cfgs)]

use std::ffi::{CStr, CString, c_char};
use std::sync::atomic::{AtomicBool, Ordering};

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
static EVENT_DISPATCHER_REGISTERED: AtomicBool = AtomicBool::new(false);

const EVENT_NAVIGATION_STARTED: i32 = 1;
const EVENT_FRAME_CREATED: i32 = 2;
const EVENT_FRAME_DESTROYED: i32 = 3;
const EVENT_LOAD_STARTED: i32 = 4;
const EVENT_NAVIGATION_COMPLETED: i32 = 5;
const EVENT_TITLE_CHANGED: i32 = 6;
const EVENT_HISTORY_STATE_CHANGED: i32 = 7;
const EVENT_REQUEST_BLOCKED: i32 = 8;
const EVENT_NAVIGATION_FAILED: i32 = 9;
const EVENT_URL_CHANGED: i32 = 10;
const EVENT_LOAD_FINISHED: i32 = 11;
const EVENT_FAVICON_CHANGED: i32 = 12;
const EVENT_TAB_CRASHED: i32 = 13;

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

/// Updates native embedding bounds for the provided tab.
#[unsafe(no_mangle)]
pub extern "C" fn netra_set_bounds(
    tab_id: *const c_char,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> i32 {
    if tab_id.is_null() {
        return 0;
    }
    let tab = unsafe { std::ffi::CStr::from_ptr(tab_id) }
        .to_string_lossy()
        .into_owned();
    if api::engine_api::set_bounds(tab, x, y, width, height).is_ok() {
        1
    } else {
        0
    }
}

fn with_string_arg<F>(ptr: *const c_char, operation: F) -> i32
where
    F: FnOnce(String) -> Result<(), crate::core::error::NetraError>,
{
    if ptr.is_null() {
        return 0;
    }
    let arg = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
    if operation(arg).is_ok() { 1 } else { 0 }
}

fn with_two_string_args<F>(ptr1: *const c_char, ptr2: *const c_char, operation: F) -> i32
where
    F: FnOnce(String, String) -> Result<(), crate::core::error::NetraError>,
{
    if ptr1.is_null() || ptr2.is_null() {
        return 0;
    }
    let arg1 = unsafe { std::ffi::CStr::from_ptr(ptr1) }.to_string_lossy().into_owned();
    let arg2 = unsafe { std::ffi::CStr::from_ptr(ptr2) }.to_string_lossy().into_owned();
    if operation(arg1, arg2).is_ok() { 1 } else { 0 }
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

    println!("[Rust] State snapshot generated");

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
pub unsafe extern "C" fn netra_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    let _ = unsafe { CString::from_raw(ptr) };
}

#[repr(C)]
pub struct FfiTabState {
    pub tab_id: *mut c_char,
    pub url: *mut c_char,
    pub title: *mut c_char,
    pub favicon_url: *mut c_char,
    pub is_active: u8,
    pub is_loading: u8,
    pub can_go_back: u8,
    pub can_go_forward: u8,
    pub is_suspended: u8,
    pub blocked_count: u32,
    pub sequence_number: u32,
}

#[repr(C)]
pub struct FfiBrowserEvent {
    pub sequence_number: u32,
    pub event_type: i32,
    pub tab_id: *mut c_char,
    pub favicon_url: *mut c_char,
    pub tab_state: FfiTabState,
}

#[repr(C)]
pub struct FfiNativeBrowserEvent {
    pub sequence_number: u32,
    pub event_type: i32,
    pub tab_id: *mut c_char,
    pub primary_string: *mut c_char,
    pub secondary_string: *mut c_char,
    pub int_value: i32,
    pub bool_value: u8,
}

fn copy_c_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    let c_string = unsafe { CStr::from_ptr(ptr) };
    Some(c_string.to_string_lossy().into_owned())
}

/// Routes a native browser event directly into the Rust core.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn netra_handle_native_event(event: FfiNativeBrowserEvent) -> i32 {
    let tab_id = copy_c_string(event.tab_id.cast_const()).unwrap_or_default();
    let primary_string = copy_c_string(event.primary_string.cast_const()).unwrap_or_default();
    let secondary_string = copy_c_string(event.secondary_string.cast_const()).unwrap_or_default();

    use crate::core::entities::browser_event::BrowserEvent;
    let browser_event = match event.event_type {
        EVENT_NAVIGATION_STARTED => BrowserEvent::NavigationStarted {
            tab_id,
            url: primary_string,
            is_same_document: event.bool_value != 0,
        },
        EVENT_FRAME_CREATED => BrowserEvent::FrameCreated { tab_id },
        EVENT_FRAME_DESTROYED => BrowserEvent::FrameDestroyed { tab_id },
        EVENT_LOAD_STARTED => BrowserEvent::LoadStarted { tab_id },
        EVENT_NAVIGATION_COMPLETED => BrowserEvent::NavigationCompleted {
            tab_id,
            url: primary_string,
            is_same_document: event.bool_value != 0,
        },
        EVENT_TITLE_CHANGED => BrowserEvent::TitleChanged {
            tab_id,
            title: primary_string,
        },
        EVENT_FAVICON_CHANGED => BrowserEvent::FaviconChanged {
            tab_id,
            favicon_url: primary_string,
        },
        EVENT_HISTORY_STATE_CHANGED => BrowserEvent::HistoryStateChanged {
            tab_id,
            can_go_back: (event.int_value & 1) != 0,
            can_go_forward: (event.int_value & 2) != 0,
        },
        EVENT_REQUEST_BLOCKED => BrowserEvent::RequestBlocked {
            tab_id,
            url: primary_string,
            resource_type: secondary_string,
        },
        EVENT_NAVIGATION_FAILED => BrowserEvent::NavigationFailed {
            tab_id,
            url: primary_string,
            error_code: event.int_value,
            description: secondary_string,
        },
        EVENT_URL_CHANGED => BrowserEvent::UrlChanged {
            tab_id,
            url: primary_string,
        },
        EVENT_LOAD_FINISHED => BrowserEvent::LoadFinished { tab_id },
        EVENT_TAB_CRASHED => BrowserEvent::TabCrashed { tab_id },
        _ => BrowserEvent::ConsoleMessage {
            tab_id,
            level: "debug".to_string(),
            message: format!("unmapped ffi event type: {}", event.event_type),
            source_id: None,
            line_number: None,
        },
    };

    match api::engine_api::handle_event_with_sequence(browser_event, event.sequence_number) {
        Ok(()) => 1,
        Err(error) => {
            eprintln!("[FFI:C] handle_event failed: {error}");
            0
        }
    }
}

pub type FlutterEventCallback = extern "C" fn(FfiBrowserEvent);

static FLUTTER_EVENT_CALLBACK: once_cell::sync::Lazy<std::sync::Mutex<Option<FlutterEventCallback>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

#[unsafe(no_mangle)]
pub extern "C" fn netra_register_event_dispatcher(callback: FlutterEventCallback) -> i32 {
    if EVENT_DISPATCHER_REGISTERED.swap(true, Ordering::SeqCst) {
        eprintln!("[FFI] Event dispatcher already registered — skipping");
        return 1;
    }

    match FLUTTER_EVENT_CALLBACK.lock() {
        Ok(mut callback_guard) => {
            *callback_guard = Some(callback);
        }
        Err(error) => {
            eprintln!("[FFI] Callback mutex poisoned: {}", error);
            return 0;
        }
    }

    let handler = std::sync::Arc::new(|seq: u32, event: crate::core::entities::browser_event::BrowserEvent, current_tab: Option<crate::core::entities::tab::Tab>| {
        match FLUTTER_EVENT_CALLBACK.lock() {
            Ok(callback_guard) => {
                if let Some(cb) = *callback_guard {
                    if let Some(ffi_event) =
                        to_ffi_browser_event(seq, &event, current_tab.as_ref())
                    {
                        cb(ffi_event);
                    }
                }
            }
            Err(error) => {
                eprintln!("[FFI] Callback mutex poisoned: {}", error);
            }
        }
    });

    let dispatcher = crate::browser::event_dispatcher::EventDispatcher::new(handler);

    match crate::browser::browser_controller::BrowserController::subscribe_safe(Box::new(
        move |event| {
            if let crate::core::events::event_bus::Event::BrowserEvent(
                seq,
                browser_event,
                current_tab,
            ) = event
            {
                dispatcher
                    .handler
                    .as_ref()(*seq, browser_event.clone(), current_tab.clone());
            }
        },
    )) {
        Ok(()) => 1,
        Err(error) => {
            eprintln!("[FFI] Failed to register event dispatcher: {}", error);
            0
        }
    }
}

fn to_ffi_tab_state(tab: Option<&crate::core::entities::tab::Tab>) -> FfiTabState {
    fn into_raw_or_null(value: String) -> *mut c_char {
        CString::new(value)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    }

    match tab {
        Some(tab) => FfiTabState {
            tab_id: into_raw_or_null(tab.id.clone()),
            url: into_raw_or_null(tab.url.clone()),
            title: into_raw_or_null(tab.title.clone()),
            favicon_url: tab
                .favicon_url
                .clone()
                .and_then(|favicon_url| CString::new(favicon_url).ok())
                .map(CString::into_raw)
                .unwrap_or(std::ptr::null_mut()),
            is_active: u8::from(tab.is_active),
            is_loading: u8::from(tab.is_loading),
            can_go_back: u8::from(tab.can_go_back),
            can_go_forward: u8::from(tab.can_go_forward),
            is_suspended: u8::from(tab.is_suspended),
            blocked_count: tab.blocked_count,
            sequence_number: tab.sequence_number,
        },
        None => FfiTabState {
            tab_id: std::ptr::null_mut(),
            url: std::ptr::null_mut(),
            title: std::ptr::null_mut(),
            favicon_url: std::ptr::null_mut(),
            is_active: 0,
            is_loading: 0,
            can_go_back: 0,
            can_go_forward: 0,
            is_suspended: 0,
            blocked_count: 0,
            sequence_number: 0,
        },
    }
}

fn to_ffi_browser_event(
    sequence_number: u32,
    event: &crate::core::entities::browser_event::BrowserEvent,
    current_tab: Option<&crate::core::entities::tab::Tab>,
) -> Option<FfiBrowserEvent> {
    use crate::core::entities::browser_event::BrowserEvent;

    let (event_type, tab_id, favicon_url) = match event {
        BrowserEvent::NavigationStarted { tab_id, .. } => {
            (EVENT_NAVIGATION_STARTED, tab_id.as_str(), None)
        }
        BrowserEvent::FrameCreated { tab_id } => (EVENT_FRAME_CREATED, tab_id.as_str(), None),
        BrowserEvent::FrameDestroyed { tab_id } => {
            (EVENT_FRAME_DESTROYED, tab_id.as_str(), None)
        }
        BrowserEvent::LoadStarted { tab_id } => (EVENT_LOAD_STARTED, tab_id.as_str(), None),
        BrowserEvent::NavigationCompleted { tab_id, .. } => {
            (EVENT_NAVIGATION_COMPLETED, tab_id.as_str(), None)
        }
        BrowserEvent::TitleChanged { tab_id, .. } => (EVENT_TITLE_CHANGED, tab_id.as_str(), None),
        BrowserEvent::HistoryStateChanged { tab_id, .. } => {
            (EVENT_HISTORY_STATE_CHANGED, tab_id.as_str(), None)
        }
        BrowserEvent::RequestBlocked { tab_id, .. } => {
            (EVENT_REQUEST_BLOCKED, tab_id.as_str(), None)
        }
        BrowserEvent::NavigationFailed { tab_id, .. } => {
            (EVENT_NAVIGATION_FAILED, tab_id.as_str(), None)
        }
        BrowserEvent::UrlChanged { tab_id, .. } => (EVENT_URL_CHANGED, tab_id.as_str(), None),
        BrowserEvent::LoadFinished { tab_id } => (EVENT_LOAD_FINISHED, tab_id.as_str(), None),
        BrowserEvent::FaviconChanged { tab_id, favicon_url } => {
            (EVENT_FAVICON_CHANGED, tab_id.as_str(), Some(favicon_url.as_str()))
        }
        BrowserEvent::TabCrashed { tab_id } => (EVENT_TAB_CRASHED, tab_id.as_str(), None),
        BrowserEvent::ConsoleMessage { .. } => return None,
    };

    Some(FfiBrowserEvent {
        sequence_number,
        event_type,
        tab_id: CString::new(tab_id)
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut()),
        favicon_url: favicon_url
            .and_then(|url| CString::new(url).ok())
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut()),
        tab_state: to_ffi_tab_state(current_tab),
    })
}
