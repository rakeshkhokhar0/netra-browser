import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../engine/adapters/webview2/webview2_adapter.dart';
import '../../engine/interfaces/i_frame.dart';
import '../../engine/models/browser_event.dart';
import '../../di/modules/engine_module.dart';

/// Describes the lightweight UI state for the currently active browser tab.
///
/// This immutable state object keeps only the presentation-facing values needed
/// by the current shell, such as the active tab identifier, visible URL, and
/// simple navigation capability flags. It does not contain browser business
/// logic.
class BrowserTabState {
  /// Creates an immutable browser tab UI state snapshot.
  const BrowserTabState({
    required this.tabId,
    this.url = '',
    this.title = '',
    this.canGoBack = false,
    this.canGoForward = false,
    this.isLoading = false,
  });

  /// Active native tab identifier.
  final String tabId;

  /// Current visible URL for the active tab.
  final String url;

  /// Current visible title for the active tab.
  final String title;

  /// Whether backward navigation is currently available.
  final bool canGoBack;

  /// Whether forward navigation is currently available.
  final bool canGoForward;

  /// Whether the tab is currently loading content.
  final bool isLoading;

  /// Creates a new state snapshot with selected fields replaced.
  BrowserTabState copyWith({
    String? tabId,
    String? url,
    String? title,
    bool? canGoBack,
    bool? canGoForward,
    bool? isLoading,
  }) {
    return BrowserTabState(
      tabId: tabId ?? this.tabId,
      url: url ?? this.url,
      title: title ?? this.title,
      canGoBack: canGoBack ?? this.canGoBack,
      canGoForward: canGoForward ?? this.canGoForward,
      isLoading: isLoading ?? this.isLoading,
    );
  }
}

/// Builds and maintains the UI-facing state for the active browser tab.
///
/// The notifier performs only shell-level coordination: it creates one native
/// frame, subscribes to typed browser events for that frame, and mirrors the
/// latest presentation state needed by the toolbar and address bar.
class ActiveTabController extends AsyncNotifier<BrowserTabState> {
  StreamSubscription<BrowserEvent>? _eventSubscription;
  IFrame? _frame;

  @override
  Future<BrowserTabState> build() async {
    final adapter = ref.read(engineProvider) as WebView2Adapter;
    await adapter.initialize();

    final frame = await adapter.createFrame();
    _frame = frame;

    _eventSubscription = adapter.events
        .where((event) => event.tabId == frame.id)
        .listen(_handleBrowserEvent);

    ref.onDispose(() {
      _eventSubscription?.cancel();
      final currentFrame = _frame;
      if (currentFrame != null) {
        unawaited(currentFrame.dispose());
      }
    });

    return BrowserTabState(tabId: frame.id);
  }

  /// Exposes the active native frame created for the shell, when available.
  IFrame? get frame => _frame;

  /// Applies a typed browser event to the current tab UI state snapshot.
  void _handleBrowserEvent(BrowserEvent event) {
    final currentState = state.valueOrNull;
    if (currentState == null) {
      return;
    }

    switch (event) {
      case NavigationStartingBrowserEvent():
        state = AsyncData(
          currentState.copyWith(
            url: event.url.isEmpty ? currentState.url : event.url,
            isLoading: true,
          ),
        );
      case ContentLoadingBrowserEvent():
        state = AsyncData(currentState.copyWith(isLoading: true));
      case UrlChangedBrowserEvent():
        state = AsyncData(
          currentState.copyWith(
            url: event.url.isEmpty ? currentState.url : event.url,
          ),
        );
      case NavigationCompletedBrowserEvent():
        state = AsyncData(
          currentState.copyWith(
            url: event.url.isEmpty ? currentState.url : event.url,
            isLoading: false,
          ),
        );
      case TitleChangedBrowserEvent():
        state = AsyncData(currentState.copyWith(title: event.title));
      case HistoryChangedBrowserEvent():
        state = AsyncData(
          currentState.copyWith(
            canGoBack: event.canGoBack,
            canGoForward: event.canGoForward,
          ),
        );
      case TabCrashedBrowserEvent():
        state = AsyncData(currentState.copyWith(isLoading: false));
      default:
        break;
    }
  }
}

/// Exposes the active browser tab state used by the shell UI.
final activeTabStateProvider =
    AsyncNotifierProvider<ActiveTabController, BrowserTabState>(
      ActiveTabController.new,
    );

/// Creates and exposes the primary browser frame used by the current shell.
///
/// This provider gives lower layers access to the current native frame without
/// duplicating frame creation work.
final activeFrameProvider = FutureProvider<IFrame>((ref) async {
  await ref.watch(activeTabStateProvider.future);

  final frame = ref.read(activeTabStateProvider.notifier).frame;
  if (frame == null) {
    throw StateError('Active frame is not available.');
  }

  return frame;
});

/// Exposes the active tab identifier derived from the created native frame.
///
/// UI components use this provider to target toolbar and address-bar commands
/// at the actual native tab created for the shell.
final activeTabIdProvider = FutureProvider<String>((ref) async {
  final tabState = await ref.watch(activeTabStateProvider.future);
  return tabState.tabId;
});
