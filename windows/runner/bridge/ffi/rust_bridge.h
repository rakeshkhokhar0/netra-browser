#ifndef NETRA_BROWSER_WINDOWS_RUNNER_BRIDGE_FFI_RUST_BRIDGE_H_
#define NETRA_BROWSER_WINDOWS_RUNNER_BRIDGE_FFI_RUST_BRIDGE_H_

#include <string>

/// Defines the narrow native surface used by the Windows bridge to delegate
/// request-blocking decisions to Rust.
///
/// The implementation in `rust_bridge.cpp` is intentionally thin and does not
/// contain filtering logic. It only translates native values into the Rust FFI
/// boundary and returns Rust-owned decisions back to WebView2 callers.
namespace netra::bridge::ffi {

/// Shared native event type identifiers used across C++ and Rust FFI.
inline constexpr int32_t kEventNavigationStarted = 1;
inline constexpr int32_t kEventFrameCreated = 2;
inline constexpr int32_t kEventFrameDestroyed = 3;
inline constexpr int32_t kEventLoadStarted = 4;
inline constexpr int32_t kEventNavigationCompleted = 5;
inline constexpr int32_t kEventTitleChanged = 6;
inline constexpr int32_t kEventHistoryStateChanged = 7;
inline constexpr int32_t kEventRequestBlocked = 8;
inline constexpr int32_t kEventNavigationFailed = 9;
inline constexpr int32_t kEventUrlChanged = 10;
inline constexpr int32_t kEventLoadFinished = 11;
inline constexpr int32_t kEventFaviconChanged = 12;
inline constexpr int32_t kEventTabCrashed = 13;

/// Asks the Rust core whether a request should be blocked.
///
/// The `url` and `resource_type` values are passed through to the Rust layer,
/// which owns the actual filtering rules and decision-making.
bool ShouldBlockRequest(const std::string& url,
                        const std::string& resource_type);

/// Describes a strongly typed native browser event used for C++ to Rust FFI.
struct FfiNativeBrowserEvent {
    uint32_t sequence_number;
    int32_t event_type;
    char* tab_id;
    char* primary_string;
    char* secondary_string;
    int32_t int_value;
    uint8_t bool_value;
};

/// Sends a strongly-typed native browser event to the Rust orchestrator.
void SendNativeEventToRust(const FfiNativeBrowserEvent& event);

/// Registers the native WebView2 executor callbacks with the Rust DLL.
///
/// After registration completes, Rust browser-controller operations can call
/// directly into the native Windows bridge without routing through Flutter
/// MethodChannel control paths.
void RegisterRustNativeExecutor();

}  // namespace netra::bridge::ffi

#endif  // NETRA_BROWSER_WINDOWS_RUNNER_BRIDGE_FFI_RUST_BRIDGE_H_
