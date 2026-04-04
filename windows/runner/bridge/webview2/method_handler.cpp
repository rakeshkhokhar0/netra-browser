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
