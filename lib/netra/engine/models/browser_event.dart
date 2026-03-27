import 'package:netra_browser/netra/core/entities/browser_event.dart';

// Defines the typed browser events produced by the WebView2 engine adapter.
//
// These event classes provide a structured Dart representation of the raw
// native payloads emitted by the C++ EventChannel bridge. They remain pure
// transport models and do not contain browser logic, state mutation, or UI
// concerns.
abstract class BrowserEvent extends EngineEvent {
  // Creates a typed browser event with the shared [EngineEvent] fields.
  const BrowserEvent({
    required super.type,
    required super.tabId,
    super.message,
  });

  // Creates a typed browser event from a raw native event payload.
  //
  // The input [event] map is expected to match the basic shape emitted by the
  // native EventChannel bridge. Unknown or partially populated payloads are
  // converted into [GenericBrowserEvent] so the adapter remains resilient
  // without interpreting event semantics beyond simple mapping.
  factory BrowserEvent.fromMap(Map<String, dynamic> event) {
    final type =
        (event['type'] as String?) ?? (event['event'] as String?) ?? 'unknown';
    final tabId =
        (event['tabId'] as String?) ?? (event['tab_id'] as String?) ?? '';

    switch (type) {
      case 'navigationStarting':
        return NavigationStartingBrowserEvent(
          tabId: tabId,
          url: (event['url'] as String?) ?? '',
        );
      case 'contentLoading':
        return ContentLoadingBrowserEvent(tabId: tabId);
      case 'urlChanged':
        return UrlChangedBrowserEvent(
          tabId: tabId,
          url: (event['url'] as String?) ?? '',
        );
      case 'navigationCompleted':
        return NavigationCompletedBrowserEvent(
          tabId: tabId,
          url: (event['url'] as String?) ?? '',
          success: (event['success'] as bool?) ?? false,
        );
      case 'titleChanged':
        return TitleChangedBrowserEvent(
          tabId: tabId,
          title: (event['title'] as String?) ?? '',
        );
      case 'faviconChanged':
        return FaviconChangedBrowserEvent(
          tabId: tabId,
          faviconUrl:
              (event['faviconUrl'] as String?) ??
              (event['favicon_url'] as String?) ??
              '',
        );
      case 'historyChanged':
        return HistoryChangedBrowserEvent(
          tabId: tabId,
          canGoBack:
              (event['canGoBack'] as bool?) ??
              (event['can_go_back'] as bool?) ??
              false,
          canGoForward:
              (event['canGoForward'] as bool?) ??
              (event['can_go_forward'] as bool?) ??
              false,
        );
      case 'requestBlocked':
        return RequestBlockedBrowserEvent(
          tabId: tabId,
          url: (event['url'] as String?) ?? '',
        );
      case 'tabCrashed':
        return TabCrashedBrowserEvent(tabId: tabId);
      case 'engineReady':
        return EngineReadyBrowserEvent(tabId: tabId);
      default:
        return GenericBrowserEvent(
          type: type,
          tabId: tabId,
          message: event['message'] as String?,
        );
    }
  }
}

// Emitted when navigation is about to begin for a tab.
class NavigationStartingBrowserEvent extends BrowserEvent {
  // Creates a typed navigation-start event.
  const NavigationStartingBrowserEvent({
    required String tabId,
    required this.url,
  }) : super(type: 'navigationStarting', tabId: tabId);

  // Target URL that the tab is beginning to load.
  final String url;
}

// Emitted when content loading begins for a tab.
class ContentLoadingBrowserEvent extends BrowserEvent {
  // Creates a typed content-loading event.
  const ContentLoadingBrowserEvent({required String tabId})
    : super(type: 'contentLoading', tabId: tabId);
}

// Emitted when the visible URL changes for a tab.
class UrlChangedBrowserEvent extends BrowserEvent {
  // Creates a typed URL-changed event.
  const UrlChangedBrowserEvent({required String tabId, required this.url})
    : super(type: 'urlChanged', tabId: tabId);

  // Latest URL reported by the native engine.
  final String url;
}

// Emitted when navigation finishes for a tab.
class NavigationCompletedBrowserEvent extends BrowserEvent {
  // Creates a typed navigation-completed event.
  const NavigationCompletedBrowserEvent({
    required String tabId,
    required this.url,
    required this.success,
  }) : super(type: 'navigationCompleted', tabId: tabId);

  // Final URL reported by the native engine.
  final String url;

  // Indicates whether the navigation completed successfully.
  final bool success;
}

// Emitted when the current document title changes for a tab.
class TitleChangedBrowserEvent extends BrowserEvent {
  // Creates a typed title-changed event.
  const TitleChangedBrowserEvent({required String tabId, required this.title})
    : super(type: 'titleChanged', tabId: tabId);

  // Latest document title reported by the native engine.
  final String title;
}

// Emitted when the favicon changes for a tab.
class FaviconChangedBrowserEvent extends BrowserEvent {
  // Creates a typed favicon-changed event.
  const FaviconChangedBrowserEvent({
    required String tabId,
    required this.faviconUrl,
  }) : super(type: 'faviconChanged', tabId: tabId);

  // URL of the favicon associated with the current page.
  final String faviconUrl;
}

// Emitted when browser history capabilities change for a tab.
class HistoryChangedBrowserEvent extends BrowserEvent {
  // Creates a typed history-change event.
  const HistoryChangedBrowserEvent({
    required String tabId,
    required this.canGoBack,
    required this.canGoForward,
  }) : super(type: 'historyChanged', tabId: tabId);

  // Indicates whether backward navigation is currently available.
  final bool canGoBack;

  // Indicates whether forward navigation is currently available.
  final bool canGoForward;
}

// Emitted when a resource request is blocked for a tab.
class RequestBlockedBrowserEvent extends BrowserEvent {
  // Creates a typed request-blocked event.
  const RequestBlockedBrowserEvent({required String tabId, required this.url})
    : super(type: 'requestBlocked', tabId: tabId);

  // Blocked resource URL reported by the native bridge.
  final String url;
}

// Emitted when the native engine reports a tab crash.
class TabCrashedBrowserEvent extends BrowserEvent {
  // Creates a typed tab-crashed event.
  const TabCrashedBrowserEvent({required String tabId})
    : super(type: 'tabCrashed', tabId: tabId);
}

// Emitted when the native WebView2 environment becomes ready.
class EngineReadyBrowserEvent extends BrowserEvent {
  // Creates a typed engine-ready event.
  const EngineReadyBrowserEvent({required String tabId})
    : super(type: 'engineReady', tabId: tabId);
}

// Fallback event used when the native payload type is unknown.
class GenericBrowserEvent extends BrowserEvent {
  // Creates a fallback browser event for unmapped native payloads.
  const GenericBrowserEvent({
    required super.type,
    required super.tabId,
    super.message,
  });
}
