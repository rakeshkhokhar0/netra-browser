# Netra Browser

Netra Browser is a Windows-first desktop browser shell built from three
cooperating layers:

- Flutter renders the application UI.
- Rust owns browser state, navigation orchestration, and event dispatch.
- C++ hosts WebView2 and bridges native browser surfaces to the Rust core.

The core rule in this repository is simple: Rust is the source of truth for
browser-domain state, while Flutter reacts to snapshots and typed events.

## Current Architecture

### Flutter shell

- App startup begins in `lib/main.dart` and initializes the Rust bridge before
  `runApp`.
- Riverpod providers in `lib/netra/ui/providers/` expose browser state to the
  UI.
- Browser commands such as create tab, close tab, activate tab, navigate,
  back, forward, reload, and stop loading go through the Dart FFI bridge.
- Flutter still owns layout updates for embedded native surfaces and sends
  `setBounds` over a MethodChannel to the Windows runner.

### Rust core

- `browser/` contains the runtime controller, state coordinator, navigation
  controller, tab manager, and event dispatcher.
- `core/` defines shared entities, value objects, errors, and the synchronous
  `EventBus`.
- `ffi/` exports the C ABI consumed by Dart and the Windows runner, including
  command entry points, browser-state snapshots, and event registration.
- `native_control.rs` lets Rust invoke Windows-native tab and navigation
  actions after the native executor is registered.

### Windows native bridge

- `windows/runner/flutter_window.cpp` registers the MethodChannel handler,
  initializes the shared WebView2 environment, and registers native executor
  callbacks with Rust at startup.
- `windows/runner/bridge/webview2/` manages tab-scoped WebView2 controllers,
  host windows, event hooks, request interception, and bounds updates.
- `windows/runner/bridge/ffi/` resolves the Rust DLL, forwards native browser
  events into Rust, and exposes the callback surface used by Rust to control
  WebView2 tabs.

## Runtime Flow

1. Flutter initializes the Rust bridge.
2. The Windows runner initializes WebView2 and registers native executor
   callbacks with Rust.
3. UI actions call Rust FFI entry points.
4. Rust mutates browser state through `BrowserController`.
5. Rust forwards tab and navigation commands to the native executor.
6. WebView2 emits browser events in C++, which are sent back into Rust.
7. Rust publishes authoritative browser events back to Flutter, and the UI
   refreshes from Rust-owned snapshots.

## Repository Map

- `lib/netra/`: Flutter shell, providers, engine adapters, and bridge glue.
- `rust/`: Rust browser runtime, FFI exports, shared entities, and services.
- `windows/runner/bridge/`: WebView2 hosting and C++ to Rust integration.
- `docs/`: supporting notes and project documentation.
- `android/`, `ios/`, `linux/`, `macos/`, `web/`: standard Flutter platform
  scaffolding. The browser-engine integration is currently wired on Windows.

## Current Status

Implemented today:

- Rust-owned multi-tab and navigation state.
- Dart FFI bridge for browser commands and state snapshots.
- Rust-to-Flutter event dispatch for typed browser updates.
- Windows WebView2 controller lifecycle and tab host windows.
- Native request interception and browser event forwarding into Rust.

Known limitations:

- The Windows MethodChannel currently exposes only `setBounds`.
- Native request blocking is still stubbed in
  `windows/runner/bridge/ffi/rust_bridge.cpp`.
- Document-created script injection is currently a placeholder in
  `windows/runner/bridge/webview2/webview_manager.h`.

## Fresh Clone Setup

This project is currently set up for Windows desktop development first. After
pulling the repo, use the steps below to get a working local environment.

### Prerequisites

- Flutter SDK with Windows desktop enabled.
- Rust toolchain with `cargo` available on `PATH`.
- Visual Studio Build Tools or Visual Studio with Desktop development for C++.
- CMake and Git.
- Microsoft Edge WebView2 Runtime installed on Windows.
- Microsoft Edge WebView2 SDK extracted to `C:/webview2-sdk`.

Important:

- `windows/runner/CMakeLists.txt` currently expects the WebView2 SDK at
  `C:/webview2-sdk`.
- That folder must contain:
  - `include/WebView2.h`
  - `lib/x64/WebView2LoaderStatic.lib`
- If your SDK is somewhere else, update `WEBVIEW2_SDK_PATH` in
  `windows/runner/CMakeLists.txt`.

### First-Time Setup

1. Clone the repository and open it in a Windows terminal.
2. Install Flutter packages:

```powershell
flutter pub get
```

3. Confirm your local toolchains are ready:

```powershell
flutter doctor -v
cargo --version
```

4. Optional but recommended: verify the Rust core compiles and tests cleanly:

```powershell
cargo test --manifest-path rust/Cargo.toml
```

5. Run the desktop app:

```powershell
flutter run -d windows
```

### Build Notes

- You do not need to build the Rust DLL manually for the normal Windows app
  flow.
- The Windows runner CMake build triggers `cargo build` automatically through
  the `rust_bridge_build` target.
- After the Rust build completes, `netra_rust.dll` is copied into the Windows
  runner output folder automatically.
- Flutter-generated folders such as `build/` and `windows/flutter/ephemeral/`
  are recreated locally during setup and build.

### Common Issues

- `WebView2.h not found. Check SDK path.`
  This means the SDK is missing from `C:/webview2-sdk` or the CMake path needs
  to be updated.
- `cargo` not found
  Install Rust and reopen the terminal so `cargo` is on `PATH`.
- Flutter Windows toolchain errors
  Run `flutter doctor -v` and complete any missing Visual Studio or desktop
  toolchain setup.
- App builds but native browser surfaces do not initialize
  Check that the WebView2 Runtime is installed and that `netra_rust.dll` was
  copied into the build output.

## Folder READMEs

For more detail, see the folder-specific documentation:

- `lib/netra/README.md`
- `rust/README.md`
- `windows/runner/bridge/README.md`
