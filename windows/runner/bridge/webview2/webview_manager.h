#ifndef NETRA_BROWSER_WINDOWS_RUNNER_BRIDGE_WEBVIEW2_WEBVIEW_MANAGER_H_
#define NETRA_BROWSER_WINDOWS_RUNNER_BRIDGE_WEBVIEW2_WEBVIEW_MANAGER_H_

#include <windows.h>
#include <wrl.h>

#include <functional>
#include <string>
#include <unordered_map>

#include <WebView2.h>

/// Manages the shared WebView2 environment and tab-scoped controllers.
///
/// This manager is responsible only for environment creation and controller
/// lifecycle. It does not handle Flutter channels, emit browser events, or
/// perform navigation behavior. Access is restricted to the UI thread so the
/// internal controller map can remain consistent without additional threading
/// primitives.
class WebViewManager {
 public:
  /// Callback invoked after shared environment initialization finishes.
  ///
  /// The callback receives the final HRESULT so callers can decide how to
  /// surface or recover from startup failures.
  using EnvironmentInitializedCallback = std::function<void(HRESULT)>;

  /// Callback invoked after controller creation finishes for a tab.
  ///
  /// The callback receives the final HRESULT and the created controller when
  /// successful. A null controller indicates creation did not complete.
  using ControllerCreatedCallback =
      std::function<void(HRESULT, ICoreWebView2Controller*)>;

  /// Returns the global WebView manager instance.
  static WebViewManager& GetInstance();

  WebViewManager(const WebViewManager&) = delete;
  WebViewManager& operator=(const WebViewManager&) = delete;

  /// Initializes the shared WebView2 environment once for the process.
  ///
  /// The provided parent window is stored for later controller creation.
  /// Repeated calls after successful initialization reuse the existing
  /// environment and complete immediately with success.
  HRESULT Initialize(HWND parent_window, EnvironmentInitializedCallback callback);

  /// Creates a controller for the specified tab and stores it by tab id.
  ///
  /// If a controller already exists for the tab, the existing controller is
  /// returned through the callback instead of creating a duplicate instance.
  HRESULT CreateController(const std::string& tab_id,
                           const RECT& bounds,
                           ControllerCreatedCallback callback);

  /// Destroys the controller associated with the provided tab id.
  ///
  /// This closes the native controller and removes it from the internal map.
  HRESULT DestroyController(const std::string& tab_id);

  /// Updates the bounds of the embedded host surface associated with a tab.
  ///
  /// The provided rectangle is interpreted in parent-window coordinates and is
  /// used to position the native child host inside the app window.
  HRESULT SetBounds(const std::string& tab_id, const RECT& bounds);

  /// Marks the specified tab as the active visible embedded surface.
  ///
  /// The active host is shown while other tab hosts are hidden.
  HRESULT SetActiveTab(const std::string& tab_id);

  /// Returns the controller associated with the provided tab id if present.
  ///
  /// A null result indicates either missing state or access from a non-UI
  /// thread.
  ICoreWebView2Controller* GetController(const std::string& tab_id) const;

  /// Indicates whether the shared environment has already been initialized.
  bool IsInitialized() const;

  /// Resizes the controller hosted inside the provided native host window.
  void ResizeHostWindowController(HWND host_window);

  /// Releases controller and host window state when a host window is destroyed.
  void ReleaseHostWindow(HWND host_window);

 private:
  WebViewManager() = default;
  ~WebViewManager() = default;

  /// Ensures the manager is being accessed from the UI thread only.
  ///
  /// All controller and environment interaction must occur on the UI thread to
  /// keep WebView2 access predictable and the controller map consistent.
  HRESULT EnsureUiThread() const;

  /// Stores the calling thread as the UI thread on first initialization.
  ///
  /// The first successful initialization call establishes the thread affinity
  /// used by the manager for all later access checks.
  void CaptureUiThread();

  /// Default DNS-over-HTTPS template applied to the shared environment.
  static constexpr wchar_t kDefaultDohTemplate[] =
      L"https://1.1.1.1/dns-query";

  /// Stub script injected into every created document.
  ///
  /// The actual privacy and fingerprint-protection scripts will be wired in
  /// later. For now this verifies the initialization hook is in place.
  static constexpr wchar_t kDocumentCreatedScript[] =
      LR"(// Netra script injection stub. Real scripts are injected later.)";

  /// Parent window used for controller creation.
  HWND parent_window_ = nullptr;

  /// UI thread captured during first initialization.
  DWORD ui_thread_id_ = 0;

  /// Shared WebView2 environment reused by every tab controller.
  Microsoft::WRL::ComPtr<ICoreWebView2Environment> environment_;

  /// Tab id to controller mapping for active WebView instances.
  std::unordered_map<std::string,
                     Microsoft::WRL::ComPtr<ICoreWebView2Controller>>
      controllers_;

  /// Tab id to native host window mapping for active WebView instances.
  std::unordered_map<std::string, HWND> host_windows_;
};

#endif  // NETRA_BROWSER_WINDOWS_RUNNER_BRIDGE_WEBVIEW2_WEBVIEW_MANAGER_H_
