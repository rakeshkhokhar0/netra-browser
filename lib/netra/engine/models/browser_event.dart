import 'package:netra_browser/netra/core/entities/browser_event.dart';

// Defines the typed browser events produced by the WebView2 engine adapter.
//
// These event classes provide a structured Dart representation of the raw
// native payloads emitted through the Rust FFI bridge. They remain pure
// transport models and do not contain browser logic, state mutation, or UI
// concerns.
abstract class BrowserEvent extends EngineEvent {
  // Creates a typed browser event with the shared [EngineEvent] fields.
  const BrowserEvent({
    required super.type,
    required super.tabId,
    this.sequenceNumber = 0,
    this.tabState,
    super.message,
  });

  final int sequenceNumber;
  final Map<String, dynamic>? tabState;

  // Creates a typed browser event from a raw native event payload.
  //
  // The input [event] map is expected to match the basic shape emitted by the
  // Rust FFI bridge. Unknown or partially populated payloads are converted
  // into [GenericBrowserEvent] so the adapter remains resilient without
  // interpreting event semantics beyond simple mapping.
  factory BrowserEvent.fromMap(Map<String, dynamic> event) {
    final type =
        (event['type'] as String?) ?? (event['event'] as String?) ?? 'unknown';
    final tabId =
        (event['tabId'] as String?) ?? (event['tab_id'] as String?) ?? '';
    final sequenceNumber = (event['sequenceNumber'] as int?) ?? 0;
    final tabState = event['tabState'] is Map
        ? Map<String, dynamic>.from(event['tabState'] as Map)
        : null;

    switch (type) {
      case 'navigationStarting':
        return NavigationStartingBrowserEvent(
          tabId: tabId,
          url: (tabState?['url'] as String?) ?? (event['url'] as String?) ?? '',
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'contentLoading':
        return ContentLoadingBrowserEvent(
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'urlChanged':
        return UrlChangedBrowserEvent(
          tabId: tabId,
          url: (tabState?['url'] as String?) ?? (event['url'] as String?) ?? '',
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'navigationCompleted':
        return NavigationCompletedBrowserEvent(
          tabId: tabId,
          url: (tabState?['url'] as String?) ?? (event['url'] as String?) ?? '',
          success: (event['success'] as bool?) ?? false,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'titleChanged':
        return TitleChangedBrowserEvent(
          tabId: tabId,
          title:
              (tabState?['title'] as String?) ?? (event['title'] as String?) ?? '',
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'faviconChanged':
        return FaviconChangedBrowserEvent(
          tabId: tabId,
          faviconUrl:
              (event['faviconUrl'] as String?) ??
              (event['favicon_url'] as String?) ??
              '',
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'historyChanged':
        return HistoryChangedBrowserEvent(
          tabId: tabId,
          canGoBack:
              (tabState?['canGoBack'] as bool?) ??
              (event['canGoBack'] as bool?) ??
              (event['can_go_back'] as bool?) ??
              false,
          canGoForward:
              (tabState?['canGoForward'] as bool?) ??
              (event['canGoForward'] as bool?) ??
              (event['can_go_forward'] as bool?) ??
              false,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'requestBlocked':
        return RequestBlockedBrowserEvent(
          tabId: tabId,
          url: (tabState?['url'] as String?) ?? (event['url'] as String?) ?? '',
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'navigationFailed':
        return NavigationFailedBrowserEvent(
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'tabCrashed':
        return TabCrashedBrowserEvent(
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'frameCreated':
        return FrameCreatedBrowserEvent(
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'frameDestroyed':
        return FrameDestroyedBrowserEvent(
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'loadFinished':
        return LoadFinishedBrowserEvent(
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      case 'engineReady':
        return EngineReadyBrowserEvent(
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
        );
      default:
        return GenericBrowserEvent(
          type: type,
          tabId: tabId,
          sequenceNumber: sequenceNumber,
          tabState: tabState,
          message: event['message'] as String?,
        );
    }
  }
}

// Emitted when navigation is about to begin for a tab.
class NavigationStartingBrowserEvent extends BrowserEvent {
  const NavigationStartingBrowserEvent({
    required String tabId,
    required this.url,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'navigationStarting',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );

  final String url;
}

class ContentLoadingBrowserEvent extends BrowserEvent {
  const ContentLoadingBrowserEvent({
    required String tabId,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'contentLoading',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );
}

class UrlChangedBrowserEvent extends BrowserEvent {
  const UrlChangedBrowserEvent({
    required String tabId,
    required this.url,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'urlChanged',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );

  final String url;
}

class NavigationCompletedBrowserEvent extends BrowserEvent {
  const NavigationCompletedBrowserEvent({
    required String tabId,
    required this.url,
    required this.success,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'navigationCompleted',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );

  final String url;
  final bool success;
}

class TitleChangedBrowserEvent extends BrowserEvent {
  const TitleChangedBrowserEvent({
    required String tabId,
    required this.title,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'titleChanged',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );

  final String title;
}

class FaviconChangedBrowserEvent extends BrowserEvent {
  const FaviconChangedBrowserEvent({
    required String tabId,
    required this.faviconUrl,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'faviconChanged',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );

  final String faviconUrl;
}

class HistoryChangedBrowserEvent extends BrowserEvent {
  const HistoryChangedBrowserEvent({
    required String tabId,
    required this.canGoBack,
    required this.canGoForward,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'historyChanged',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );

  final bool canGoBack;
  final bool canGoForward;
}

class RequestBlockedBrowserEvent extends BrowserEvent {
  const RequestBlockedBrowserEvent({
    required String tabId,
    required this.url,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'requestBlocked',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );

  final String url;
}

class NavigationFailedBrowserEvent extends BrowserEvent {
  const NavigationFailedBrowserEvent({
    required String tabId,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'navigationFailed',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );
}

class TabCrashedBrowserEvent extends BrowserEvent {
  const TabCrashedBrowserEvent({
    required String tabId,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'tabCrashed',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );
}

class FrameCreatedBrowserEvent extends BrowserEvent {
  const FrameCreatedBrowserEvent({
    required String tabId,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'frameCreated',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );
}

class FrameDestroyedBrowserEvent extends BrowserEvent {
  const FrameDestroyedBrowserEvent({
    required String tabId,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'frameDestroyed',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );
}

class LoadFinishedBrowserEvent extends BrowserEvent {
  const LoadFinishedBrowserEvent({
    required String tabId,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'loadFinished',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );
}

class EngineReadyBrowserEvent extends BrowserEvent {
  const EngineReadyBrowserEvent({
    required String tabId,
    int sequenceNumber = 0,
    Map<String, dynamic>? tabState,
  }) : super(
         type: 'engineReady',
         tabId: tabId,
         sequenceNumber: sequenceNumber,
         tabState: tabState,
       );
}

class GenericBrowserEvent extends BrowserEvent {
  const GenericBrowserEvent({
    required super.type,
    required super.tabId,
    super.sequenceNumber = 0,
    super.tabState,
    super.message,
  });
}
