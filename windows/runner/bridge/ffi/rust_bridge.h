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

/// Asks the Rust core whether a request should be blocked.
///
/// The `url` and `resource_type` values are passed through to the Rust layer,
/// which owns the actual filtering rules and decision-making.
bool ShouldBlockRequest(const std::string& url,
                        const std::string& resource_type);

}  // namespace netra::bridge::ffi

#endif  // NETRA_BROWSER_WINDOWS_RUNNER_BRIDGE_FFI_RUST_BRIDGE_H_
