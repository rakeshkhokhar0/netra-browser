#include "bridge/ffi/rust_bridge.h"

#include <windows.h>
#include <wrl.h>

#include <cstdio>
#include <cstdlib>
#include <string>

#include "bridge/webview2/webview_manager.h"

namespace {

/// Matches the Rust `NativeExecutorBindings` C layout.
struct NativeExecutorBindings {
  int(__cdecl* create_tab)(const char* tab_id);
  int(__cdecl* close_tab)(const char* tab_id);
  int(__cdecl* set_active_tab)(const char* tab_id);
  int(__cdecl* load_url)(const char* tab_id, const char* url);
  int(__cdecl* go_back)(const char* tab_id);
  int(__cdecl* go_forward)(const char* tab_id);
  int(__cdecl* reload)(const char* tab_id);
  int(__cdecl* stop_loading)(const char* tab_id);
};

using RegisterNativeExecutorFn = int(__cdecl*)(NativeExecutorBindings);

/// Converts a UTF-8 string into a wide string for WebView2 APIs.
std::wstring ToWide(const std::string& value) {
  if (value.empty()) {
    return std::wstring();
  }

  const int size =
      MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, nullptr, 0);
  std::wstring wide_value(static_cast<size_t>(size), L'\0');
  MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, wide_value.data(), size);
  wide_value.pop_back();
  return wide_value;
}

/// Resolves a WebView instance from a tab id using the shared manager.
Microsoft::WRL::ComPtr<ICoreWebView2> GetWebViewForTab(
    const std::string& tab_id) {
  Microsoft::WRL::ComPtr<ICoreWebView2> webview;
  ICoreWebView2Controller* controller =
      WebViewManager::GetInstance().GetController(tab_id);
  if (controller == nullptr) {
    return webview;
  }

  controller->get_CoreWebView2(&webview);
  return webview;
}

/// Reads a non-empty UTF-8 string argument from a C string pointer.
bool ReadRequiredString(const char* raw_value, std::string* value) {
  if (raw_value == nullptr || value == nullptr) {
    return false;
  }

  const std::string string_value(raw_value);
  if (string_value.empty()) {
    return false;
  }

  *value = string_value;
  return true;
}

/// Creates a native WebView2 tab for the provided tab identifier.
int __cdecl CreateTabBridge(const char* raw_tab_id) {
  std::string tab_id;
  if (!ReadRequiredString(raw_tab_id, &tab_id)) {
    return 0;
  }

  auto& manager = WebViewManager::GetInstance();
  if (!manager.IsInitialized()) {
    return 0;
  }

  HWND parent_window = manager.GetParentWindow();
  if (parent_window == nullptr) {
    return 0;
  }

  RECT bounds{};
  ::GetClientRect(parent_window, &bounds);

  const HRESULT result = manager.CreateController(
      tab_id, bounds,
      [tab_id](HRESULT status, ICoreWebView2Controller* controller) {
        if (SUCCEEDED(status) && controller != nullptr) {
          WebViewManager::GetInstance().SetActiveTab(tab_id);
        }
      });

  return SUCCEEDED(result) ? 1 : 0;
}

/// Closes a native WebView2 tab for the provided tab identifier.
int __cdecl CloseTabBridge(const char* raw_tab_id) {
  std::string tab_id;
  if (!ReadRequiredString(raw_tab_id, &tab_id)) {
    return 0;
  }

  const HRESULT result = WebViewManager::GetInstance().DestroyController(tab_id);
  return SUCCEEDED(result) ? 1 : 0;
}

/// Marks the provided native tab as active.
int __cdecl SetActiveTabBridge(const char* raw_tab_id) {
  std::string tab_id;
  if (!ReadRequiredString(raw_tab_id, &tab_id)) {
    return 0;
  }

  const HRESULT result = WebViewManager::GetInstance().SetActiveTab(tab_id);
  return SUCCEEDED(result) ? 1 : 0;
}

/// Loads a URL in the provided native tab.
int __cdecl LoadUrlBridge(const char* raw_tab_id, const char* raw_url) {
  std::string tab_id;
  std::string url;
  if (!ReadRequiredString(raw_tab_id, &tab_id) ||
      !ReadRequiredString(raw_url, &url)) {
    return 0;
  }

  Microsoft::WRL::ComPtr<ICoreWebView2> webview = GetWebViewForTab(tab_id);
  if (!webview) {
    return 0;
  }

  const HRESULT result = webview->Navigate(ToWide(url).c_str());
  return SUCCEEDED(result) ? 1 : 0;
}

/// Requests backward navigation for the provided native tab.
int __cdecl GoBackBridge(const char* raw_tab_id) {
  std::string tab_id;
  if (!ReadRequiredString(raw_tab_id, &tab_id)) {
    return 0;
  }

  Microsoft::WRL::ComPtr<ICoreWebView2> webview = GetWebViewForTab(tab_id);
  if (!webview) {
    return 0;
  }

  const HRESULT result = webview->GoBack();
  return SUCCEEDED(result) ? 1 : 0;
}

/// Requests forward navigation for the provided native tab.
int __cdecl GoForwardBridge(const char* raw_tab_id) {
  std::string tab_id;
  if (!ReadRequiredString(raw_tab_id, &tab_id)) {
    return 0;
  }

  Microsoft::WRL::ComPtr<ICoreWebView2> webview = GetWebViewForTab(tab_id);
  if (!webview) {
    return 0;
  }

  const HRESULT result = webview->GoForward();
  return SUCCEEDED(result) ? 1 : 0;
}

/// Reloads the provided native tab.
int __cdecl ReloadBridge(const char* raw_tab_id) {
  std::string tab_id;
  if (!ReadRequiredString(raw_tab_id, &tab_id)) {
    return 0;
  }

  Microsoft::WRL::ComPtr<ICoreWebView2> webview = GetWebViewForTab(tab_id);
  if (!webview) {
    return 0;
  }

  const HRESULT result = webview->Reload();
  return SUCCEEDED(result) ? 1 : 0;
}

/// Stops loading in the provided native tab.
int __cdecl StopLoadingBridge(const char* raw_tab_id) {
  std::string tab_id;
  if (!ReadRequiredString(raw_tab_id, &tab_id)) {
    return 0;
  }

  Microsoft::WRL::ComPtr<ICoreWebView2> webview = GetWebViewForTab(tab_id);
  if (!webview) {
    return 0;
  }

  const HRESULT result = webview->Stop();
  return SUCCEEDED(result) ? 1 : 0;
}

}  // namespace

namespace netra::bridge::ffi {

bool ShouldBlockRequest(const std::string& url,
                        const std::string& resource_type) {
  // The Rust request pipeline will own the real filtering decision. This stub
  // preserves the native bridge contract so WebView2 interception can be wired
  // without embedding filtering rules in C++.
  (void)url;
  (void)resource_type;
  return false;
}

using HandleNativeEventFn = int(__cdecl*)(FfiNativeBrowserEvent);

/// Cached Rust DLL handle resolved once during the first FFI call.
static HMODULE g_rust_module = nullptr;

/// Cached function pointer to `netra_handle_native_event` in the Rust DLL.
static HandleNativeEventFn g_handle_native_event = nullptr;

/// Cached function pointer to `netra_register_native_executor` in the Rust DLL.
static RegisterNativeExecutorFn g_register_native_executor = nullptr;

/// Resolves the Rust DLL handle and caches all required function pointers.
///
/// This function is called lazily on the first FFI interaction and is a no-op
/// on every subsequent call.  All WebView2 event callbacks run on the UI thread
/// so no synchronisation is required for the static cache.
static void EnsureRustBindings() {
  if (g_rust_module != nullptr) {
    return;
  }

  g_rust_module = ::GetModuleHandleW(L"netra_rust.dll");
  if (g_rust_module == nullptr) {
    g_rust_module = ::LoadLibraryW(L"netra_rust.dll");
  }

  if (g_rust_module == nullptr) {
    std::printf("[C++] Failed to resolve netra_rust.dll\n");
    return;
  }

  g_handle_native_event = reinterpret_cast<HandleNativeEventFn>(
      ::GetProcAddress(g_rust_module, "netra_handle_native_event"));
  if (g_handle_native_event == nullptr) {
    std::printf("[C++] Failed to resolve netra_handle_native_event\n");
  }

  g_register_native_executor = reinterpret_cast<RegisterNativeExecutorFn>(
      ::GetProcAddress(g_rust_module, "netra_register_native_executor"));
  if (g_register_native_executor == nullptr) {
    std::printf("[C++] Failed to resolve netra_register_native_executor\n");
  }
}

void FreeEventStrings(const FfiNativeBrowserEvent& event) {
  std::free(event.tab_id);
  std::free(event.primary_string);
  std::free(event.secondary_string);
}

void SendNativeEventToRust(const FfiNativeBrowserEvent& event) {
  EnsureRustBindings();

  if (g_handle_native_event == nullptr) {
    FreeEventStrings(event);
    return;
  }

  const int status = g_handle_native_event(event);
  std::printf("[C++] Rust event handoff status: %d\n", status);
  FreeEventStrings(event);
}

void RegisterRustNativeExecutor() {
  EnsureRustBindings();

  if (g_register_native_executor == nullptr) {
    return;
  }

  const NativeExecutorBindings bindings{
      &CreateTabBridge,   &CloseTabBridge, &SetActiveTabBridge, &LoadUrlBridge,
      &GoBackBridge,      &GoForwardBridge, &ReloadBridge,      &StopLoadingBridge,
  };

  (void)g_register_native_executor(bindings);
}

}  // namespace netra::bridge::ffi
