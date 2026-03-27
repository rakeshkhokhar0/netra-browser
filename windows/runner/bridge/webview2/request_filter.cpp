#include "bridge/ffi/rust_bridge.h"

#include <windows.h>
#include <wrl.h>

#include <string>

#include <WebView2.h>

namespace {

/// Converts a wide string returned by WebView2 into a UTF-8 narrow string.
std::string ToUtf8(const std::wstring& value) {
  if (value.empty()) {
    return std::string();
  }

  const int size = WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, nullptr,
                                       0, nullptr, nullptr);
  std::string utf8_value(static_cast<size_t>(size), '\0');
  WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, utf8_value.data(), size,
                      nullptr, nullptr);
  utf8_value.pop_back();
  return utf8_value;
}

/// Reads an LPWSTR-style value from WebView2 and converts it into UTF-8.
std::string ReadAllocatedWideStringAndFree(LPWSTR value) {
  if (value == nullptr) {
    return std::string();
  }

  const std::wstring wide_value(value);
  CoTaskMemFree(value);
  return ToUtf8(wide_value);
}

/// Maps a WebView2 resource context to a stable string sent into the Rust
/// request pipeline.
std::string ResourceContextToString(COREWEBVIEW2_WEB_RESOURCE_CONTEXT context) {
  switch (context) {
    case COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT:
      return "document";
    case COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET:
      return "stylesheet";
    case COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE:
      return "image";
    case COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA:
      return "media";
    case COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT:
      return "script";
    case COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST:
      return "xhr";
    default:
      return "other";
  }
}

/// Creates an empty WebView2 response used when Rust decides to block a
/// request.
HRESULT CreateBlockedResponse(ICoreWebView2Environment* environment,
                              ICoreWebView2WebResourceRequestedEventArgs* args) {
  Microsoft::WRL::ComPtr<ICoreWebView2WebResourceResponse> response;
  const HRESULT response_result = environment->CreateWebResourceResponse(
      nullptr, 200, L"OK", L"Content-Type: text/plain", &response);
  if (FAILED(response_result) || !response) {
    return response_result;
  }

  return args->put_Response(response.Get());
}

}  // namespace

/// Registers a WebResourceRequested filter for all resources on a WebView.
///
/// The native filter extracts request metadata, delegates the allow/block
/// decision to Rust, and either supplies an empty success response or allows
/// the original request to continue. No filtering rules are stored in C++.
HRESULT RegisterRequestFilter(ICoreWebView2Environment* environment,
                              ICoreWebView2* webview) {
  if (environment == nullptr || webview == nullptr) {
    return E_INVALIDARG;
  }

  const HRESULT add_filter_result = webview->AddWebResourceRequestedFilter(
      L"*", COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL);
  if (FAILED(add_filter_result)) {
    return add_filter_result;
  }

  return webview->add_WebResourceRequested(
      Microsoft::WRL::Callback<ICoreWebView2WebResourceRequestedEventHandler>(
          [environment](ICoreWebView2* /*sender*/,
                        ICoreWebView2WebResourceRequestedEventArgs* args)
              -> HRESULT {
            if (args == nullptr) {
              return E_INVALIDARG;
            }

            Microsoft::WRL::ComPtr<ICoreWebView2WebResourceRequest> request;
            const HRESULT request_result = args->get_Request(&request);
            if (FAILED(request_result) || !request) {
              return request_result;
            }

            LPWSTR raw_uri = nullptr;
            const HRESULT uri_result = request->get_Uri(&raw_uri);
            if (FAILED(uri_result)) {
              return uri_result;
            }
            const std::string url = ReadAllocatedWideStringAndFree(raw_uri);

            COREWEBVIEW2_WEB_RESOURCE_CONTEXT context =
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
            const HRESULT context_result = args->get_ResourceContext(&context);
            if (FAILED(context_result)) {
              return context_result;
            }
            const std::string resource_type = ResourceContextToString(context);

            const bool should_block =
                netra::bridge::ffi::ShouldBlockRequest(url, resource_type);
            if (!should_block) {
              return S_OK;
            }

            return CreateBlockedResponse(environment, args);
          })
          .Get(),
      nullptr);
}
