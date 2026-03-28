# Netra Browser

Netra Browser is a multi-layer browser architecture built with:

- Flutter for UI only
- Rust for core logic, state, navigation, and the event system
- C++ for the WebView2 bridge
- `flutter_rust_bridge` for the FFI layer

Core principle:

- Rust owns all state and logic
- UI is fully decoupled and event-driven

## Project Overview

Netra is organized as a browser platform rather than a single monolithic app
layer. Each layer has one clear job, and the system is designed so that the
Rust core remains the single source of truth while Flutter only renders and
reacts to events.

## Architecture

### 1. UI Layer (Flutter)

- Stateless relative to browser-domain state
- Renders tabs, navigation controls, and browser shell UI
- Receives state changes and events from the Rust-driven pipeline

### 2. Rust Core

- `TabManager`
  - Owns tab lifecycle
  - Owns active-tab tracking
  - Owns suspension and LRU background tab behavior
- `NavigationController`
  - Owns per-tab history
  - Owns URL normalization and navigation state flags
- `BrowserState`
  - Coordinates `TabManager` and `NavigationController`
  - Synchronizes navigation state back into tab state
- `BrowserController`
  - Single entry point for browser operations
  - Publishes integration events through `EventBus`
- `EventBus`
  - Internal synchronous publish/subscribe system
- `EventDispatcher`
  - Converts internal events into simple FFI-ready payloads

### 3. Bridge Layer (C++)

- Hosts WebView2 integration
- Translates between native browser surfaces and the higher-level app stack
- Contains no browser business logic

### 4. FFI Layer

- Uses `flutter_rust_bridge`
- Converts Rust calls and events into Dart-consumable forms

## Data Flow

User action -> Flutter UI  
-> FFI -> `BrowserController`  
-> `BrowserState`  
-> `TabManager` / `NavigationController`

Then:

`BrowserController` -> `EventBus`  
-> `EventDispatcher`  
-> Flutter UI updates

## Tab System

- Multi-tab support is implemented
- Only one tab can be active at a time
- Background tabs are managed with LRU rules
- Maximum of 5 background tabs stay active
- Opening a 6th background tab suspends the oldest background tab
- Idle background tabs are suspended after 5 minutes
- Suspended tabs reload when they are activated again

## Navigation System

- URL normalization is implemented
- Missing schemes are normalized with `https://`
- Inputs containing spaces are treated as search queries
- Search queries are converted to Google search URLs
- Each tab has its own history stack
- Supported navigation actions:
  - navigate
  - back
  - forward
- Forward history is cleared on new navigation

## Event System

- The browser core uses one unified event pipeline through `EventBus`
- Supported internal events include:
  - `TabCreated`
  - `TabClosed`
  - `TabSuspended`
  - `TabResumed`
  - `NavigationCompleted`
  - `BlockedRequest`
- Event flow:
  - `BrowserController` -> `EventBus` -> `EventDispatcher` -> Flutter
- No duplicate event systems exist in the Rust core

## State Management Rules

- Rust owns all state
- Flutter does not own browser-domain state
- State is not duplicated across modules
- `TabManager` is the single source of truth for tabs
- `NavigationController` is the single source of truth for history

## Current Status

Phase 2 is complete.

Implemented:

- Multi-tab system
- Navigation engine
- Global state coordination
- Event-driven architecture
- FFI-ready browser controller

## Next Steps (Phase 3)

- Flutter UI integration against the finalized Rust core
- Event stream handling in Dart
- WebView2 control wiring through the C++ bridge
- Loading-state and UI synchronization
- Advanced features such as bookmarks, downloads, and related browser services

## Design Principles

- Event-driven architecture
- Strict separation of concerns
- Single source of truth
- No cross-layer logic leakage
- Deterministic synchronous core

## Repository Map

- `lib/netra/`
  - Flutter UI shell, typed browser events, Dart bridge adapters
- `rust/`
  - Browser core, shared entities, event system, FFI-facing logic
- `windows/runner/bridge/`
  - Native C++ bridge and WebView2 integration
