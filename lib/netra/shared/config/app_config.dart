/// Holds top-level shell configuration values used across the Flutter app.
abstract final class AppConfig {
  /// Display name used by the restored bridge shell.
  static const String appName = 'Netra Browser';

  /// MethodChannel name used for Dart-to-native browser commands.
  ///
  /// The value is shared from configuration so infrastructure adapters and any
  /// future engine-facing callers use one stable channel identifier.
  static const String browserMethodChannelName = 'netra/browser/methods';

  /// EventChannel name used for native-to-Dart browser events.
  ///
  /// The value is shared from configuration so infrastructure adapters and any
  /// future engine-facing callers use one stable event identifier.
  static const String browserEventChannelName = 'netra/browser/events';
}
