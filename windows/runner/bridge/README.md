# Windows Native Bridge

This folder contains the Windows-specific bridge that hosts WebView2, exposes
the narrow Flutter MethodChannel surface, and connects native browser events to
the Rust runtime.

## What This Layer Owns

- WebView2 environment and controller lifecycle.
- Native child host windows used to embed one browser surface per tab.
- The Flutter MethodChannel used for native layout updates.
- Native-to-Rust event forwarding for browser lifecycle and navigation events.
- Request interception before the Rust core makes a block decision.

## Subfolders

- `webview2/`: WebView2 manager, MethodChannel handler, native event emitter,
  request interception, and placeholder expansion points.
- `ffi/`: Rust DLL resolution, native executor registration, and C++ to Rust
  event handoff.

## Startup Integration

The bridge is wired from `windows/runner/flutter_window.cpp`:

- registers the `netra/browser/methods` MethodChannel handler
- initializes the shared `WebViewManager`
- registers native executor callbacks with Rust through
  `RegisterRustNativeExecutor()`

Once that registration is complete, Rust browser-controller operations can call
directly into the Windows-native WebView2 executor without routing browser
actions back through Flutter.

## Current State

- Tab-scoped WebView2 controllers are created and destroyed natively.
- Each tab gets its own dedicated host window.
- Native browser events are forwarded into Rust, not directly into Flutter.
- Request interception is active and delegates allow or block decisions through
  the FFI bridge.
- Flutter currently uses this layer mainly for native bounds synchronization.

## Current Limitations

- The Flutter MethodChannel surface is intentionally narrow and currently
  supports only `setBounds`.
- Request blocking is still stubbed in the C++ FFI layer, so intercepted
  requests are not yet blocked by Rust policy.
- Script injection is registered through a placeholder stub in the WebView2
  manager.
