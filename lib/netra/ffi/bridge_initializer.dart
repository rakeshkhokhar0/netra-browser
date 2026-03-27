import 'bridge.dart';

/// Bootstraps the Rust bridge for the Dart shell.
class BridgeInitializer {
  static Future<void>? _initialization;

  /// Initializes the Rust core bridge once for the current process.
  static Future<void> init() {
    return _initialization ??= initEngine();
  }
}
