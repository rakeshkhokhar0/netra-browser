/// Defines the shared exception surface used by the Dart shell.
class NetraException implements Exception {
  /// Creates a shell-side exception with a human-readable message.
  const NetraException(this.message);

  /// Human-readable exception message.
  final String message;

  @override
  String toString() => 'NetraException: $message';
}
