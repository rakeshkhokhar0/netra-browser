#include "bridge/webview2/webview_manager.h"

#include <windows.h>
#include <wrl.h>

#include <memory>
#include <optional>
#include <string>
#include <utility>

#include <flutter/encodable_value.h>
#include <flutter/method_call.h>
#include <flutter/method_channel.h>
#include <flutter/standard_method_codec.h>

/// Emits a tab-created event into the Flutter event stream.
void EmitTabCreated(const std::string& tab_id);

/// Emits a tab-closed event into the Flutter event stream.
void EmitTabClosed(const std::string& tab_id);

namespace {

using EncodableMap = flutter::EncodableMap;
using EncodableValue = flutter::EncodableValue;
using MethodCall = flutter::MethodCall<EncodableValue>;
using MethodResult = flutter::MethodResult<EncodableValue>;
using MethodChannel = flutter::MethodChannel<EncodableValue>;

/// Returns the map entry for a string key when the argument payload is a map.
const EncodableValue* FindArgument(const EncodableMap& arguments,
                                   const char* key) {
  const auto value = arguments.find(EncodableValue(key));
  if (value == arguments.end()) {
    return nullptr;
  }

  return &value->second;
}

/// Extracts a required string argument from a Flutter method payload.
std::optional<std::string> ReadStringArgument(const EncodableMap& arguments,
                                              const char* key) {
  const EncodableValue* value = FindArgument(arguments, key);
  if (value == nullptr) {
    return std::nullopt;
  }

  const auto string_value = std::get_if<std::string>(value);
  if (string_value == nullptr || string_value->empty()) {
    return std::nullopt;
  }

  return *string_value;
}

/// Extracts a required rectangle argument from a Flutter method payload.
std::optional<RECT> ReadBoundsArgument(const EncodableMap& arguments,
                                       const char* key) {
  const EncodableValue* value = FindArgument(arguments, key);
  if (value == nullptr) {
    return std::nullopt;
  }

  const auto bounds_map = std::get_if<EncodableMap>(value);
  if (bounds_map == nullptr) {
    return std::nullopt;
  }

  const EncodableValue* left = FindArgument(*bounds_map, "left");
  const EncodableValue* top = FindArgument(*bounds_map, "top");
  const EncodableValue* right = FindArgument(*bounds_map, "right");
  const EncodableValue* bottom = FindArgument(*bounds_map, "bottom");
  if (left == nullptr || top == nullptr || right == nullptr || bottom == nullptr) {
    return std::nullopt;
  }

  const auto left_value = std::get_if<int32_t>(left);
  const auto top_value = std::get_if<int32_t>(top);
  const auto right_value = std::get_if<int32_t>(right);
  const auto bottom_value = std::get_if<int32_t>(bottom);
  if (left_value == nullptr || top_value == nullptr || right_value == nullptr ||
      bottom_value == nullptr) {
    return std::nullopt;
  }

  return RECT{
      .left = *left_value,
      .top = *top_value,
      .right = *right_value,
      .bottom = *bottom_value,
  };
}

/// Converts a UTF-8 string into a wide string for WebView2 APIs.
std::wstring ToWide(const std::string& value) {
  if (value.empty()) {
    return std::wstring();
  }

  const int size = MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, nullptr, 0);
  std::wstring wide_value(static_cast<size_t>(size), L'\0');
  MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, wide_value.data(), size);
  wide_value.pop_back();
  return wide_value;
}

/// Resolves a WebView instance from a tab id using the manager-owned controller.
Microsoft::WRL::ComPtr<ICoreWebView2> GetWebViewForTab(const std::string& tab_id) {
  Microsoft::WRL::ComPtr<ICoreWebView2> webview;
  ICoreWebView2Controller* controller =
      WebViewManager::GetInstance().GetController(tab_id);
  if (controller == nullptr) {
    return webview;
  }

  controller->get_CoreWebView2(&webview);
  return webview;
}

/// Reports a generic invalid-argument error back to Flutter.
void ReturnInvalidArguments(MethodResult* result, const char* message) {
  result->Error("invalid_arguments", message);
}

/// Reports an HRESULT failure back to Flutter.
void ReturnHresultError(MethodResult* result,
                        const char* method_name,
                        HRESULT result_code) {
  result->Error(method_name, std::to_string(static_cast<long>(result_code)));
}

/// Verifies the current method call is running on the UI thread captured when
/// the handler was registered.
bool EnsureUiThread(MethodResult* result, DWORD ui_thread_id) {
  if (::GetCurrentThreadId() == ui_thread_id) {
    return true;
  }

  result->Error("wrong_thread", "MethodChannel call must run on the UI thread.");
  return false;
}

/// Handles Flutter MethodChannel calls and routes them to WebView2 APIs through
/// the shared WebView manager.
///
/// The handler parses input arguments, performs minimal validation, and invokes
/// controller operations. It intentionally does not store application state,
/// emit events, or perform browser business logic.
void HandleMethodCall(HWND parent_window,
                      DWORD ui_thread_id,
                      const MethodCall& call,
                      std::unique_ptr<MethodResult> result) {
  if (!EnsureUiThread(result.get(), ui_thread_id)) {
    return;
  }

  const auto* arguments = std::get_if<EncodableMap>(call.arguments());

  if (call.method_name() == "createTab") {
    if (arguments == nullptr) {
      ReturnInvalidArguments(result.get(), "createTab requires arguments.");
      return;
    }

    const auto tab_id = ReadStringArgument(*arguments, "tab_id");
    if (!tab_id.has_value()) {
      ReturnInvalidArguments(result.get(), "createTab requires a non-empty tab_id.");
      return;
    }

    RECT bounds{};
    ::GetClientRect(parent_window, &bounds);

    if (!WebViewManager::GetInstance().IsInitialized()) {
      result->Error(
          "engine_not_ready",
          "WebView2 environment is still initializing. Try again in a moment.");
      return;
    }

    std::shared_ptr<MethodResult> shared_result(std::move(result));
    const std::string created_tab_id = *tab_id;

    const HRESULT create_result = WebViewManager::GetInstance().CreateController(
        *tab_id, bounds,
        [shared_result, created_tab_id](HRESULT status,
                                        ICoreWebView2Controller* controller) {
          if (FAILED(status) || controller == nullptr) {
            ReturnHresultError(shared_result.get(), "createTab", status);
            return;
          }

          EmitTabCreated(created_tab_id);
          shared_result->Success(EncodableValue(true));
        });

    if (FAILED(create_result)) {
      ReturnHresultError(shared_result.get(), "createTab", create_result);
    }

    return;
  }

  if (call.method_name() == "closeTab") {
    if (arguments == nullptr) {
      ReturnInvalidArguments(result.get(), "closeTab requires arguments.");
      return;
    }

    const auto tab_id = ReadStringArgument(*arguments, "tab_id");
    if (!tab_id.has_value()) {
      ReturnInvalidArguments(result.get(), "closeTab requires a non-empty tab_id.");
      return;
    }

    const HRESULT destroy_result =
        WebViewManager::GetInstance().DestroyController(*tab_id);
    if (FAILED(destroy_result)) {
      ReturnHresultError(result.get(), "closeTab", destroy_result);
      return;
    }

    EmitTabClosed(*tab_id);
    result->Success(EncodableValue(true));
    return;
  }

  if (call.method_name() == "clearData") {
    result->Error(
        "not_supported",
        "clearData requires profile access support in WebViewManager.");
    return;
  }

  if (arguments == nullptr) {
    ReturnInvalidArguments(result.get(), "Method requires arguments.");
    return;
  }

  const auto tab_id = ReadStringArgument(*arguments, "tab_id");
  if (!tab_id.has_value()) {
    ReturnInvalidArguments(result.get(), "Method requires a non-empty tab_id.");
    return;
  }

  if (call.method_name() == "setBounds") {
    const auto bounds = ReadBoundsArgument(*arguments, "bounds");
    if (!bounds.has_value()) {
      ReturnInvalidArguments(result.get(), "setBounds requires bounds.");
      return;
    }

    const HRESULT bounds_result =
        WebViewManager::GetInstance().SetBounds(*tab_id, *bounds);
    if (FAILED(bounds_result)) {
      ReturnHresultError(result.get(), "setBounds", bounds_result);
      return;
    }

    result->Success(EncodableValue(true));
    return;
  }

  ICoreWebView2Controller* controller =
      WebViewManager::GetInstance().GetController(*tab_id);
  if (controller == nullptr) {
    result->Error("missing_tab", "No controller exists for the provided tab_id.");
    return;
  }

  Microsoft::WRL::ComPtr<ICoreWebView2> webview = GetWebViewForTab(*tab_id);
  if (!webview) {
    result->Error("missing_webview", "No WebView instance exists for the tab.");
    return;
  }

  if (call.method_name() == "navigate") {
    const auto url = ReadStringArgument(*arguments, "url");
    if (!url.has_value()) {
      ReturnInvalidArguments(result.get(), "navigate requires a non-empty url.");
      return;
    }

    const HRESULT navigate_result = webview->Navigate(ToWide(*url).c_str());
    if (FAILED(navigate_result)) {
      ReturnHresultError(result.get(), "navigate", navigate_result);
      return;
    }

    result->Success(EncodableValue(true));
    return;
  }

  if (call.method_name() == "goBack") {
    const HRESULT back_result = webview->GoBack();
    if (FAILED(back_result)) {
      ReturnHresultError(result.get(), "goBack", back_result);
      return;
    }

    result->Success(EncodableValue(true));
    return;
  }

  if (call.method_name() == "goForward") {
    const HRESULT forward_result = webview->GoForward();
    if (FAILED(forward_result)) {
      ReturnHresultError(result.get(), "goForward", forward_result);
      return;
    }

    result->Success(EncodableValue(true));
    return;
  }

  if (call.method_name() == "reload") {
    const HRESULT reload_result = webview->Reload();
    if (FAILED(reload_result)) {
      ReturnHresultError(result.get(), "reload", reload_result);
      return;
    }

    result->Success(EncodableValue(true));
    return;
  }

  if (call.method_name() == "setActiveTab") {
    const HRESULT visible_result =
        WebViewManager::GetInstance().SetActiveTab(*tab_id);
    if (FAILED(visible_result)) {
      ReturnHresultError(result.get(), "setActiveTab", visible_result);
      return;
    }

    result->Success(EncodableValue(true));
    return;
  }

  if (call.method_name() == "stopLoading") {
    const HRESULT stop_result = webview->Stop();
    if (FAILED(stop_result)) {
      ReturnHresultError(result.get(), "stopLoading", stop_result);
      return;
    }

    result->Success(EncodableValue(true));
    return;
  }

  result->NotImplemented();
}

}  // namespace

/// Registers the native browser MethodChannel handler for WebView2 commands.
///
/// The handler is attached to the provided Flutter binary messenger and uses
/// the supplied parent window to size newly created controllers. All method
/// calls are expected to execute on the UI thread captured during registration.
void RegisterWebViewMethodHandler(flutter::BinaryMessenger* messenger,
                                  HWND parent_window) {
  auto channel = std::make_unique<MethodChannel>(
      messenger, "netra/browser/methods",
      &flutter::StandardMethodCodec::GetInstance());

  const DWORD ui_thread_id = ::GetCurrentThreadId();

  channel->SetMethodCallHandler(
      [parent_window, ui_thread_id](
          const MethodCall& call,
          std::unique_ptr<MethodResult> result) {
        HandleMethodCall(parent_window, ui_thread_id, call, std::move(result));
      });

  channel.release();
}
