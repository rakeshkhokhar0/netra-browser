import 'dart:async';

import 'package:netra_browser/netra/core/entities/engine_event.dart';
import '../../interfaces/i_engine.dart';
import '../../interfaces/i_frame.dart';
import '../../models/browser_event.dart';
import '../../../ffi/bridge.dart';

/// Implements the shell-side browser engine contract using the native WebView2
/// bridge.
///
/// This adapter remains a thin translation layer between the platform-agnostic
/// engine interfaces and the Rust-driven native event stream. It owns only
/// event translation. It does not execute browser control commands, contain
/// browser business logic, or store browser state.
class WebView2Adapter implements IEngine {
  /// Creates a stateless WebView2 engine adapter.
  const WebView2Adapter();

  /// Shared controller that owns the typed browser-event pump for the process.
  ///
  /// The adapter keeps a single subscription to the underlying Rust bridge and
  /// re-exposes typed [BrowserEvent] instances through this controller. This
  /// avoids making provider lifecycle depend on a transient mapped-stream
  /// subscription chain.
  static final StreamController<BrowserEvent> _eventController =
      StreamController<BrowserEvent>.broadcast();
  static StreamSubscription<Map<String, dynamic>>? _bridgeSubscription;

  /// Shared typed event stream derived from the stable adapter-owned pump.
  static final Stream<BrowserEvent> _events = _eventController.stream;

  /// Initializes the adapter boundary during app startup.
  ///
  /// The WebView2 engine is initialized by the native runner during Windows
  /// startup, so the Dart-side adapter does not need to perform extra work
  /// here. The method remains asynchronous to satisfy the engine contract.
  @override
  Future<void> initialize() async {
    _ensureEventPump();
  }

  /// Creates a new frame wrapper.
  ///
  /// Direct frame control no longer belongs to the Flutter adapter layer. All
  /// browser control is routed through Rust, so requesting frames from this
  /// adapter is unsupported.
  @override
  Future<IFrame> createFrame() async {
    throw UnsupportedError(
      'WebView2Adapter no longer creates control frames. Use Rust control surfaces instead.',
    );
  }

  /// Destroys the specified frame.
  ///
  /// Direct frame destruction is no longer handled by the Flutter adapter
  /// layer. All browser control is routed through Rust.
  @override
  Future<void> destroyFrame(String frameId) async {
    throw UnsupportedError(
      'WebView2Adapter no longer destroys control frames. Use Rust control surfaces instead.',
    );
  }

  /// Exposes the typed browser event stream as the engine event stream.
  ///
  /// Each event originates from the Rust FFI event dispatcher and is mapped
  /// into a typed [BrowserEvent], which remains compatible with the
  /// [EngineEvent] contract.
  @override
  Stream<EngineEvent> get engineEvents {
    _ensureEventPump();
    return _events;
  }

  /// Exposes the typed browser event stream for adapter consumers that need
  /// the richer event model.
  Stream<BrowserEvent> get events {
    _ensureEventPump();
    return _events;
  }

  /// Releases adapter-owned resources.
  ///
  /// The adapter is stateless and owns no disposable Dart resources beyond the
  /// shared bridge streams, so disposal completes immediately.
  @override
  Future<void> dispose() async {}

  static void _ensureEventPump() {
    if (_bridgeSubscription != null) {
      return;
    }

    _bridgeSubscription = RustBridge.instance.browserEvents.listen(
      (event) {
        _eventController.add(BrowserEvent.fromMap(event));
      },
      onError: _eventController.addError,
      onDone: () {
        _bridgeSubscription = null;
      },
    );
  }
}
