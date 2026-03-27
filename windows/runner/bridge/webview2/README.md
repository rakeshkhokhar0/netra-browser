# WebView2 Native Bridge

This folder contains the Windows WebView2 integration used by the Flutter
desktop shell.

## File Responsibilities

- `webview_manager.h/.cpp`:
  - Initializes shared WebView2 environment.
  - Creates and destroys tab-scoped controllers.
  - Manages host windows, bounds updates, and active-tab visibility.
  - Registers browser event callbacks and request interception.
- `method_handler.cpp`:
  - Handles Flutter MethodChannel commands and maps arguments to native calls.
  - Supports tab lifecycle, navigation, reload, stop, bounds, active-tab.
- `event_emitter.cpp`:
  - Registers Flutter EventChannel and broadcasts structured browser events.
- `request_filter.cpp`:
  - Hooks `WebResourceRequested` and delegates block decisions to Rust bridge.
- `webview2_bridge_stub.cpp`:
  - Placeholder file reserved for future native bridge expansion.

## Work Done

- Method and event channels are wired into Windows runner startup.
- Browser command handling is implemented for core tab/navigation operations.
- Event translation to Flutter map payloads is implemented.
- Request filtering hook is integrated and connected to FFI bridge contract.
- UI-thread checks and controller lifecycle handling are in place.

## Current Limitations

- `clearData` is not supported yet in method handler.
- Script injection currently uses a stub script placeholder.
- Rust block decision is currently a stub return path in FFI implementation.
