# Netra Rust Core

The `rust/` directory contains the browser runtime that owns state, navigation,
and the authoritative event pipeline for Netra Browser.

## Folder Layout

- `browser/`: runtime controller, browser state coordinator, tab manager,
  navigation controller, session manager, and event dispatcher.
- `core/`: shared entities, value objects, errors, and the synchronous
  `EventBus`.
- `ffi/`: exported C ABI for Dart and native callers, plus API modules used by
  the bridge entry points.
- `network/`: request and response interception pipeline components.
- `privacy/`: adblock, DNS, fingerprint, and WebRTC protection modules.
- `services/`: bookmarks, downloads, filter lists, history, and settings
  managers.
- `storage/`: SQLite, encryption, and migration helpers.
- `platform/`: platform-specific Rust modules.
- `utils/`: logging, config, and shared utility code.

## Active Runtime Path

The current browser runtime is centered around these pieces:

- `browser/browser_controller.rs`: synchronous orchestration layer used by FFI
  entry points.
- `browser/browser_state.rs`: coordinates tab and navigation state.
- `browser/tab_manager.rs`: authoritative tab lifecycle and sequencing.
- `browser/navigation_controller.rs`: history and URL normalization.
- `browser/event_dispatcher.rs`: forwards browser events toward the FFI
  boundary.
- `ffi/mod.rs`: exported C functions for smoke tests, browser commands, state
  snapshots, native event intake, and Flutter callback registration.
- `ffi/native_control.rs`: registered callback surface used to invoke the
  Windows-native WebView2 executor from Rust.

## Ownership Model

- Rust owns browser-domain state.
- Rust applies authoritative native browser events to that state.
- Rust publishes typed browser events back to Flutter.
- Flutter reads snapshots and renders them, but does not own tab or history
  state.

## Browser Capabilities Wired Today

- Tab creation, closing, activation, and sequencing.
- URL normalization and per-tab history navigation.
- Browser-state snapshot export as JSON for Dart consumers.
- C ABI functions for create tab, close tab, set active tab, load URL, back,
  forward, reload, and stop loading.
- Intake of native WebView2 events such as navigation, title, favicon, history,
  URL changes, blocked requests, and crash notifications.

## Notes On Broader Modules

The `network/`, `privacy/`, `services/`, and `storage/` folders provide the
foundation for a wider browser feature set, but not every module in those
folders is fully wired into the active Windows runtime yet. The core browser
path described above is the part currently driving the app.

## Current Limitations

- Native request blocking is not finalized end to end because the Windows-side
  `ShouldBlockRequest` bridge still returns `false`.
- Some platform folders exist for future expansion, but the current native
  browser executor path is wired on Windows.
