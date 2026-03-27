import 'dart:async';
import 'dart:ffi';
import 'dart:io';

import 'package:netra_browser/netra/core/entities/engine_event.dart';

/// Immutable bridge-verification payload returned by the Rust FFI boundary.
class RustConnectionResult {
  const RustConnectionResult({required this.code, required this.message});

  final int code;
  final String message;
}

typedef _SmokeTestNative = Int32 Function();
typedef _SmokeTestDart = int Function();
typedef _MessageNative = Pointer<Uint8> Function();
typedef _MessageDart = Pointer<Uint8> Function();

/// Thin FFI adapter that translates Dart calls into Rust library calls.
class RustBridge {
  RustBridge._(DynamicLibrary library)
    : _smokeTest = library.lookupFunction<_SmokeTestNative, _SmokeTestDart>(
        'netra_connection_smoke_test',
      ),
      _message = library.lookupFunction<_MessageNative, _MessageDart>(
        'netra_connection_message',
      );

  final _SmokeTestDart _smokeTest;
  final _MessageDart _message;
  final StreamController<EngineEvent> _engineEvents =
      StreamController<EngineEvent>.broadcast();

  /// Opens the Rust dynamic library for the current platform.
  factory RustBridge.open() {
    return RustBridge._(DynamicLibrary.open(_libraryName));
  }

  /// Emits transport-level events produced while the bridge is being used.
  Stream<EngineEvent> get engineEvents => _engineEvents.stream;

  /// Calls a lightweight Rust verification function to confirm the bridge is active.
  Future<RustConnectionResult> connectionSmokeTest() async {
    final result = RustConnectionResult(
      code: _smokeTest(),
      message: _readNullTerminatedUtf8(_message()),
    );

    _engineEvents.add(
      EngineEvent(type: 'rust_connection_verified', message: result.message),
    );

    return result;
  }

  /// Disposes the bridge-side event stream.
  Future<void> dispose() async {
    await _engineEvents.close();
  }

  static String get _libraryName {
    if (Platform.isWindows) {
      return 'netra_rust.dll';
    }
    if (Platform.isMacOS) {
      return 'libnetra_rust.dylib';
    }
    return 'libnetra_rust.so';
  }

  static String _readNullTerminatedUtf8(Pointer<Uint8> pointer) {
    final bytes = <int>[];
    var current = pointer;

    while (true) {
      final byte = current.value;
      if (byte == 0) {
        return String.fromCharCodes(bytes);
      }
      bytes.add(byte);
      current = current + 1;
    }
  }
}

/// Initializes the Rust engine bridge for the current process.
Future<void> initEngine() async {
  RustBridge.open();
}
