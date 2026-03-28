import 'dart:developer' as developer;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:netra_browser/netra/ffi/bridge.dart';

import 'browser_provider.dart';

/// Exposes the tab-operation provider used by the Flutter shell.
///
/// This provider creates a thin lifecycle-only wrapper around the Rust bridge
/// so widgets and higher-level coordinators can perform tab operations without
/// depending on low-level bridge construction details directly.
final tabProvider = Provider<TabProvider>((ref) {
  return TabProvider(ref, RustBridge.instance);
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
  /// Creates a lifecycle-only tab provider backed by the Rust bridge.
  TabProvider(this._ref, this._rust);

  final Ref _ref;
  final RustBridge _rust;

  /// Creates a new browser tab and makes it the active tab.
  ///
  /// Rust generates the identifier and returns it to Flutter as the single
  /// authoritative tab ID for the rest of the app lifecycle.
  Future<String> createTab() async {
    _logDebug('createTab -> Rust only');
    final rustTab = await _rust.createTab();
    await _ref.read(browserProvider).refreshState(clearErrorMessage: true);

    return rustTab.id;
  }

  /// Closes the browser tab associated with [tabId].
  Future<void> closeTab(String tabId) async {
    _logDebug('closeTab -> Rust only');
    await _rust.closeTab(tabId: tabId);
    await _ref.read(browserProvider).refreshState(clearErrorMessage: true);
  }

  /// Switches the active browser tab to [tabId].
  Future<void> switchTab(String tabId) async {
    _logDebug('switchTab -> Rust only');
    await _rust.setActiveTab(tabId: tabId);
    await _ref.read(browserProvider).refreshState(clearErrorMessage: true);
  }

  void _logDebug(String message) {
    developer.log(message, name: 'NetraBrowser');
  }
}
