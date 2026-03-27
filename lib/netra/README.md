# Netra Dart Layer

This folder contains the Flutter/Dart side of Netra Browser. It defines the
UI shell, engine abstractions, bridge transport adapters, and shared models.

## Current Status

- Windows browser shell path is wired from UI -> provider -> engine adapter ->
  MethodChannel/EventChannel.
- Typed browser event mapping is implemented in Dart.
- Rust bridge bootstrap path is initialized at app startup.
- Several secondary UI screens/components are still placeholders.

## Subfolder Ownership

- `core/`
  - Shared entities and event contracts used across engine and UI layers.
  - Examples: tab state, history entry, bookmarks, browser event base types.
- `di/`
  - Dependency injection wiring through Riverpod providers/modules.
  - `engine_module.dart` is the single adapter construction entry.
- `engine/`
  - Platform-agnostic interfaces plus concrete WebView2 adapter.
  - Owns typed browser event translation and frame-state cache.
- `ffi/`
  - Dart FFI bootstrap for Rust dynamic library loading and smoke test calls.
- `infrastructure/`
  - Platform channel transport adapters (`MethodChannel` and `EventChannel`).
  - No business logic; command/event plumbing only.
- `shared/`
  - App-wide config constants, shared errors, and utility helpers.
- `ui/`
  - Browser shell widgets, layout, providers, and user interaction handling.
  - Main shell is active; some feature screens/components are placeholders.

## Work Done In This Layer

- App initializes Rust bridge before rendering UI.
- Active tab lifecycle and state sync implemented via Riverpod.
- Toolbar and address bar commands are connected to native methods.
- Native event stream is converted to typed `BrowserEvent` classes.
- Browser layout reports bounds to native side and sets active tab.

## Known Gaps

- `executeScript` is not exposed through the Dart method-channel path yet.
- Some screens/components (bookmarks/history/downloads/settings) are scaffolded
  placeholders and need full UI + feature wiring.
