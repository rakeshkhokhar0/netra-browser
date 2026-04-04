#include "flutter_window.h"

#include <optional>

#include "bridge/ffi/rust_bridge.h"
#include "bridge/webview2/webview_manager.h"
#include "generated_plugin_registrant.h"

namespace flutter {
class BinaryMessenger;
}

/// Registers the native browser MethodChannel surface with the Flutter engine.
void RegisterWebViewMethodHandler(flutter::BinaryMessenger* messenger,
                                  HWND parent_window);

FlutterWindow::FlutterWindow(const flutter::DartProject& project)
    : project_(project) {}

FlutterWindow::~FlutterWindow() {}

bool FlutterWindow::OnCreate() {
  if (!Win32Window::OnCreate()) {
    return false;
  }

  RECT frame = GetClientArea();

  flutter_controller_ = std::make_unique<flutter::FlutterViewController>(
      frame.right - frame.left, frame.bottom - frame.top, project_);
  if (!flutter_controller_->engine() || !flutter_controller_->view()) {
    return false;
  }

  RegisterPlugins(flutter_controller_->engine());
  SetChildContent(flutter_controller_->view()->GetNativeWindow());

  // Restore the native bridge startup path so the Windows runner exposes the
  // WebView2 method channel and Rust FFI bridge.
  auto* messenger = flutter_controller_->engine()->messenger();
  RegisterWebViewMethodHandler(messenger, GetHandle());
  WebViewManager::GetInstance().Initialize(GetHandle(), [](HRESULT) {});
  netra::bridge::ffi::RegisterRustNativeExecutor();

  flutter_controller_->engine()->SetNextFrameCallback([&]() {
    this->Show();
  });

  flutter_controller_->ForceRedraw();

  return true;
}

void FlutterWindow::OnDestroy() {
  if (flutter_controller_) {
    flutter_controller_ = nullptr;
  }

  Win32Window::OnDestroy();
}

LRESULT FlutterWindow::MessageHandler(HWND hwnd,
                                      UINT const message,
                                      WPARAM const wparam,
                                      LPARAM const lparam) noexcept {
  if (flutter_controller_) {
    std::optional<LRESULT> result =
        flutter_controller_->HandleTopLevelWindowProc(hwnd, message, wparam,
                                                      lparam);
    if (result) {
      return *result;
    }
  }

  switch (message) {
    case WM_FONTCHANGE:
      flutter_controller_->engine()->ReloadSystemFonts();
      break;
  }

  return Win32Window::MessageHandler(hwnd, message, wparam, lparam);
}
