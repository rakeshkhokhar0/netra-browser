# WebView2 Native Bridge

This folder contains the Windows WebView2 hosting code used by the Netra
desktop runner.

## File Responsibilities

- `webview_manager.h/.cpp`
  - initializes the shared WebView2 environment
  - creates one controller and one native host window per tab
  - tracks active-tab visibility and bounds updates
  - registers browser event callbacks, request interception, and document
    script injection
- `method_handler.cpp`
  - owns the Flutter MethodChannel entry point
  - currently validates arguments and handles only the `setBounds` command
- `event_emitter.cpp`
  - converts WebView2 callbacks into strongly typed native events for Rust
- `request_filter.cpp`
  - hooks `WebResourceRequested` and calls `ShouldBlockRequest`
- `webview2_bridge_stub.cpp`
  - reserved placeholder for future Windows bridge expansion

## Runtime Notes

- The shared WebView2 environment is initialized once and reused across tabs.
- Browser arguments enable DNS-over-HTTPS for created controllers.
- Host windows are created as child windows inside the main Flutter runner
  window.
- Native browser events such as navigation, title, favicon, history, URL
  changes, and crashes are forwarded into Rust.

## Current Limitations

- The MethodChannel surface in this folder is intentionally small and currently
  supports only `setBounds`.
- Document-created script injection still uses a placeholder stub.
- Request interception is wired, but blocking is not yet enforced because the
  FFI bridge still returns `false` for block decisions.
