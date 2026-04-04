import 'dart:developer' as developer;
import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:netra_browser/netra/ffi/bridge.dart';

import 'browser_provider.dart';

/// Thrown when Rust rejects tab creation because the hard tab limit was hit.
class TabLimitReachedException implements Exception {
  /// Creates a typed tab-limit failure using the authoritative Rust tab count.
  const TabLimitReachedException(this.limit);

  /// Maximum number of tabs currently allowed by the Rust core.
  final int limit;

  @override
  String toString() => 'Tab limit reached ($limit tabs).';
}

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

  /// Mirrors the current Rust-owned hard tab cap for user-facing messaging.
  static const int maxTabs = 20;

  final Ref _ref;
  final RustBridge _rust;
  Future<void> _createQueue = Future<void>.value();

  /// Creates a new browser tab and makes it the active tab.
  ///
  /// Rust generates the identifier and returns it to Flutter as the single
  /// authoritative tab ID for the rest of the app lifecycle.
  Future<String> createTab() async {
    final completer = Completer<String>();

    _createQueue = _createQueue.catchError((_) {}).then((_) async {
      try {
        completer.complete(await _createTabOnce());
      } catch (error, stackTrace) {
        completer.completeError(error, stackTrace);
      }
    });

    return completer.future;
  }

  /// Closes the browser tab associated with [tabId].
  Future<void> closeTab(String tabId) async {
    _logDebug('closeTab -> Rust only');
    await _rust.closeTab(tabId: tabId);
    await _ref.read(browserProvider).refreshState(clearErrorMessage: true);

    if (_ref.read(browserStateProvider).tabs.isEmpty) {
      await createTab();
    }
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

  Future<String> _createTabOnce() async {
    _logDebug('createTab -> Rust only');
    final currentTabCount = _ref.read(browserStateProvider).tabs.length;
    if (currentTabCount >= maxTabs) {
      throw const TabLimitReachedException(maxTabs);
    }

    try {
      final rustTab = await _rust.createTab();
      return rustTab.id;
    } on StateError {
      final rustState = await _rust.getBrowserState();
      final refreshedTabCount = rustState.tabs.length;
      if (refreshedTabCount >= maxTabs) {
        throw TabLimitReachedException(refreshedTabCount);
      }
      rethrow;
    }
  }
}
