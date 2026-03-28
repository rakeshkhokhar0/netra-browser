import 'dart:async';
import 'dart:developer' as developer;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:netra_browser/netra/di/modules/engine_module.dart';
import 'package:netra_browser/netra/engine/adapters/webview2/webview2_adapter.dart';
import 'package:netra_browser/netra/engine/models/browser_event.dart';
import 'package:netra_browser/netra/ffi/bridge.dart';
import 'package:netra_browser/netra/shared/utils/url_utils.dart';

import '../models/tab_state.dart';

/// Immutable Riverpod state exposed by [BrowserProvider].
///
/// This state is intentionally read-only from Flutter's perspective. The tab
/// list and active-tab information are refreshed from Rust snapshots rather
/// than being rebuilt locally in Dart. The only shell-local field retained
/// here is [errorMessage], which is used solely for UI feedback.
class BrowserProviderState {
  /// Creates an immutable browser-state snapshot for the UI layer.
  const BrowserProviderState({
    this.tabs = const <TabState>[],
    this.activeTabId,
    this.errorMessage,
  });

  /// Ordered list of tabs mirrored from the Rust browser core.
  final List<TabState> tabs;

  /// Identifier of the currently active tab mirrored from Rust state.
  final String? activeTabId;

  /// Latest non-fatal command or event error surfaced to the UI.
  final String? errorMessage;

  /// Whether the currently active tab is loading.
  bool get isLoading => tabs.any((tab) => tab.isActive && tab.isLoading);

  /// Total blocked-request count aggregated across all tabs.
  int get blockedRequestCount =>
      tabs.fold<int>(0, (sum, tab) => sum + tab.blockedCount);

  /// Returns a new immutable state snapshot with selected fields replaced.
  BrowserProviderState copyWith({
    List<TabState>? tabs,
    String? activeTabId,
    bool clearActiveTabId = false,
    String? errorMessage,
    bool clearErrorMessage = false,
  }) {
    return BrowserProviderState(
      tabs: tabs ?? this.tabs,
      activeTabId: clearActiveTabId ? null : (activeTabId ?? this.activeTabId),
      errorMessage: clearErrorMessage
          ? null
          : (errorMessage ?? this.errorMessage),
    );
  }
}

/// Riverpod state provider for the browser shell.
///
/// Widgets can watch this provider to react to browser state changes while
/// command-oriented code can continue reading [browserProvider] to access the
/// notifier methods directly.
final browserStateProvider =
    StateNotifierProvider<BrowserProvider, BrowserProviderState>((ref) {
      final engine = ref.read(engineProvider) as WebView2Adapter;
      final notifier = BrowserProvider(engine);
      ref.onDispose(notifier.dispose);
      return notifier;
    });

/// Exposes the [BrowserProvider] notifier instance used by command-oriented
/// UI code.
final browserProvider = Provider<BrowserProvider>(
  (ref) => ref.read(browserStateProvider.notifier),
);

/// Refresh-driven browser provider for the Flutter shell.
///
/// Responsibilities:
/// - routes browser commands through the Rust bridge
/// - listens to typed adapter events only to trigger Rust-state refreshes
/// - exposes a Flutter-friendly immutable state object derived from Rust
/// - avoids any local tab-list ownership or browser-state mutation logic
class BrowserProvider extends StateNotifier<BrowserProviderState> {
  /// Creates a browser provider backed by the given event adapter.
  BrowserProvider(this._engine)
    : super(const BrowserProviderState()) {
    _eventSubscription = _engine.events.listen(
      _handleEvent,
      onError: _handleStreamError,
    );
    _scheduleRefresh();
  }

  final WebView2Adapter _engine;
  final RustBridge _rust = RustBridge.instance;

  StreamSubscription<BrowserEvent>? _eventSubscription;

  /// Navigates the specified tab to the requested URL via Rust control.
  Future<void> navigate(String tabId, String url) async {
    final normalizedInput = normalizeNavigationInput(url);
    if (normalizedInput.isEmpty) {
      return;
    }

    try {
      _logDebug('loadUrl -> Rust state refresh');
      await _rust.loadUrl(tabId: tabId, url: normalizedInput);
      await refreshState(clearErrorMessage: true);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Requests backward navigation for the specified tab via Rust control.
  Future<void> goBack(String tabId) async {
    try {
      _logDebug('goBack -> Rust state refresh');
      await _rust.goBack(tabId: tabId);
      await refreshState(clearErrorMessage: true);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Requests forward navigation for the specified tab via Rust control.
  Future<void> goForward(String tabId) async {
    try {
      _logDebug('goForward -> Rust state refresh');
      await _rust.goForward(tabId: tabId);
      await refreshState(clearErrorMessage: true);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Requests a reload of the specified tab via Rust control.
  Future<void> reload(String tabId) async {
    try {
      _logDebug('reload -> Rust state refresh');
      await _rust.reload(tabId: tabId);
      await refreshState(clearErrorMessage: true);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Marks the specified tab as the active visible browser tab.
  Future<void> setActiveTab(String tabId) async {
    try {
      _logDebug('setActiveTab -> Rust state refresh');
      await _rust.setActiveTab(tabId: tabId);
      await refreshState(clearErrorMessage: true);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Updates the in-window bounds of the specified native browser surface.
  Future<void> setBounds(
    String tabId,
    double x,
    double y,
    double width,
    double height,
  ) async {
    try {
      await _rust.setBounds(
        tabId: tabId,
        x: x,
        y: y,
        width: width,
        height: height,
      );
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Refreshes the Flutter-visible browser snapshot from the Rust core.
  Future<void> refreshState({
    String? errorMessage,
    bool clearErrorMessage = false,
  }) async {
    try {
      final rustState = await _rust.getBrowserState();
      final browserState = _browserStateFromRust(
        rustState,
        previousErrorMessage: state.errorMessage,
        clearErrorMessage: clearErrorMessage,
        nextErrorMessage: errorMessage,
      );
      state = browserState;
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  @override
  void dispose() {
    _eventSubscription?.cancel();
    super.dispose();
  }

  void _handleEvent(BrowserEvent event) {
    if (event is TabCrashedBrowserEvent) {
      _scheduleRefresh(
        errorMessage: 'The active tab crashed while rendering content.',
      );
      return;
    }

    _scheduleRefresh(clearErrorMessage: true);
  }

  void _scheduleRefresh({
    String? errorMessage,
    bool clearErrorMessage = false,
  }) {
    unawaited(
      refreshState(
        errorMessage: errorMessage,
        clearErrorMessage: clearErrorMessage,
      ).catchError((Object error, StackTrace stackTrace) {
        _handleStreamError(error, stackTrace);
      }),
    );
  }

  void _handleStreamError(Object error, StackTrace stackTrace) {
    _setError(error);
  }

  void _setError(Object error) {
    state = state.copyWith(
      errorMessage: error.toString(),
    );
  }

  BrowserProviderState _browserStateFromRust(
    RustBrowserStateSnapshot rustState, {
    required String? previousErrorMessage,
    required bool clearErrorMessage,
    required String? nextErrorMessage,
  }) {
    final resolvedActiveTabId = rustState.activeTabId;

    final tabs = rustState.tabs
        .map((rawTab) {
          return TabState.fromMap({
            ...rawTab,
            'isActive':
                resolvedActiveTabId != null &&
                rawTab['id'] == resolvedActiveTabId,
          });
        })
        .toList(growable: false);

    return BrowserProviderState(
      tabs: tabs,
      activeTabId: resolvedActiveTabId,
      errorMessage: clearErrorMessage
          ? null
          : (nextErrorMessage ?? previousErrorMessage),
    );
  }

  void _logDebug(String message) {
    developer.log(message, name: 'NetraBrowser');
  }
}
