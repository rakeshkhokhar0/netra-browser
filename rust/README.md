# Netra Rust Core

The `rust/` directory contains the synchronous browser core for Netra Browser.

## Role In The System

- Owns browser logic and browser state
- Owns tab lifecycle, navigation, and event flow
- Exposes FFI-ready controller surfaces
- Stays fully decoupled from UI concerns

## Core Modules

- `browser/`
  - `TabManager`
    - single source of truth for tab state
    - owns lifecycle, active-tab rules, suspension, and LRU behavior
  - `NavigationController`
    - single source of truth for per-tab history
    - owns URL normalization and navigation stack behavior
  - `BrowserState`
    - coordinates tab state and navigation state
    - syncs navigation results back into tab state
  - `BrowserController`
    - single entry point for browser operations
    - publishes integration events
  - `EventDispatcher`
    - forwards internal events toward the FFI boundary
- `core/`
  - shared entities, errors, value objects, and the unified `EventBus`
- `ffi/`
  - Rust APIs exposed through `flutter_rust_bridge`
- `network/`
  - request/response interception pipeline surfaces
- `privacy/`
  - privacy modules such as ad blocking, DNS, fingerprint, and WebRTC
- `services/`
  - bookmarks, downloads, filter lists, history, and settings services
- `storage/`
  - persistence, migrations, and storage helpers

## Final Architecture After Phase 2

- Rust owns all logic and all browser-domain state
- The core is deterministic and synchronous
- One unified event system exists through `EventBus`
- One unified `Tab` type exists through `crate::core::entities::tab::Tab`
- No duplicate state systems are used for tabs or history

## Tab System

- Multi-tab support is implemented
- Only one tab is active at a time
- Background tabs are managed using LRU rules
- At most 5 background tabs remain active
- Oldest background tabs are suspended when limits are exceeded
- Idle background tabs are suspended after 5 minutes
- Suspended tabs are resumed on activation

## Navigation System

- URL normalization is handled in Rust
- Inputs without a scheme are normalized to `https://`
- Inputs with spaces are treated as search queries
- Search queries are converted to Google search URLs
- History is stored per tab
- Back and forward navigation are supported
- Forward history is cleared on new navigation

## Event System

- `EventBus` is the single internal event pipeline
- Supported events include:
  - `TabCreated`
  - `TabClosed`
  - `TabSuspended`
  - `TabResumed`
  - `NavigationCompleted`
  - `BlockedRequest`
- `EventDispatcher` converts internal events into simple FFI payloads

## State Rules

- `TabManager` is the single source of truth for tabs
- `NavigationController` is the single source of truth for history
- `BrowserState` coordinates, but does not duplicate ownership
- Flutter does not own Rust browser-domain state

## Current Status

Phase 2 is complete.

Implemented:

- Multi-tab system
- Navigation engine
- Global state coordination
- Unified event architecture
- FFI-ready global browser controller access

## Phase 3 Focus

- Wire Flutter UI to the stabilized Rust core
- Handle event streams in Dart
- Connect WebView2 control flow through the native bridge
- Add UI sync for loading and navigation state
- Continue advanced browser features
