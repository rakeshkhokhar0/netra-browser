# Windows Rust FFI Bridge

This folder contains the narrow C++ boundary used to communicate with the Rust
DLL from the Windows runner.

## Files

- `rust_bridge.h`: public native bridge contract shared by the Windows runner.
- `rust_bridge.cpp`: Rust DLL loading, symbol resolution, executor
  registration, native event forwarding, and request-filter bridge functions.

## What This Folder Owns

- Resolving `netra_rust.dll` and caching required function pointers.
- Registering native executor callbacks so Rust can create tabs, load URLs, and
  control navigation directly in WebView2.
- Forwarding structured native browser events into `netra_handle_native_event`.
- Exposing `ShouldBlockRequest` to the request interception layer.

## Active Runtime Role

This bridge is more than a request-filter hook now:

- `RegisterRustNativeExecutor()` binds the Windows WebView2 control surface to
  Rust's `native_control` layer.
- `SendNativeEventToRust()` forwards C++ browser events into the Rust core.
- `ShouldBlockRequest()` preserves the filtering contract used by
  `request_filter.cpp`.

## Current Limitation

`ShouldBlockRequest()` is still a stub and always returns `false`, so the
native interception path is wired but not yet enforcing Rust-owned filtering
policy.
