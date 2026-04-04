# Netra Flutter Layer

The `lib/netra/` directory contains the Flutter-side browser shell and the
Dart bridge code that talks to Rust and the Windows runner.

## What Lives Here

- `ui/`: app widget tree, shell screens, layouts, visual components, and
  Riverpod providers.
- `engine/`: typed engine contracts plus the WebView2 adapter that exposes the
  Rust-driven browser event stream.
- `ffi/`: Dart FFI bootstrap and the direct Rust bridge used for browser
  commands, state snapshots, and native event registration.
- `infrastructure/`: platform transport helpers such as the MethodChannel
  bridge used for `setBounds`.
- `di/`: dependency registration for the shell.
- `core/`: shared Dart entities for browser, bookmark, download, history, and
  filter-list data.
- `shared/`: app configuration, shared errors, and file utilities.

## Current Responsibility Split

- Flutter renders the browser shell and user-facing screens.
- `BrowserProvider` treats Rust snapshots as authoritative and refreshes UI
  state from `netra_get_browser_state_json`.
- `TabProvider` owns only tab lifecycle commands and does not store browser
  state.
- `WebView2Adapter` is event-only on the Dart side. Direct tab and navigation
  control no longer belong to the adapter layer.

## Command and Event Paths

- Tab and navigation commands go from Dart to Rust through `ffi/bridge.dart`.
- Native bounds updates go from Dart to the Windows runner through
  `infrastructure/bridge/method_channel_bridge.dart`.
- Browser events come back from Rust as typed `BrowserEvent` instances and are
  consumed by the providers in `ui/providers/`.

## Important Rules

- Do not introduce a second source of truth for tabs, navigation, or loading
  state in Dart.
- Prefer reflecting Rust-owned state over reconstructing browser state locally.
- Keep infrastructure and engine adapters transport-focused and free of browser
  business logic.

## Current State

This folder is now actively wired into the runtime rather than being only a
placeholder shell:

- `BridgeInitializer` starts the Rust bridge during app boot.
- `BrowserShell` creates the initial tab after the first frame when needed.
- Riverpod providers react to Rust-owned browser events and snapshots.
- The UI can drive tab lifecycle, navigation, and native bounds updates.

Known limitation:

- The Flutter-to-native MethodChannel currently supports only the `setBounds`
  layout command. Browser actions themselves are routed through Rust.
