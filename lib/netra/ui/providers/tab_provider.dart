import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:netra_browser/netra/di/modules/engine_module.dart';
import 'package:netra_browser/netra/engine/adapters/webview2/webview2_adapter.dart';
import 'package:uuid/uuid.dart';

/// Exposes the tab-operation provider used by the Flutter shell.
///
/// This provider creates a thin lifecycle-only wrapper around the engine
/// adapter so widgets and higher-level coordinators can perform tab
/// operations without depending on engine construction details directly.
final tabProvider = Provider<TabProvider>((ref) {
  final engine = ref.read(engineProvider) as WebView2Adapter;
  return TabProvider(engine);
});

/// Wraps browser tab lifecycle operations for the Flutter layer.
///
/// Responsibilities:
/// - create browser tabs
/// - close browser tabs
/// - switch the active browser tab
///
/// This provider intentionally does not store browser state, does not handle
/// navigation, and does not call the native method channel directly.
class TabProvider {
  /// Creates a lifecycle-only tab provider backed by the engine adapter.
  TabProvider(this._engine);

  final WebView2Adapter _engine;
  final Uuid _uuid = const Uuid();

  /// Creates a new browser tab and makes it the active tab.
  ///
  /// A unique identifier is generated in Dart so the same tab id can be used
  /// consistently across the shell and native layers.
  Future<String> createTab() async {
    final tabId = _uuid.v4();
    await _engine.createTab(tabId);
    await _engine.setActiveTab(tabId);
    return tabId;
  }

  /// Closes the browser tab associated with [tabId].
  Future<void> closeTab(String tabId) {
    return _engine.closeTab(tabId);
  }

  /// Switches the active browser tab to [tabId].
  Future<void> switchTab(String tabId) {
    return _engine.setActiveTab(tabId);
  }
}
