#include "bridge/webview2/webview_manager.h"

#include <WebView2EnvironmentOptions.h>
#include <string>
#include <utility>

/// Registers the request interception hook on a newly created WebView.
HRESULT RegisterRequestFilter(ICoreWebView2Environment* environment,
                              ICoreWebView2* webview);

/// Emits an engine-ready event into the Rust FFI event pipeline.
void EmitEngineReady(const std::string& tab_id);

/// Emits a navigation-starting event into the Rust FFI event pipeline.
void EmitNavigationStarting(const std::string& tab_id, const std::string& url);

/// Emits a content-loading event into the Rust FFI event pipeline.
void EmitContentLoading(const std::string& tab_id);

/// Emits a URL-changed event into the Rust FFI event pipeline.
void EmitUrlChanged(const std::string& tab_id, const std::string& url);

/// Emits a navigation-completed event into the Rust FFI event pipeline.
void EmitNavigationCompleted(const std::string& tab_id,
                             const std::string& url);

/// Emits a navigation-failed event into the Rust FFI event pipeline.
void EmitNavigationFailed(const std::string& tab_id,
                          const std::string& url,
                          int error_code,
                          const std::string& description);

/// Emits a title-changed event into the Rust FFI event pipeline.
void EmitTitleChanged(const std::string& tab_id, const std::string& title);

/// Emits a favicon-changed event into the Rust FFI event pipeline.
void EmitFaviconChanged(const std::string& tab_id,
                        const std::string& favicon_url);

/// Emits a history-changed event into the Rust FFI event pipeline.
void EmitHistoryChanged(const std::string& tab_id,
                        bool can_go_back,
                        bool can_go_forward);

/// Emits a tab-crashed event into the Rust FFI event pipeline.
void EmitTabCrashed(const std::string& tab_id);

namespace {

constexpr const wchar_t kWebViewHostWindowClassName[] =
    L"NETRA_WEBVIEW2_HOST_WINDOW";

/// Default window procedure used by dedicated WebView2 host windows.
LRESULT CALLBACK WebViewHostWindowProc(HWND hwnd,
                                       UINT message,
                                       WPARAM wparam,
                                       LPARAM lparam) {
  switch (message) {
    case WM_SIZE:
      WebViewManager::GetInstance().ResizeHostWindowController(hwnd);
      return 0;
    case WM_DESTROY:
      WebViewManager::GetInstance().ReleaseHostWindow(hwnd);
      return 0;
    default:
      return DefWindowProc(hwnd, message, wparam, lparam);
  }
}

/// Ensures the dedicated WebView2 host window class is registered once.
ATOM EnsureWebViewHostWindowClass(HINSTANCE instance) {
  static ATOM host_window_class = 0;
  if (host_window_class != 0) {
    return host_window_class;
  }

  WNDCLASS window_class{};
  window_class.lpfnWndProc = WebViewHostWindowProc;
  window_class.hInstance = instance;
  window_class.hCursor = LoadCursor(nullptr, IDC_ARROW);
  window_class.lpszClassName = kWebViewHostWindowClassName;
  host_window_class = RegisterClass(&window_class);
  return host_window_class;
}

/// Creates an embedded native child window for a WebView2 controller.
HWND CreateHostWindowForTab(HWND parent_window, const RECT& bounds) {
  HINSTANCE instance = GetModuleHandle(nullptr);
  if (parent_window == nullptr || EnsureWebViewHostWindowClass(instance) == 0) {
    return nullptr;
  }

  HWND host_window = CreateWindowEx(
      0,
      kWebViewHostWindowClassName,
      L"",
      WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
      bounds.left,
      bounds.top,
      bounds.right - bounds.left,
      bounds.bottom - bounds.top,
      parent_window,
      nullptr,
      instance,
      nullptr);

  return host_window;
}

// Builds the browser arguments used when creating the shared WebView2
// environment. The manager enables DNS-over-HTTPS up front so all created
// controllers inherit the same network configuration baseline.
std::wstring BuildAdditionalBrowserArguments() {
  return std::wstring(L"--enable-features=DnsOverHttps "
                      L"--dns-over-https-mode=automatic "
                      L"--dns-over-https-templates=https://cloudflare-dns.com/dns-query");
}

/// Converts a WebView2 wide string into a UTF-8 std::string.
std::string ToUtf8AndFree(LPWSTR value) {
  if (value == nullptr) {
    return std::string();
  }

  const int size = WideCharToMultiByte(CP_UTF8, 0, value, -1, nullptr, 0,
                                       nullptr, nullptr);
  std::string utf8_value(static_cast<size_t>(size), '\0');
  WideCharToMultiByte(CP_UTF8, 0, value, -1, utf8_value.data(), size, nullptr,
                      nullptr);
  CoTaskMemFree(value);
  utf8_value.pop_back();
  return utf8_value;
}

/// Converts a WebView2 web-error status into a readable string.
std::string DescribeWebErrorStatus(COREWEBVIEW2_WEB_ERROR_STATUS status) {
  switch (status) {
    case COREWEBVIEW2_WEB_ERROR_STATUS_UNKNOWN:
      return "Unknown web error";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_COMMON_NAME_IS_INCORRECT:
      return "Certificate common name is incorrect";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_EXPIRED:
      return "Certificate expired";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CLIENT_CERTIFICATE_CONTAINS_ERRORS:
      return "Client certificate contains errors";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_REVOKED:
      return "Certificate revoked";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CERTIFICATE_IS_INVALID:
      return "Certificate is invalid";
    case COREWEBVIEW2_WEB_ERROR_STATUS_SERVER_UNREACHABLE:
      return "Server unreachable";
    case COREWEBVIEW2_WEB_ERROR_STATUS_TIMEOUT:
      return "Request timed out";
    case COREWEBVIEW2_WEB_ERROR_STATUS_ERROR_HTTP_INVALID_SERVER_RESPONSE:
      return "Invalid server response";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CONNECTION_ABORTED:
      return "Connection aborted";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CONNECTION_RESET:
      return "Connection reset";
    case COREWEBVIEW2_WEB_ERROR_STATUS_DISCONNECTED:
      return "Disconnected";
    case COREWEBVIEW2_WEB_ERROR_STATUS_CANNOT_CONNECT:
      return "Cannot connect";
    case COREWEBVIEW2_WEB_ERROR_STATUS_HOST_NAME_NOT_RESOLVED:
      return "Host name not resolved";
    case COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED:
      return "Operation canceled";
    case COREWEBVIEW2_WEB_ERROR_STATUS_REDIRECT_FAILED:
      return "Redirect failed";
    case COREWEBVIEW2_WEB_ERROR_STATUS_UNEXPECTED_ERROR:
      return "Unexpected web error";
    default:
      return "Navigation failed";
  }
}

/// Registers browser event callbacks for a created WebView2 instance.
void RegisterBrowserEvents(const std::string& tab_id, ICoreWebView2* webview) {
  webview->add_NavigationStarting(
      Microsoft::WRL::Callback<ICoreWebView2NavigationStartingEventHandler>(
          [tab_id](ICoreWebView2* /*sender*/,
                   ICoreWebView2NavigationStartingEventArgs* args) -> HRESULT {
            LPWSTR raw_uri = nullptr;
            if (SUCCEEDED(args->get_Uri(&raw_uri))) {
              EmitNavigationStarting(tab_id, ToUtf8AndFree(raw_uri));
            }
            return S_OK;
          })
          .Get(),
      nullptr);

  webview->add_ContentLoading(
      Microsoft::WRL::Callback<ICoreWebView2ContentLoadingEventHandler>(
          [tab_id](ICoreWebView2* /*sender*/,
                   ICoreWebView2ContentLoadingEventArgs* /*args*/) -> HRESULT {
            EmitContentLoading(tab_id);
            return S_OK;
          })
          .Get(),
      nullptr);

  webview->add_SourceChanged(
      Microsoft::WRL::Callback<ICoreWebView2SourceChangedEventHandler>(
          [tab_id](ICoreWebView2* sender,
                   ICoreWebView2SourceChangedEventArgs* /*args*/) -> HRESULT {
            if (sender == nullptr) {
              return S_OK;
            }

            LPWSTR raw_uri = nullptr;
            if (SUCCEEDED(sender->get_Source(&raw_uri))) {
              EmitUrlChanged(tab_id, ToUtf8AndFree(raw_uri));
            }
            return S_OK;
          })
          .Get(),
      nullptr);

  webview->add_NavigationCompleted(
      Microsoft::WRL::Callback<ICoreWebView2NavigationCompletedEventHandler>(
          [tab_id](ICoreWebView2* sender,
                   ICoreWebView2NavigationCompletedEventArgs* args) -> HRESULT {
            BOOL is_success = FALSE;
            args->get_IsSuccess(&is_success);

            LPWSTR raw_uri = nullptr;
            std::string url;
            if (sender != nullptr) {
              if (SUCCEEDED(sender->get_Source(&raw_uri))) {
                url = ToUtf8AndFree(raw_uri);
              }
            }

            if (is_success == TRUE) {
              EmitNavigationCompleted(tab_id, url);
              return S_OK;
            }

            COREWEBVIEW2_WEB_ERROR_STATUS web_error_status =
                COREWEBVIEW2_WEB_ERROR_STATUS_UNKNOWN;
            args->get_WebErrorStatus(&web_error_status);
            EmitNavigationFailed(
                tab_id,
                url,
                static_cast<int>(web_error_status),
                DescribeWebErrorStatus(web_error_status));
            return S_OK;
          })
          .Get(),
      nullptr);

  webview->add_DocumentTitleChanged(
      Microsoft::WRL::Callback<ICoreWebView2DocumentTitleChangedEventHandler>(
          [tab_id](ICoreWebView2* sender, IUnknown* /*args*/) -> HRESULT {
            if (sender == nullptr) {
              return S_OK;
            }

            LPWSTR raw_title = nullptr;
            if (SUCCEEDED(sender->get_DocumentTitle(&raw_title))) {
              EmitTitleChanged(tab_id, ToUtf8AndFree(raw_title));
            }
            return S_OK;
          })
          .Get(),
      nullptr);

  Microsoft::WRL::ComPtr<ICoreWebView2_15> webview15;
  if (SUCCEEDED(webview->QueryInterface(IID_PPV_ARGS(&webview15))) &&
      webview15) {
    webview15->add_FaviconChanged(
        Microsoft::WRL::Callback<ICoreWebView2FaviconChangedEventHandler>(
            [tab_id](ICoreWebView2* sender, IUnknown* /*args*/) -> HRESULT {
              if (sender == nullptr) {
                return S_OK;
              }

              Microsoft::WRL::ComPtr<ICoreWebView2_15> sender_webview15;
              if (FAILED(sender->QueryInterface(IID_PPV_ARGS(&sender_webview15))) ||
                  !sender_webview15) {
                return S_OK;
              }

              LPWSTR raw_favicon_uri = nullptr;
              if (SUCCEEDED(sender_webview15->get_FaviconUri(&raw_favicon_uri))) {
                EmitFaviconChanged(tab_id, ToUtf8AndFree(raw_favicon_uri));
              }
              return S_OK;
            })
            .Get(),
        nullptr);
  }

  webview->add_HistoryChanged(
      Microsoft::WRL::Callback<ICoreWebView2HistoryChangedEventHandler>(
          [tab_id](ICoreWebView2* sender, IUnknown* /*args*/) -> HRESULT {
            if (sender == nullptr) {
              return S_OK;
            }

            BOOL can_go_back = FALSE;
            BOOL can_go_forward = FALSE;
            sender->get_CanGoBack(&can_go_back);
            sender->get_CanGoForward(&can_go_forward);
            EmitHistoryChanged(tab_id, can_go_back == TRUE,
                               can_go_forward == TRUE);
            return S_OK;
          })
          .Get(),
      nullptr);

  webview->add_ProcessFailed(
      Microsoft::WRL::Callback<ICoreWebView2ProcessFailedEventHandler>(
          [tab_id](ICoreWebView2* /*sender*/,
                   ICoreWebView2ProcessFailedEventArgs* /*args*/) -> HRESULT {
            EmitTabCrashed(tab_id);
            return S_OK;
          })
          .Get(),
      nullptr);
}

}  // namespace

WebViewManager& WebViewManager::GetInstance() {
  static WebViewManager instance;
  return instance;
}

HRESULT WebViewManager::Initialize(HWND parent_window,
                                   EnvironmentInitializedCallback callback) {
  if (!callback) {
    return E_POINTER;
  }

  CaptureUiThread();

  const HRESULT thread_result = EnsureUiThread();
  if (FAILED(thread_result)) {
    callback(thread_result);
    return thread_result;
  }

  if (parent_window == nullptr) {
    callback(E_INVALIDARG);
    return E_INVALIDARG;
  }

  parent_window_ = parent_window;

  if (environment_) {
    callback(S_OK);
    return S_OK;
  }

  const std::wstring additional_browser_arguments =
      BuildAdditionalBrowserArguments();

  auto options = Microsoft::WRL::Make<CoreWebView2EnvironmentOptions>();
  options->put_AdditionalBrowserArguments(additional_browser_arguments.c_str());

  return CreateCoreWebView2EnvironmentWithOptions(
      nullptr, nullptr, options.Get(),
      Microsoft::WRL::Callback<
          ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler>(
          [this, callback](
              HRESULT result,
              ICoreWebView2Environment* created_environment) -> HRESULT {
            if (SUCCEEDED(result)) {
              environment_ = created_environment;
              EmitEngineReady("global");
            }

            callback(result);
            return S_OK;
          })
          .Get());
}

HRESULT WebViewManager::CreateController(const std::string& tab_id,
                                         const RECT& bounds,
                                         ControllerCreatedCallback callback) {
  if (!callback) {
    return E_POINTER;
  }

  const HRESULT thread_result = EnsureUiThread();
  if (FAILED(thread_result)) {
    callback(thread_result, nullptr);
    return thread_result;
  }

  if (tab_id.empty() || parent_window_ == nullptr || !environment_) {
    callback(E_INVALIDARG, nullptr);
    return E_INVALIDARG;
  }

  HWND host_window = nullptr;
  const auto existing = controllers_.find(tab_id);
  if (existing != controllers_.end()) {
    callback(S_OK, existing->second.Get());
    return S_OK;
  }

  const auto existing_host = host_windows_.find(tab_id);
  if (existing_host != host_windows_.end()) {
    host_window = existing_host->second;
    SetWindowPos(
        host_window,
        nullptr,
        bounds.left,
        bounds.top,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
        SWP_NOACTIVATE | SWP_NOZORDER);
  } else {
    host_window = CreateHostWindowForTab(parent_window_, bounds);
    if (host_window == nullptr) {
      callback(E_FAIL, nullptr);
      return E_FAIL;
    }
    host_windows_[tab_id] = host_window;
  }

  return environment_->CreateCoreWebView2Controller(
      host_window,
      Microsoft::WRL::Callback<
          ICoreWebView2CreateCoreWebView2ControllerCompletedHandler>(
          [this, tab_id, callback](
              HRESULT result,
              ICoreWebView2Controller* created_controller) -> HRESULT {
            if (FAILED(result) || created_controller == nullptr) {
              callback(result, nullptr);
              return S_OK;
            }

            RECT host_bounds{};
            const auto host_window_entry = host_windows_.find(tab_id);
            if (host_window_entry == host_windows_.end() ||
                host_window_entry->second == nullptr) {
              callback(E_FAIL, nullptr);
              return S_OK;
            }
            GetClientRect(host_window_entry->second, &host_bounds);

            const HRESULT bounds_result = created_controller->put_Bounds(host_bounds);
            if (FAILED(bounds_result)) {
              callback(bounds_result, nullptr);
              return S_OK;
            }

            Microsoft::WRL::ComPtr<ICoreWebView2> webview;
            const HRESULT webview_result =
                created_controller->get_CoreWebView2(&webview);
            if (FAILED(webview_result) || !webview) {
              callback(webview_result, nullptr);
              return S_OK;
            }

            const HRESULT filter_result =
                RegisterRequestFilter(environment_.Get(), webview.Get());
            if (FAILED(filter_result)) {
              callback(filter_result, nullptr);
              return S_OK;
            }

            RegisterBrowserEvents(tab_id, webview.Get());

            Microsoft::WRL::ComPtr<ICoreWebView2Controller> controller =
                created_controller;
            controllers_[tab_id] = controller;
            callback(S_OK, controller.Get());

            // Register script injection without blocking controller creation.
            webview->AddScriptToExecuteOnDocumentCreated(
                kDocumentCreatedScript,
                Microsoft::WRL::Callback<
                    ICoreWebView2AddScriptToExecuteOnDocumentCreatedCompletedHandler>(
                    [](HRESULT /*add_result*/, PCWSTR /*script_id*/) -> HRESULT {
                      return S_OK;
                    })
                    .Get());

            return S_OK;
          })
          .Get());
}

HRESULT WebViewManager::DestroyController(const std::string& tab_id) {
  const HRESULT thread_result = EnsureUiThread();
  if (FAILED(thread_result)) {
    return thread_result;
  }

  const auto controller_entry = controllers_.find(tab_id);
  if (controller_entry == controllers_.end()) {
    return HRESULT_FROM_WIN32(ERROR_NOT_FOUND);
  }

  controller_entry->second->Close();
  controllers_.erase(controller_entry);

  HWND host_window = nullptr;
  const auto host_window_entry = host_windows_.find(tab_id);
  if (host_window_entry != host_windows_.end()) {
    host_window = host_window_entry->second;
    host_windows_.erase(host_window_entry);
  }

  if (host_window != nullptr) {
    DestroyWindow(host_window);
  }

  return S_OK;
}

HRESULT WebViewManager::SetBounds(const std::string& tab_id, const RECT& bounds) {
  const HRESULT thread_result = EnsureUiThread();
  if (FAILED(thread_result)) {
    return thread_result;
  }

  const auto host_window_entry = host_windows_.find(tab_id);
  if (host_window_entry == host_windows_.end() ||
      host_window_entry->second == nullptr) {
    return HRESULT_FROM_WIN32(ERROR_NOT_FOUND);
  }

  HWND host_window = host_window_entry->second;
  if (!SetWindowPos(
          host_window,
          nullptr,
          bounds.left,
          bounds.top,
          bounds.right - bounds.left,
          bounds.bottom - bounds.top,
          SWP_NOACTIVATE | SWP_NOZORDER)) {
    return HRESULT_FROM_WIN32(GetLastError());
  }

  ResizeHostWindowController(host_window);
  return S_OK;
}

HRESULT WebViewManager::SetActiveTab(const std::string& tab_id) {
  const HRESULT thread_result = EnsureUiThread();
  if (FAILED(thread_result)) {
    return thread_result;
  }

  const auto active_host_entry = host_windows_.find(tab_id);
  const auto active_controller_entry = controllers_.find(tab_id);
  if (active_host_entry == host_windows_.end() ||
      active_host_entry->second == nullptr ||
      active_controller_entry == controllers_.end() ||
      !active_controller_entry->second) {
    return HRESULT_FROM_WIN32(ERROR_NOT_FOUND);
  }

  for (const auto& host_entry : host_windows_) {
    const bool is_active = host_entry.first == tab_id;
    ShowWindow(host_entry.second, is_active ? SW_SHOW : SW_HIDE);

    const auto controller_entry = controllers_.find(host_entry.first);
    if (controller_entry != controllers_.end() && controller_entry->second) {
      controller_entry->second->put_IsVisible(is_active ? TRUE : FALSE);
    }
  }

  SetWindowPos(
      active_host_entry->second,
      HWND_TOP,
      0,
      0,
      0,
      0,
      SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
  ResizeHostWindowController(active_host_entry->second);
  return S_OK;
}

void WebViewManager::ResizeHostWindowController(HWND host_window) {
  if (FAILED(EnsureUiThread()) || host_window == nullptr) {
    return;
  }

  std::string tab_id;
  for (const auto& host_window_entry : host_windows_) {
    if (host_window_entry.second == host_window) {
      tab_id = host_window_entry.first;
      break;
    }
  }

  if (tab_id.empty()) {
    return;
  }

  const auto controller_entry = controllers_.find(tab_id);
  if (controller_entry == controllers_.end() || !controller_entry->second) {
    return;
  }

  RECT bounds{};
  GetClientRect(host_window, &bounds);
  controller_entry->second->put_Bounds(bounds);
}

void WebViewManager::ReleaseHostWindow(HWND host_window) {
  if (FAILED(EnsureUiThread()) || host_window == nullptr) {
    return;
  }

  std::string tab_id;
  for (const auto& host_window_entry : host_windows_) {
    if (host_window_entry.second == host_window) {
      tab_id = host_window_entry.first;
      break;
    }
  }

  if (tab_id.empty()) {
    return;
  }

  const auto controller_entry = controllers_.find(tab_id);
  if (controller_entry != controllers_.end()) {
    if (controller_entry->second) {
      controller_entry->second->Close();
    }
    controllers_.erase(controller_entry);
  }

  host_windows_.erase(tab_id);
}

ICoreWebView2Controller* WebViewManager::GetController(
    const std::string& tab_id) const {
  if (FAILED(EnsureUiThread())) {
    return nullptr;
  }

  const auto controller_entry = controllers_.find(tab_id);
  if (controller_entry == controllers_.end()) {
    return nullptr;
  }

  return controller_entry->second.Get();
}

bool WebViewManager::IsInitialized() const {
  return environment_ != nullptr;
}

HWND WebViewManager::GetParentWindow() const {
  return parent_window_;
}

HRESULT WebViewManager::EnsureUiThread() const {
  if (ui_thread_id_ == 0 || ::GetCurrentThreadId() != ui_thread_id_) {
    return HRESULT_FROM_WIN32(ERROR_INVALID_THREAD_ID);
  }

  return S_OK;
}

void WebViewManager::CaptureUiThread() {
  if (ui_thread_id_ == 0) {
    ui_thread_id_ = ::GetCurrentThreadId();
  }
}
