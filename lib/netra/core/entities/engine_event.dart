/// Describes an engine-level event flowing through the Dart shell.
///
/// This is a lightweight transport contract owned by the shell-side engine
/// interfaces so adapters and bridge layers can exchange event metadata
/// without depending on Flutter UI code.
class EngineEvent {
  /// Creates an immutable engine event payload.
  const EngineEvent({
    required this.type,
    this.tabId,
    this.message,
  });

  /// Stable event type identifier such as `engine_ready` or `tab_crashed`.
  final String type;

  /// Optional tab identifier associated with the event.
  final String? tabId;

  /// Optional human-readable message attached to the event.
  final String? message;
}
