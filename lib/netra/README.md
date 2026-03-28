# Netra Flutter Layer

The `lib/netra/` directory contains the Flutter-side browser shell.

## Role In The System

- Renders UI only
- Sends user actions into the Rust-driven browser pipeline
- Receives browser events and typed state updates
- Avoids owning browser-domain logic and persistent browser state

## Layer Breakdown

- `ui/`
  - tab strip, toolbar, address bar, layout, and screens
- `engine/`
  - typed browser events and engine-facing Dart contracts
- `ffi/`
  - Rust bridge bootstrap and Dart FFI entry points
- `infrastructure/`
  - Dart bridge adapters and transport helpers
- `di/`
  - dependency wiring
- `core/`
  - shared Dart-side models and event contracts used by the shell
- `shared/`
  - configuration, errors, and utilities

## Architecture Rules

- Flutter is stateless relative to browser-domain ownership
- Rust owns all browser logic and state
- Flutter reacts to events and renders current state
- No duplicate browser state should be introduced in Dart

## Current Status

Phase 2 is complete on the Rust side.

The Flutter layer is prepared for Phase 3 integration:

- browser shell structure exists
- typed event models exist
- bridge bootstrap is in place
- UI integration against the finalized Rust event flow is the next step

## Phase 3 Focus

- connect Dart event handling to the unified Rust event pipeline
- bind UI state to Rust-driven tab and navigation updates
- complete WebView2/native bridge integration with the shell
