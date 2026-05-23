# Netra Browser

> A privacy-first browser built with Flutter, Rust, and native WebViews.

**Status:** Netra is currently under active development.

---

## Overview

Netra is a cross-platform browser project built using:

- Flutter for UI
- Rust for the browser engine
- Native WebViews for rendering
- C++ / Kotlin / Swift bridges for platform integration

Instead of embedding a full browser engine like Chromium for all browser logic, Netra uses native platform rendering engines while keeping browser management, downloads, networking, state handling, and browser systems inside a Rust-based engine.

---

# Tech Stack

| Layer | Technology |
|---|---|
| UI | Flutter |
| Core Engine | Rust |
| State Management | Riverpod |
| FFI Bridge | flutter_rust_bridge |
| Windows Rendering | WebView2 |
| Windows Native Layer | C++ |
| Ad Blocking | adblock-rust |
| DNS | DNS-over-HTTPS |

---

# Current Platform Support

| Platform | Status |
|---|---|
| Windows | 🚧 In Development |
| Android | ⏳ Planned |
| iOS | ⏳ Planned |
| macOS | ⏳ Planned |
| Linux | ⏳ Planned |

---

# Project Structure

```text
netra-browser/
│
├── android/              # Android platform runner
├── ios/                  # iOS platform runner
├── linux/                # Linux platform runner
├── macos/                # macOS platform runner
│
├── lib/                  # Flutter application
│   └── netra/
│
├── rust/                 # Rust browser engine
│
├── windows/
│   └── runner/
│       └── bridge/       # C++ WebView2 bridge
│
├── test/                 # Flutter tests
├── web/                  # Web assets
│
└── pubspec.yaml
```

---

# Windows Architecture

```text
Flutter UI
    ↓
Dart Interfaces
    ↓
flutter_rust_bridge
    ↓
Rust Core
    ↓
C++ Bridge
    ↓
Microsoft Edge WebView2
```

The Rust layer handles:
- Browser state
- Tabs
- Navigation
- Downloads
- Ad blocking
- Session management

The C++ layer only manages native WebView2 controls.

---

# Requirements

## 1. Flutter SDK

Install Flutter Stable SDK:

https://docs.flutter.dev/get-started/install/windows

Verify installation:

```bash
flutter doctor
```

---

## 2. Rust Toolchain

Install Rust:

https://rustup.rs/

Verify installation:

```bash
rustc --version
cargo --version
```

---

## 3. Visual Studio 2022

Install:

https://visualstudio.microsoft.com/downloads/

Required workload:
- Desktop development with C++

Required components:
- MSVC v143 build tools
- Windows SDK
- CMake tools

---

## 4. Microsoft Edge WebView2 Runtime

Netra uses WebView2 for webpage rendering.

Most Windows 10/11 systems already include it.

If not installed:

https://developer.microsoft.com/en-us/microsoft-edge/webview2/

---

## 5. WebView2 SDK (Required)

Download the WebView2 SDK from Microsoft.

After extracting, place it at:

```text
C:/webview2-sdk
```

Required structure:

```text
C:/webview2-sdk/
├── include/
│   └── WebView2.h
├── lib/
└── build/
```

The Windows build system expects:

```text
C:/webview2-sdk/include/WebView2.h
```

If the SDK is not placed correctly, the Windows build will fail.

---

## 6. Git

Install Git:

https://git-scm.com/downloads

Verify:

```bash
git --version
```

---

# Setup Instructions

## Clone Repository

```bash
git clone <repository-url>
```

Move into project:

```bash
cd netra-browser
```

---

## Install Flutter Dependencies

```bash
flutter pub get
```

---

## Fetch Rust Dependencies

```bash
cd rust
cargo fetch
cd ..
```

---

# Running Netra

## Run on Windows

```bash
flutter run -d windows
```

The Windows build system automatically:
- Builds the Rust engine
- Links Rust DLLs
- Builds the Flutter Windows runner
- Links WebView2
- Launches the application

---

# Manual Rust Build (Optional)

```bash
cd rust
cargo build --release
```

Generated output:

```text
target/release/netra_rust.dll
```

---

# Build Release Version

```bash
flutter build windows --release
```

---

# Common Issues

## flutter doctor Errors

Run:

```bash
flutter doctor
```

Resolve all missing dependencies before building.

---

## WebView2.h Not Found

Error:

```text
fatal error: WebView2.h: No such file or directory
```

Fix:
Ensure WebView2 SDK exists at:

```text
C:/webview2-sdk
```

---

## Rust DLL Not Generated

Run manually:

```bash
cd rust
cargo build
```

---

## MSVC Compiler Missing

Open Visual Studio Installer and install:
- Desktop development with C++
- MSVC v143 build tools

---

## CMake Errors

Ensure:
- Windows SDK installed
- CMake tools installed
- Visual Studio C++ workload installed

---

# Recommended VS Code Extensions

- Flutter
- Dart
- Rust Analyzer
- Better TOML
- CMake Tools
- Error Lens

---

# Development Status

| Component | Status |
|---|---|
| Flutter UI | 🚧 Active |
| Rust Engine | 🚧 Active |
| WebView2 Bridge | 🚧 Active |
| Networking Layer | 🚧 In Progress |
| Ad Blocking | 🚧 In Progress |

---

# Planned Features

- Multi-tab management
- Download manager
- Ad blocking
- DNS-over-HTTPS
- Session restore
- Memory optimization
- Native sync system
- AI-powered browser tools

---

# Contributing

Netra is currently not open for external code contributions while the architecture is being stabilized.

Bug reports, suggestions, and feedback are welcome.

---

# License

MIT License

---

# Author

Built by **Rakesh Khokhar**

LinkedIn:
https://www.linkedin.com/in/rakesh-khokhar-53b221349
