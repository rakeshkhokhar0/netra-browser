# Netra Browser

Netra Browser is a Windows-first browser shell built with Flutter, a native
WebView2 bridge in C++, and a Rust core that is being integrated incrementally.

## Current Status

This repository is in active development. The Windows browser pipeline is
partially wired end-to-end:

- Flutter UI shell is running with Riverpod state management.
- Native MethodChannel commands are connected for tab and navigation actions.
- Native EventChannel events are emitted and mapped to typed Dart events.
- WebView2 controllers are created and managed per tab on the Windows side.
- Request interception hook is in place and calls a Rust bridge function.
- Rust library builds and loads, with smoke-test FFI exported functions working.

## What Is Implemented

- App startup initializes Flutter and the Rust bridge bootstrap path.
- Browser toolbar and address bar can trigger:
  - `createTab`, `closeTab`
  - `navigate`, `goBack`, `goForward`, `reload`, `stopLoading`
  - `setBounds`, `setActiveTab`
- Native events currently emitted:
  - `engineReady`
  - `navigationStarting`, `contentLoading`, `urlChanged`
  - `navigationCompleted`, `titleChanged`, `historyChanged`
  - `tabCrashed`
- Windows native layer manages dedicated host windows for WebView2 tabs.
- Rust side includes base controller + tab manager scaffolding and typed error
  handling.

## Folder Overview

- `lib/netra/core`: shared entities and event contracts used across layers.
- `lib/netra/engine`: engine interfaces, typed browser events, and WebView2 adapter.
- `lib/netra/infrastructure`: MethodChannel/EventChannel transport adapters.
- `lib/netra/ui`: shell UI, providers, widgets, and browser layout syncing.
- `lib/netra/ffi`: Dart FFI bootstrap and Rust smoke-test bridge bindings.
- `lib/netra/di`: dependency wiring modules and container entry points.
- `lib/netra/shared`: app config, shared errors, and utility helpers.
- `windows/runner/bridge`: native Windows bridge split into WebView2 + Rust FFI.
- `rust`: Rust core modules for browser, privacy, services, storage, and FFI APIs.

## Windows Build Notes

Prerequisites:

- Flutter SDK
- Rust toolchain (`cargo`)
- Visual Studio C++ build tools
- WebView2 SDK installed at `C:/webview2-sdk` (current CMake path)

Run:

```bash
flutter pub get
flutter run -d windows
```

## Known Gaps

- Rust request-blocking decision in C++ bridge is currently a stub return path.
- `executeScript` is not exposed through the Dart MethodChannel path yet.
- `clearData` method reports `not_supported` until profile integration is added.
- Multiple Rust modules are present as scaffolding/placeholders and will be
  implemented progressively.
