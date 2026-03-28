import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:netra_browser/netra/di/modules/engine_module.dart';
import 'package:netra_browser/netra/engine/adapters/webview2/webview2_adapter.dart';
import 'package:netra_browser/netra/engine/models/browser_event.dart';
import 'package:netra_browser/netra/shared/utils/url_utils.dart';

import '../models/tab_state.dart';
import 'tab_provider.dart';

/// Immutable Riverpod state owned by [BrowserProvider].
///
/// This model represents the shell-side browser snapshot that Flutter widgets
/// can observe without depending on the Rust or native bridge layers
/// directly. It contains only presentation-facing state derived from browser
/// events and command lifecycle updates.
class BrowserProviderState {
  /// Creates an immutable browser-provider state snapshot.
  const BrowserProviderState({
    this.tabs = const <TabState>[],
    this.activeTabId,
    this.isLoading = false,
    this.errorMessage,
    this.blockedRequestCount = 0,
  });

  /// Ordered list of browser tabs currently known to the Flutter layer.
  final List<TabState> tabs;

  /// Identifier of the tab currently treated as active by the UI layer.
  final String? activeTabId;

  /// Whether the browser is currently processing a loading/navigation action.
  final bool isLoading;

  /// Latest error message surfaced while processing browser commands or events.
  final String? errorMessage;

  /// Count of blocked network requests observed from the event stream.
  final int blockedRequestCount;

  /// Returns a new immutable state snapshot with selected fields replaced.
  BrowserProviderState copyWith({
    List<TabState>? tabs,
    String? activeTabId,
    bool clearActiveTabId = false,
    bool? isLoading,
    String? errorMessage,
    bool clearErrorMessage = false,
    int? blockedRequestCount,
  }) {
    return BrowserProviderState(
      tabs: tabs ?? this.tabs,
      activeTabId: clearActiveTabId ? null : (activeTabId ?? this.activeTabId),
      isLoading: isLoading ?? this.isLoading,
      errorMessage: clearErrorMessage
          ? null
          : (errorMessage ?? this.errorMessage),
      blockedRequestCount: blockedRequestCount ?? this.blockedRequestCount,
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
      final notifier = BrowserProvider(ref, engine);
      ref.onDispose(() {
        notifier.dispose();
      });
      return notifier;
    });

/// Exposes the [BrowserProvider] notifier instance used by command-oriented
/// UI code.
///
/// This compatibility provider lets existing widgets call browser commands via
/// `ref.read(browserProvider)` while the observable state lives in
/// [browserStateProvider].
final browserProvider = Provider<BrowserProvider>(
  (ref) => ref.read(browserStateProvider.notifier),
);

/// Manages browser state for the Flutter shell using Riverpod.
///
/// Responsibilities:
/// - stores the tab list and active-tab identifier
/// - tracks global loading and error state
/// - counts blocked requests reported by the event stream
/// - listens to typed browser events emitted by the engine layer
///
/// The provider does not contain UI code and does not call the low-level FFI
/// bridge directly. It uses the engine adapter as the single command/event
/// boundary for the Flutter layer.
class BrowserProvider extends StateNotifier<BrowserProviderState> {
  /// Creates a browser provider backed by the given engine adapter.
  BrowserProvider(this._ref, this._engine)
    : super(const BrowserProviderState()) {
    _eventSubscription = _engine.events.listen(
      _handleEvent,
      onError: _handleStreamError,
    );
  }

  final Ref _ref;

  /// Engine adapter used for browser commands and event delivery.
  final WebView2Adapter _engine;

  StreamSubscription<BrowserEvent>? _eventSubscription;

  /// Navigates the specified tab to the requested URL.
  ///
  /// The provider updates local loading state before delegating the command to
  /// the engine. Final URL synchronization is completed when the matching
  /// navigation event arrives.
  Future<void> navigate(String tabId, String url) async {
    final normalizedInput = normalizeNavigationInput(url);
    if (normalizedInput.isEmpty) {
      return;
    }

    _setCommandLoading(tabId);
    try {
      await _engine.navigate(tabId, normalizedInput);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Requests backward navigation for the specified tab.
  Future<void> goBack(String tabId) async {
    _setCommandLoading(tabId);
    try {
      await _engine.goBack(tabId);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Requests forward navigation for the specified tab.
  Future<void> goForward(String tabId) async {
    _setCommandLoading(tabId);
    try {
      await _engine.goForward(tabId);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Requests a reload of the current page for the specified tab.
  Future<void> reload(String tabId) async {
    _setCommandLoading(tabId);
    try {
      await _engine.reload(tabId);
    } catch (error) {
      _setError(error);
      rethrow;
    }
  }

  /// Marks the specified tab as the active visible native browser surface.
  Future<void> setActiveTab(String tabId) async {
    _activateTab(tabId);
    try {
      await _engine.setActiveTab(tabId);
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
      await _engine.setBounds(tabId, x, y, width, height);
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
    final type = event.type.toLowerCase();

    switch (type) {
      case 'tabcreated':
        _handleTabCreated(event.tabId);
        break;
      case 'tabclosed':
        _handleTabClosed(event.tabId);
        break;
      case 'navigationcompleted':
        if (event is NavigationCompletedBrowserEvent) {
          _handleNavigationCompleted(
            event.tabId,
            event.url,
            success: event.success,
          );
        } else {
          _handleNavigationCompleted(event.tabId, event.message ?? '');
        }
        break;
      case 'requestblocked':
        _handleBlockedRequest();
        break;
      case 'navigationstarting':
        if (event is NavigationStartingBrowserEvent) {
          _markTabLoading(event.tabId, url: event.url);
        }
        break;
      case 'contentloading':
        if (event is ContentLoadingBrowserEvent) {
          _markTabLoading(event.tabId);
        }
        break;
      case 'urlchanged':
        if (event is UrlChangedBrowserEvent) {
          _updateTab(
            event.tabId,
            (tab) => tab.copyWith(url: event.url),
          );
        }
        break;
      case 'titlechanged':
        if (event is TitleChangedBrowserEvent) {
          _updateTab(
            event.tabId,
            (tab) => tab.copyWith(title: event.title),
          );
        }
        break;
      case 'historychanged':
        if (event is HistoryChangedBrowserEvent) {
          _updateTab(
            event.tabId,
            (tab) => tab.copyWith(
              canGoBack: event.canGoBack,
              canGoForward: event.canGoForward,
            ),
          );
        }
        break;
      case 'tabcrashed':
        if (event is TabCrashedBrowserEvent) {
          _setError('The active tab crashed while rendering content.');
        }
        break;
      default:
        break;
    }
  }

  void _handleStreamError(Object error, StackTrace stackTrace) {
    _setError(error);
  }

  void _handleTabCreated(String? tabId) {
    if (tabId == null || tabId.isEmpty) {
      return;
    }

    final existingIndex = state.tabs.indexWhere((tab) => tab.id == tabId);
    final createdTab = TabState(
      id: tabId,
      title: '',
      url: '',
      isActive: true,
      isLoading: false,
      canGoBack: false,
      canGoForward: false,
      isSuspended: false,
    );

    final nextTabs = state.tabs
        .map((tab) => tab.copyWith(isActive: false))
        .toList(growable: true);

    if (existingIndex >= 0) {
      nextTabs[existingIndex] = createdTab;
    } else {
      nextTabs.add(createdTab);
    }

    state = state.copyWith(
      tabs: nextTabs,
      activeTabId: _resolveActiveTabId(nextTabs, tabId),
      isLoading: false,
      clearErrorMessage: true,
    );
  }

  Future<void> _handleTabClosed(String? tabId) async {
    if (tabId == null || tabId.isEmpty) {
      return;
    }

    final nextTabs = state.tabs.where((tab) => tab.id != tabId).toList();
    if (nextTabs.isEmpty) {
      final newTabId = await _ref.read(tabProvider).createTab();

      state = state.copyWith(
        tabs: const <TabState>[],
        activeTabId: newTabId,
        isLoading: false,
        clearErrorMessage: true,
      );

      return;
    }

    final nextActiveTabId = state.activeTabId == tabId
        ? (nextTabs.isNotEmpty ? nextTabs.first.id : null)
        : state.activeTabId;
    final normalizedTabs = nextTabs
        .map(
          (tab) => tab.copyWith(
            isActive: tab.id == nextActiveTabId,
          ),
        )
        .toList();

    state = state.copyWith(
      tabs: normalizedTabs,
      activeTabId: _resolveActiveTabId(normalizedTabs, nextActiveTabId),
      clearActiveTabId: nextActiveTabId == null,
      isLoading: false,
      clearErrorMessage: true,
    );
  }

  void _handleNavigationCompleted(
    String? tabId,
    String url, {
    bool success = true,
  }) {
    if (tabId == null || tabId.isEmpty) {
      return;
    }

    final normalizedUrl = _normalizeNavigationUrl(tabId, url);
    final nextTabs = _upsertTab(
      tabId,
      (tab) => tab.copyWith(
        url: normalizedUrl.isEmpty ? tab.url : normalizedUrl,
        isActive: true,
        isLoading: false,
      ),
    );

    state = state.copyWith(
      tabs: _markOnlyActive(nextTabs, tabId),
      activeTabId: _resolveActiveTabId(nextTabs, tabId),
      isLoading: false,
      errorMessage: success
          ? null
          : _buildNavigationErrorMessage(normalizedUrl),
      clearErrorMessage: success,
    );
  }

  void _handleBlockedRequest() {
    state = state.copyWith(
      blockedRequestCount: state.blockedRequestCount + 1,
    );
  }

  void _setCommandLoading(String tabId) {
    final nextTabs = _upsertTab(
      tabId,
      (tab) => tab.copyWith(
        isActive: true,
        isLoading: true,
      ),
    );

    state = state.copyWith(
      tabs: _markOnlyActive(nextTabs, tabId),
      activeTabId: _resolveActiveTabId(nextTabs, tabId),
      isLoading: true,
      clearErrorMessage: true,
    );
  }

  void _activateTab(String tabId) {
    final shouldClearError = state.activeTabId != tabId;
    final nextTabs = _upsertTab(
      tabId,
      (tab) => tab.copyWith(isActive: true),
    );

    state = state.copyWith(
      tabs: _markOnlyActive(nextTabs, tabId),
      activeTabId: _resolveActiveTabId(nextTabs, tabId),
      clearErrorMessage: shouldClearError,
    );
  }

  void _markTabLoading(String? tabId, {String? url}) {
    if (tabId == null || tabId.isEmpty) {
      return;
    }

    final nextTabs = _upsertTab(
      tabId,
      (tab) => tab.copyWith(
        url: url != null && url.isNotEmpty ? url : tab.url,
        isActive: true,
        isLoading: true,
      ),
    );

    state = state.copyWith(
      tabs: _markOnlyActive(nextTabs, tabId),
      activeTabId: _resolveActiveTabId(nextTabs, tabId),
      isLoading: true,
      clearErrorMessage: true,
    );
  }

  void _updateTab(String? tabId, TabState Function(TabState tab) transform) {
    if (tabId == null || tabId.isEmpty) {
      return;
    }

    final nextTabs = _upsertTab(tabId, transform);
    state = state.copyWith(
      tabs: nextTabs,
      activeTabId: _resolveActiveTabId(nextTabs, state.activeTabId),
    );
  }

  void _setError(Object error) {
    state = state.copyWith(
      isLoading: false,
      errorMessage: error.toString(),
    );
  }

  List<TabState> _upsertTab(
    String tabId,
    TabState Function(TabState tab) transform,
  ) {
    final tabs = List<TabState>.from(state.tabs);
    final index = tabs.indexWhere((tab) => tab.id == tabId);

    if (index >= 0) {
      tabs[index] = transform(tabs[index]);
      return tabs;
    }

    tabs.add(
      transform(
        TabState(
          id: tabId,
          title: '',
          url: '',
          isActive: false,
          isLoading: false,
          canGoBack: false,
          canGoForward: false,
          isSuspended: false,
        ),
      ),
    );
    return tabs;
  }

  List<TabState> _markOnlyActive(List<TabState> tabs, String activeTabId) {
    return tabs
        .map((tab) => tab.copyWith(isActive: tab.id == activeTabId))
        .toList();
  }

  String _normalizeNavigationUrl(String tabId, String url) {
    if (url.isNotEmpty) {
      return url;
    }

    return state.tabs.where((tab) => tab.id == tabId).firstOrNull?.url ?? '';
  }

  String _buildNavigationErrorMessage(String url) {
    if (url.isEmpty) {
      return 'Network error while loading the requested page.';
    }

    return 'Network error while loading $url';
  }

  String? _resolveActiveTabId(List<TabState> tabs, String? activeTabId) {
    if (tabs.isEmpty) {
      return null;
    }

    if (activeTabId != null && tabs.any((tab) => tab.id == activeTabId)) {
      return activeTabId;
    }

    return tabs.first.id;
  }
}

extension on Iterable<TabState> {
  /// Returns the first item in the iterable or `null` when it is empty.
  TabState? get firstOrNull {
    final iterator = this.iterator;
    return iterator.moveNext() ? iterator.current : null;
  }
}
