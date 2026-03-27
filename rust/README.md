# Netra Rust Core

This folder contains the Rust side of Netra Browser: FFI entry points, core
domain modules, privacy/network pipeline scaffolding, and service/storage
surfaces.

## Current Status

- Rust library builds as `cdylib`/`staticlib` and is loaded by the Windows app.
- FFI smoke-test exports are implemented and callable from Dart.
- Core browser controller and tab manager scaffolding are in place.
- Many privacy/service/storage modules are currently documented placeholders.

## Subfolder Ownership

- `ffi/`
  - Public Rust FFI boundary and API entry points consumed by Flutter/native.
  - Includes smoke-test exports and browser API delegation.
- `core/`
  - Domain entities, typed errors, events, and value objects.
- `browser/`
  - Browser controller/tab/session/navigation orchestration primitives.
  - Current implementation keeps minimal in-memory tab state and validation.
- `network/`
  - Request/response interception and request-pipeline surfaces.
  - Structure exists; implementation is staged.
- `privacy/`
  - Adblock, DNS, fingerprint, and WebRTC protection modules.
  - Mostly scaffolding at this stage.
- `services/`
  - Feature service boundaries (bookmarks, history, downloads, settings,
    filter lists).
  - Mostly scaffolding at this stage.
- `storage/`
  - SQLite, migrations, and encryption surfaces for persisted state.
  - Structural groundwork present; implementation is staged.
- `platform/`
  - Platform-specific bridge points (windows/android/ios modules).
- `utils/`
  - Shared configuration, error helpers, and logger surfaces.
- `target/`
  - Cargo build output directory (generated artifacts, not source ownership).

## Work Done In This Layer

- `Cargo.toml` configured for shared library output used by Windows runner.
- FFI exports:
  - `netra_connection_smoke_test`
  - `netra_connection_message`
- Browser API entry points validate inputs and delegate into browser controller.
- Typed `NetraError` model implemented and used across controller paths.
- Tab manager includes active/background/suspended state logic with tests.

## Known Gaps

- Filtering/privacy/service/storage modules need full behavior implementation.
- Rust request decision is not yet fully connected to enforce blocking in
  production flow.
