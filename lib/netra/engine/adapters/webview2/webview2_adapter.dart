import 'package:uuid/uuid.dart';
import 'package:netra_browser/netra/core/entities/engine_event.dart';
import '../../interfaces/i_engine.dart';
import '../../interfaces/i_frame.dart';
import '../../interfaces/i_request_handler.dart';
import '../../models/browser_event.dart';
import '../../../infrastructure/bridge/event_channel_listener.dart';
import '../../../infrastructure/bridge/method_channel_bridge.dart';

/// Implements the shell-side browser engine contract using the native WebView2
/// bridge.
///
/// This adapter remains a thin translation layer between the platform-agnostic
/// engine interfaces and the concrete MethodChannel and EventChannel bridge
/// adapters. It owns command forwarding and event mapping only. It does not
/// contain browser business logic, tab orchestration policy, UI code, or any
/// direct Rust interaction.
class WebView2Adapter implements IEngine {
  /// Creates a stateless WebView2 engine adapter.
  const WebView2Adapter();

  /// Shared typed event stream derived from the native EventChannel bridge.
  ///
  /// Raw native maps are converted into typed [BrowserEvent] instances so the
  /// engine layer can subscribe to structured browser events without depending
  /// on infrastructure payload formats.
  static final Stream<BrowserEvent> _events = EventChannelListener.events
      .map(BrowserEvent.fromMap)
      .asBroadcastStream();

  /// Shared frame-state cache derived from typed browser events.
  ///
  /// The cache keeps the latest URL and title snapshot for each frame so the
  /// frame contract can answer lightweight query methods without requiring
  /// additional synchronous native bridge calls.
  static final _WebView2FrameStateCache _frameStateCache =
      _WebView2FrameStateCache(_events);

  /// Initializes the adapter boundary during app startup.
  ///
  /// The WebView2 engine is initialized by the native runner during Windows
  /// startup, so the Dart-side adapter does not need to perform extra work
  /// here. The method remains asynchronous to satisfy the engine contract.
  @override
  Future<void> initialize() async {}

  /// Creates a new frame backed by a native WebView2 tab.
  ///
  /// A transport-safe frame identifier is generated in Dart, forwarded to the
  /// native MethodChannel bridge, and then wrapped in an [IFrame]
  /// implementation that delegates future commands back through the same
  /// bridge layer.
  @override
  Future<IFrame> createFrame() async {
    final frameId = _generateFrameId();
    await createTab(frameId);
    return _WebView2FrameAdapter(frameId);
  }

  /// Destroys the specified native WebView2 frame.
  ///
  /// The [frameId] is forwarded directly to the native `closeTab` command.
  @override
  Future<void> destroyFrame(String frameId) {
    return closeTab(frameId);
  }

  /// Creates a native browser tab for the provided [tabId].
  ///
  /// The adapter ensures local frame metadata exists before forwarding the
  /// create-tab command to the native bridge.
  Future<void> createTab(String tabId) {
    _frameStateCache.ensureFrame(tabId);
    return MethodChannelBridge.createTab(tabId);
  }

  /// Closes the native browser tab associated with [tabId].
  ///
  /// The adapter removes any cached shell-side metadata before forwarding the
  /// close command to the native bridge.
  Future<void> closeTab(String tabId) {
    _frameStateCache.removeFrame(tabId);
    return MethodChannelBridge.closeTab(tabId);
  }

  /// Navigates the specified native tab to the requested URL.
  ///
  /// This helper gives upper layers a direct tab-addressed command surface
  /// while still routing through the adapter rather than calling the
  /// infrastructure bridge directly.
  Future<void> navigate(String tabId, String url) {
    return MethodChannelBridge.navigate(tabId, url);
  }

  /// Requests backward navigation for the specified native tab.
  ///
  /// The adapter forwards the command without storing state or interpreting
  /// the result.
  Future<void> goBack(String tabId) {
    return MethodChannelBridge.goBack(tabId);
  }

  /// Requests forward navigation for the specified native tab.
  ///
  /// The adapter forwards the command without storing state or interpreting
  /// the result.
  Future<void> goForward(String tabId) {
    return MethodChannelBridge.goForward(tabId);
  }

  /// Requests a reload of the current document for the specified native tab.
  ///
  /// The adapter forwards the command without storing state or interpreting
  /// the result.
  Future<void> reload(String tabId) {
    return MethodChannelBridge.reload(tabId);
  }

  /// Marks the specified native tab as the active visible browser surface.
  ///
  /// This forwards only the active-tab selection command required by the
  /// native bridge and does not store tab state inside the adapter.
  Future<void> setActiveTab(String tabId) {
    return MethodChannelBridge.setActiveTab(tabId);
  }

  /// Updates the native bounds used to place the tab surface in the app.
  ///
  /// The supplied geometry is forwarded directly to the MethodChannel bridge so
  /// the native WebView host can align itself with the Flutter layout region.
  Future<void> setBounds(
    String tabId,
    double x,
    double y,
    double width,
    double height,
  ) {
    return MethodChannelBridge.setBounds(tabId, x, y, width, height);
  }

  /// Exposes the typed browser event stream as the engine event stream.
  ///
  /// Each event originates from the native EventChannel bridge and is mapped
  /// into a typed [BrowserEvent], which remains compatible with the
  /// [EngineEvent] contract.
  @override
  Stream<EngineEvent> get engineEvents => _events;

  /// Exposes the typed browser event stream for adapter consumers that need
  /// the richer event model.
  Stream<BrowserEvent> get events => _events;

  /// Releases adapter-owned resources.
  ///
  /// The adapter is stateless and owns no disposable Dart resources beyond the
  /// shared bridge streams, so disposal completes immediately.
  @override
  Future<void> dispose() async {}

  /// Generates a transport-safe frame identifier for native tab creation.
  static String _generateFrameId() {
    return const Uuid().v4();
  }
}

/// Implements a single WebView2-backed frame using the native command bridge.
///
/// The frame holds only its immutable identifier and forwards browser commands
/// to the native layer through [MethodChannelBridge]. It does not store page
/// state, parse responses, or perform UI work.
class _WebView2FrameAdapter implements IFrame {
  /// Creates a frame wrapper for an existing native WebView2 tab.
  _WebView2FrameAdapter(this.id);

  /// Stable identifier used by the native bridge to address the tab.
  @override
  final String id;

  /// Emits only the events associated with this frame identifier.
  ///
  /// The underlying event stream comes from the shared adapter-level
  /// EventChannel mapping and is filtered to the current frame.
  @override
  Stream<EngineEvent> get events => WebView2Adapter._events
      .where((event) => event.tabId == id)
      .asBroadcastStream();

  /// Loads the provided URL inside the native WebView2 tab.
  @override
  Future<void> navigate(String url) {
    return MethodChannelBridge.navigate(id, url);
  }

  /// Reloads the current page inside the native WebView2 tab.
  @override
  Future<void> reload() {
    return MethodChannelBridge.reload(id);
  }

  /// Requests backward navigation for the native tab.
  @override
  Future<void> goBack() {
    return MethodChannelBridge.goBack(id);
  }

  /// Requests forward navigation for the native tab.
  @override
  Future<void> goForward() {
    return MethodChannelBridge.goForward(id);
  }

  /// Stops the current loading operation for the native tab.
  @override
  Future<void> stopLoading() {
    return MethodChannelBridge.stopLoading(id);
  }

  /// Returns the current visible URL for the frame.
  @override
  Future<String> getUrl() {
    return Future<String>.value(
      WebView2Adapter._frameStateCache.urlFor(id) ?? '',
    );
  }

  /// Returns the current document title for the frame when available.
  @override
  Future<String?> getTitle() {
    return Future<String?>.value(WebView2Adapter._frameStateCache.titleFor(id));
  }

  /// Executes JavaScript inside the frame.
  ///
  /// The current native MethodChannel bridge does not yet expose a matching
  /// execute-script command, so this remains unsupported until that transport
  /// boundary is implemented.
  @override
  Future<String> executeScript(String script) {
    return Future<String>.error(
      UnsupportedError(
        'executeScript is not yet exposed by MethodChannelBridge.',
      ),
    );
  }

  /// Records the request handler associated with this frame.
  ///
  /// The shell keeps the assigned handler reference with the frame snapshot so
  /// future native request-pipeline integration can resolve the contract
  /// without requiring UI-layer changes.
  @override
  Future<void> setRequestHandler(IRequestHandler handler) async {
    WebView2Adapter._frameStateCache.setRequestHandler(id, handler);
  }

  /// Releases the native tab associated with this frame.
  @override
  Future<void> dispose() {
    WebView2Adapter._frameStateCache.removeFrame(id);
    return MethodChannelBridge.closeTab(id);
  }
}

/// Maintains the latest typed state snapshot for native WebView2 frames.
///
/// The cache listens to the shared typed browser event stream and records only
/// the lightweight frame metadata required by the shell contract, such as URL,
/// title, and the optional shell-side request handler reference.
class _WebView2FrameStateCache {
  /// Starts caching frame state from the shared typed browser event stream.
  _WebView2FrameStateCache(Stream<BrowserEvent> events) {
    events.listen(_applyEvent);
  }

  final Map<String, _WebView2FrameState> _states =
      <String, _WebView2FrameState>{};

  /// Ensures a state bucket exists for the provided frame identifier.
  void ensureFrame(String frameId) {
    _states.putIfAbsent(frameId, _WebView2FrameState.new);
  }

  /// Removes the cached state for a frame that is no longer active.
  void removeFrame(String frameId) {
    _states.remove(frameId);
  }

  /// Records the request handler assigned to the given frame.
  void setRequestHandler(String frameId, IRequestHandler handler) {
    final state = _states.putIfAbsent(frameId, _WebView2FrameState.new);
    state.requestHandler = handler;
  }

  /// Returns the latest known URL for the provided frame.
  String? urlFor(String frameId) => _states[frameId]?.url;

  /// Returns the latest known title for the provided frame.
  String? titleFor(String frameId) => _states[frameId]?.title;

  /// Applies a typed browser event to the cached frame snapshot.
  void _applyEvent(BrowserEvent event) {
    final tabId = event.tabId;
    if (tabId == null || tabId.isEmpty) {
      return;
    }

    final state = _states.putIfAbsent(tabId, _WebView2FrameState.new);

    switch (event) {
      case NavigationStartingBrowserEvent():
        if (event.url.isNotEmpty) {
          state.url = event.url;
        }
      case UrlChangedBrowserEvent():
        if (event.url.isNotEmpty) {
          state.url = event.url;
        }
      case NavigationCompletedBrowserEvent():
        if (event.url.isNotEmpty) {
          state.url = event.url;
        }
      case TitleChangedBrowserEvent():
        state.title = event.title;
      default:
        break;
    }
  }
}

/// Stores the latest shell-visible metadata for a single frame.
class _WebView2FrameState {
  String? url;
  String? title;
  IRequestHandler? requestHandler;
}
