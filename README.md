# Netra 
> A privacy-first browser, built from scratch.

> 🚧 **Work in Progress:** Netra is currently under active development and is not fully built yet. The core architecture is actively being stabilized and bootstrapped.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Platform](https://img.shields.io/badge/platform-Windows-blue)
![Flutter](https://img.shields.io/badge/Made_with-Flutter-46b0fa?logo=flutter)
![Rust](https://img.shields.io/badge/Made_with-Rust-black?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![Built with AI](https://img.shields.io/badge/Built_with-AI-purple)

## What is Netra?

Netra (which means "eye" in Sanskrit) is a privacy-first, cross-platform browser engineered from the ground up to prioritize user control and system efficiency. Instead of relying on full-fledged massive browser engines like Chromium for the business logic, Netra builds its own lightweight core using Rust and a beautiful interface using Flutter, utilizing platform-native WebViews (like Edge WebView2 on Windows) strictly for rendering web content.

**Core Philosophy:**
- **Zero Telemetry on Launch:** Netra does not track you. Any future telemetry will be completely transparent and strictly opt-in.
- **DuckDuckGo Default:** Privacy defaults out-of-the-box.
- **Modular by Design:** Every third-party component sits behind a clean interface. The long-term goal is to replace external implementations (like `adblock-rust` or `sqflite` (planned to be replaced)) with Netra-owned code over time.
- **No Chrome Extensions:** Extension compatibility has been dropped by design to reduce complexity and attack surfaces.
- **Monetization Deferred:** The focus right now is purely on shipping a robust, secure, and blazing fast privacy browser.

Netra is uniquely developed. It is built natively via an AI-assisted approach where agents (Codex, Claude, Gemini, Opus) author the implementation details while a human engineer actively owns all architecture, interface boundaries, and system design. 

## Architecture Overview

Netra utilizes a robust, deterministic, layered architecture to ensure separation of concerns and thread safety across its hybrid tech stack:

1. **Flutter UI Layer (`lib/`):** Written in Dart. Manages the visual rendering and user interactions, utilizing `flutter_riverpod` for state management.
2. **Dart Interfaces:** Abstract the underlying engine, sending deterministic UI intent events.
3. **FFI Bridge (`flutter_rust_bridge`):** Handles bidirectional, memory-safe communication between Dart and Rust.
4. **Rust Core (`rust/`):** The brain of the browser. It owns the true state, handles the tab manager (with a planned LRU background suspension model), adblocking (currently `adblock-rust`), DNS-over-HTTPS (via Cloudflare 1.1.1.1), and download management.
5. **C++ WebView2 Bridge (`windows/runner/bridge/`):** Receives commands from Rust through FFI and manipulates Microsoft Edge WebView2 controls for rendering the actual web pages on Windows.

*(The Android and iOS versions will utilize Kotlin and Swift bridges respectively to interface with their native WebViews under the exact same Rust core model).*

## Folder Structure

```text
netra-browser/
├── android/       # Platform runner for Android (Phase 2 planned)
├── ios/           # Platform runner for iOS (Phase 2 planned)
├── lib/           # Flutter UI layer and Dart application logic
│   └── netra/     # Application components and infrastructure
├── linux/         # Platform runner for Linux (Phase 3 planned)
├── macos/         # Platform runner for macOS (Phase 3 planned)
├── rust/          # Core browser logic (Rust crate, FFI bridge)
├── test/          # Flutter widget and unit tests
├── web/           # Web asset structure (Not targeted for deployment)
└── windows/       # Windows C++ runner, FFI bridge, and WebView2 integration
```

## Getting Started

Follow these steps to set up the Netra environment on Windows, the primary supported platform for Phase 1.

### Prerequisites

1. **[Flutter SDK](https://docs.flutter.dev/get-started/install/windows)** (stable channel)
2. **[Rust Toolchain](https://rustup.rs/)** (via `rustup`)
3. **Visual Studio 2022** with the following installed:
   - "Desktop development with C++" workload
   - MSVC v143 build tools
   - Windows 10/11 SDK
4. **Microsoft Edge WebView2 Runtime:** Pre-installed on modern Windows 10/11. If you are on an older build, download the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) from Microsoft.
5. **Microsoft Edge WebView2 SDK:** 
   - The SDK is integrated directly on Windows environments. Ensure you have extracted the WebView2 SDK and moved or symlinked it to exactly `C:/webview2-sdk` — the `windows/runner/CMakeLists.txt` build configuration specifically looks for `C:/webview2-sdk/include/WebView2.h`. 
6. **Git**

### Clone & Build
1. Clone the repository:
   ```bash
   git clone <repository_url>
   ```
2. Navigate into the project:
   ```bash
   cd netra-browser
   ```
3. Fetch Dart dependencies:
   ```bash
   flutter pub get
   ```
4. Build and Run on Windows:
   ```bash
   flutter run -d windows
   ```
   *Note: Our custom `CMakeLists.txt` is configured to automatically run `cargo build` for the Rust FFI core (`netra_rust.dll`) as a pre-build dependency via the `rust_bridge_build` target. If you prefer to build the Rust core manually, you can navigate to `rust/` and run `cargo build --release`.*

### Verify Setup
- Run `flutter doctor` to confirm your Flutter and Visual Studio C++ environments are fully resolved.
- Run `rustc --version` and `cargo --version` to confirm your Rust toolchain.
- **Common Error:** *WebView2.h not found. Check SDK path.* -> Ensure the WebView2 SDK is located at `C:/webview2-sdk` so CMake can resolve the C++ headers and `WebView2LoaderStatic.lib`.

## Roadmap

| Phase | Platforms / Features | Status |
| :--- | :--- | :--- |
| **Phase 1** | Windows | 🚧 Work in Progress |
| **Phase 2** | Android, iOS | ⏳ Planned |
| **Phase 3** | macOS, Linux, AI features, Native Sync | ⏳ Planned |

## Built With AI

Netra's implementation is aggressively accelerated using AI agents, including **Codex**, **Claude Sonnet**, **Gemini**, and **Claude Opus**. 

AI writes the implementation, but **the engineer owns the system**. Every line of code, architectural boundary, and system invariant is validated and driven by human design. This project is proudly part of the `#buildinpublic` series showcasing human-guided AI software development.

## Contributing

Netra is currently **not open for code contributions** as the core architecture is being tightly stabilized and bootstrapped. 

However, stars ⭐ and issue reports or feature requests are incredibly welcome and appreciated!

## License

This project is licensed under the **MIT License**.

## Author

Built by **Rakesh Khokhar**  
www.linkedin.com/in/rakesh-khokhar-53b221349
