# Windows Native Bridge

This folder contains the Windows-side native bridge between Flutter and the
embedded WebView2 browser engine, plus the FFI boundary toward Rust.

## Folder Purpose

- Expose browser commands from Flutter to native WebView2.
- Emit browser lifecycle/navigation events back to Flutter.
- Intercept network requests and delegate filtering decisions to Rust.
- Keep browser-hosting concerns out of Dart UI code.

## Subfolders

- `webview2/`: WebView2 environment/controller lifecycle, channel handlers,
  event emission, request interception.
- `ffi/`: thin C++ boundary that forwards native request metadata to Rust.

## Work Done

- WebView2 manager added with tab-scoped controller creation/destruction.
- Dedicated host windows per tab and active-tab visibility switching.
- MethodChannel handler wired for create/close/navigate/history/reload/bounds.
- EventChannel emitter wired for navigation/title/history/crash/engine events.
- Request filter registration hooked into WebView2 resource interception.
- C++ to Rust `ShouldBlockRequest` call boundary created.

## Current Limitation

- Request-blocking decision in the Rust bridge implementation is currently a
  stub path and returns `false` until the Rust pipeline is fully connected.
