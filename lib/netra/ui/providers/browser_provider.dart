import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:netra_browser/netra/engine/adapters/webview2/webview2_adapter.dart';
import 'package:netra_browser/netra/di/modules/engine_module.dart';

/// Exposes the browser provider instance used by the Flutter UI layer.
///
/// The provider creates a thin presentation-facing wrapper around the engine
/// so widgets can issue browser commands through Riverpod without depending
/// directly on engine adapter construction details.
final browserProvider = Provider<BrowserProvider>(
  (ref) => BrowserProvider(ref.read(engineProvider) as WebView2Adapter),
);

/// Provides a simple UI-facing interface for browser engine commands.
///
/// This provider wrapper owns only method delegation from the UI layer into
/// the engine adapter. It does not contain business logic, maintain complex
/// state, or interpret browser results beyond forwarding asynchronous command
/// completion back to callers.
class BrowserProvider {
  /// Creates a browser provider backed by the given engine adapter.
  const BrowserProvider(this._engine);

  /// Stateless engine adapter used to forward UI commands to the native bridge.
  final WebView2Adapter _engine;

  /// Navigates the specified tab to the requested URL.
  Future<void> navigate(String tabId, String url) {
    return _engine.navigate(tabId, url);
  }

  /// Requests backward navigation for the specified tab.
  Future<void> goBack(String tabId) {
    return _engine.goBack(tabId);
  }

  /// Requests forward navigation for the specified tab.
  Future<void> goForward(String tabId) {
    return _engine.goForward(tabId);
  }

  /// Requests a reload of the current page for the specified tab.
  Future<void> reload(String tabId) {
    return _engine.reload(tabId);
  }

  /// Marks the specified tab as the active visible native browser surface.
  Future<void> setActiveTab(String tabId) {
    return _engine.setActiveTab(tabId);
  }

  /// Updates the in-window bounds of the specified native browser surface.
  Future<void> setBounds(
    String tabId,
    double x,
    double y,
    double width,
    double height,
  ) {
    return _engine.setBounds(tabId, x, y, width, height);
  }
}
