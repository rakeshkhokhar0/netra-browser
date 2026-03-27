import 'package:netra_browser/netra/core/entities/engine_event.dart';

import 'i_frame.dart';

/// Defines the platform-agnostic browser engine contract used by the shell.
abstract class IEngine {
  /// Initializes the engine once during startup.
  Future<void> initialize();

  /// Creates a new browser frame managed by the engine.
  Future<IFrame> createFrame();

  /// Destroys an existing frame by identifier.
  Future<void> destroyFrame(String frameId);

  /// Emits global engine lifecycle and diagnostic events.
  Stream<EngineEvent> get engineEvents;

  /// Disposes the engine and all remaining resources.
  Future<void> dispose();
}
