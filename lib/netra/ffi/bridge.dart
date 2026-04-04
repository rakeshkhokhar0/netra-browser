import 'package:ffi/ffi.dart';
import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:netra_browser/netra/core/entities/engine_event.dart';
import 'package:netra_browser/netra/engine/adapters/webview2/webview2_native_executor.dart';

/// Immutable bridge-verification payload returned by the Rust FFI boundary.
class RustConnectionResult {
  const RustConnectionResult({required this.code, required this.message});

  final int code;
  final String message;
}

/// Lightweight Rust tab handle returned by the direct Dart FFI bridge.
class RustTabHandle {
  const RustTabHandle({
    required this.id,
  });

  final String id;
}

/// Read-only browser-state snapshot returned by the Rust core.
class RustBrowserStateSnapshot {
  const RustBrowserStateSnapshot({
    required this.activeTabId,
    required this.tabs,
  });

  final String? activeTabId;
  final List<Map<String, dynamic>> tabs;

  factory RustBrowserStateSnapshot.fromJsonString(String json) {
    final decoded = jsonDecode(json) as Map<String, dynamic>;
    final rawTabs = decoded['tabs'] as List<dynamic>? ?? const <dynamic>[];

    return RustBrowserStateSnapshot(
      activeTabId:
          decoded['active_tab_id'] as String? ??
          decoded['activeTabId'] as String?,
      tabs: rawTabs
          .whereType<Map<String, dynamic>>()
          .map(Map<String, dynamic>.from)
          .toList(growable: false),
    );
  }
}

typedef _SmokeTestNative = Int32 Function();
typedef _SmokeTestDart = int Function();
typedef _MessageNative = Pointer<Uint8> Function();
typedef _MessageDart = Pointer<Uint8> Function();
typedef _StringOperationNative = Int32 Function(Pointer<Utf8>);
typedef _StringOperationDart = int Function(Pointer<Utf8>);
typedef _TwoStringOperationNative = Int32 Function(Pointer<Utf8>, Pointer<Utf8>);
typedef _TwoStringOperationDart = int Function(Pointer<Utf8>, Pointer<Utf8>);
typedef _StringFreeNative = Void Function(Pointer<Utf8>);
typedef _StringFreeDart = void Function(Pointer<Utf8>);
typedef _CreateTabIdNative = Pointer<Utf8> Function();
typedef _CreateTabIdDart = Pointer<Utf8> Function();
typedef _GetBrowserStateJsonNative = Pointer<Utf8> Function();
typedef _GetBrowserStateJsonDart = Pointer<Utf8> Function();

final class FfiTabState extends Struct {
  external Pointer<Utf8> tabId;
  external Pointer<Utf8> url;
  external Pointer<Utf8> title;
  external Pointer<Utf8> faviconUrl;

  @Uint8()
  external int isActive;

  @Uint8()
  external int isLoading;

  @Uint8()
  external int canGoBack;

  @Uint8()
  external int canGoForward;

  @Uint8()
  external int isSuspended;

  @Uint32()
  external int blockedCount;

  @Uint32()
  external int sequenceNumber;
}

final class FfiBrowserEvent extends Struct {
  @Uint32()
  external int sequenceNumber;

  @Int32()
  external int eventType;

  external Pointer<Utf8> tabId;
  external Pointer<Utf8> faviconUrl;
  external FfiTabState tabState;
}

typedef _FlutterEventCallbackNative = Void Function(FfiBrowserEvent);
typedef _RegisterEventDispatcherNative = Int32 Function(Pointer<NativeFunction<_FlutterEventCallbackNative>>);
typedef _RegisterEventDispatcherDart = int Function(Pointer<NativeFunction<_FlutterEventCallbackNative>>);

/// Thin FFI adapter that translates Dart calls into Rust library calls.
class RustBridge {
  RustBridge._(DynamicLibrary library)
    : _smokeTest = library.lookupFunction<_SmokeTestNative, _SmokeTestDart>(
        'netra_connection_smoke_test',
      ),
      _message = library.lookupFunction<_MessageNative, _MessageDart>(
        'netra_connection_message',
      ),
      _createTabId =
          library.lookupFunction<_CreateTabIdNative, _CreateTabIdDart>(
            'netra_create_tab_id',
          ),
      _closeTab =
          library.lookupFunction<_StringOperationNative, _StringOperationDart>(
            'netra_close_tab',
          ),
      _setActiveTab =
          library.lookupFunction<_StringOperationNative, _StringOperationDart>(
            'netra_set_active_tab',
          ),
      _loadUrl =
          library
              .lookupFunction<_TwoStringOperationNative, _TwoStringOperationDart>(
                'netra_load_url',
              ),
      _goBack =
          library.lookupFunction<_StringOperationNative, _StringOperationDart>(
            'netra_go_back',
          ),
      _goForward =
          library.lookupFunction<_StringOperationNative, _StringOperationDart>(
            'netra_go_forward',
          ),
      _reload =
          library.lookupFunction<_StringOperationNative, _StringOperationDart>(
            'netra_reload',
          ),
      _stopLoading =
          library.lookupFunction<_StringOperationNative, _StringOperationDart>(
            'netra_stop_loading',
          ),
      _registerEventDispatcher =
          library.lookupFunction<_RegisterEventDispatcherNative, _RegisterEventDispatcherDart>(
            'netra_register_event_dispatcher',
          ),
      _getBrowserStateJson = library.lookupFunction<
        _GetBrowserStateJsonNative,
        _GetBrowserStateJsonDart
      >('netra_get_browser_state_json'),
      _stringFree = library.lookupFunction<_StringFreeNative, _StringFreeDart>(
        'netra_string_free',
      );

  final _SmokeTestDart _smokeTest;
  final _MessageDart _message;
  final _CreateTabIdDart _createTabId;
  final _StringOperationDart _closeTab;
  final _StringOperationDart _setActiveTab;
  final _TwoStringOperationDart _loadUrl;
  final _StringOperationDart _goBack;
  final _StringOperationDart _goForward;
  final _StringOperationDart _reload;
  final _StringOperationDart _stopLoading;
  final _RegisterEventDispatcherDart _registerEventDispatcher;
  final _GetBrowserStateJsonDart _getBrowserStateJson;
  final _StringFreeDart _stringFree;
  final StreamController<EngineEvent> _engineEvents =
      StreamController<EngineEvent>.broadcast();

  final StreamController<Map<String, dynamic>> _browserEvents =
      StreamController<Map<String, dynamic>>.broadcast();

  Stream<Map<String, dynamic>> get browserEvents => _browserEvents.stream;

  late final NativeCallable<_FlutterEventCallbackNative> _eventListener;

  static final RustBridge instance = RustBridge.open();

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

  /// Creates a new Rust-owned browser tab and returns its identifier.
  ///
  /// Rust is the only owner of tab-ID generation, so Flutter simply requests a
  /// new tab and reuses the returned identifier everywhere else.
  Future<RustTabHandle> createTab() async {
    final pointer = _createTabId();
    if (pointer == nullptr) {
      throw StateError('Rust createTab failed.');
    }

    try {
      final rustTabId = pointer.toDartString();
      return RustTabHandle(id: rustTabId);
    } finally {
      _stringFree(pointer);
    }
  }

  /// Closes the Rust-owned browser tab associated with the provided identifier.
  Future<void> closeTab({
    required String tabId,
  }) async {
    _invokeStringOperation(
      operationName: 'closeTab',
      value: tabId,
      operation: _closeTab,
    );
  }

  /// Marks the provided Rust-owned tab as the active browser tab.
  Future<void> setActiveTab({
    required String tabId,
  }) async {
    _invokeStringOperation(
      operationName: 'setActiveTab',
      value: tabId,
      operation: _setActiveTab,
    );
  }

  /// Loads [url] in the Rust-owned tab identified by [tabId].
  Future<void> loadUrl({
    required String tabId,
    required String url,
  }) async {
    final tabIdPointer = tabId.toNativeUtf8();
    final urlPointer = url.toNativeUtf8();

    try {
      final status = _loadUrl(tabIdPointer, urlPointer);
      if (status != 1) {
        throw StateError('Rust loadUrl failed for tab $tabId.');
      }
    } finally {
      malloc.free(tabIdPointer);
      malloc.free(urlPointer);
    }
  }

  /// Requests backward navigation for the Rust-owned tab identified by [tabId].
  Future<void> goBack({
    required String tabId,
  }) async {
    _invokeStringOperation(
      operationName: 'goBack',
      value: tabId,
      operation: _goBack,
    );
  }

  /// Requests forward navigation for the Rust-owned tab identified by [tabId].
  Future<void> goForward({
    required String tabId,
  }) async {
    _invokeStringOperation(
      operationName: 'goForward',
      value: tabId,
      operation: _goForward,
    );
  }

  /// Reloads the Rust-owned tab identified by [tabId].
  Future<void> reload({
    required String tabId,
  }) async {
    _invokeStringOperation(
      operationName: 'reload',
      value: tabId,
      operation: _reload,
    );
  }

  /// Stops loading in the Rust-owned tab identified by [tabId].
  Future<void> stopLoading({
    required String tabId,
  }) async {
    _invokeStringOperation(
      operationName: 'stopLoading',
      value: tabId,
      operation: _stopLoading,
    );
  }

  /// Updates the native bounds used to embed a browser tab in the Flutter UI.
  Future<void> setBounds({
    required String tabId,
    required double x,
    required double y,
    required double width,
    required double height,
  }) {
    return WebView2NativeExecutor.setBounds(tabId, x, y, width, height);
  }

  void _initDispatcher() {
    _eventListener = NativeCallable<_FlutterEventCallbackNative>.listener(_onNativeEvent);
    _registerEventDispatcher(_eventListener.nativeFunction);
  }

  static void _onNativeEvent(FfiBrowserEvent event) {
    String safeString(Pointer<Utf8> ptr) {
      if (ptr == nullptr) return '';
      final str = ptr.toDartString();
      RustBridge.instance._stringFree(ptr);
      return str;
    }

    Map<String, dynamic>? safeTabState(FfiTabState ffiTabState) {
      final tabId = safeString(ffiTabState.tabId);
      final url = safeString(ffiTabState.url);
      final title = safeString(ffiTabState.title);
      final faviconUrl = safeString(ffiTabState.faviconUrl);

      if (tabId.isEmpty) {
        return null;
      }

      return {
        'id': tabId,
        'url': url,
        'title': title,
        'faviconUrl': faviconUrl.isEmpty ? null : faviconUrl,
        'isActive': ffiTabState.isActive != 0,
        'isLoading': ffiTabState.isLoading != 0,
        'canGoBack': ffiTabState.canGoBack != 0,
        'canGoForward': ffiTabState.canGoForward != 0,
        'isSuspended': ffiTabState.isSuspended != 0,
        'blockedCount': ffiTabState.blockedCount,
        'sequenceNumber': ffiTabState.sequenceNumber,
      };
    }

    final seq = event.sequenceNumber;
    final tabId = safeString(event.tabId);
    final faviconUrl = safeString(event.faviconUrl);
    print('[Flutter] Event received from Rust');
    final tabState = safeTabState(event.tabState);

    String eventName;

    switch (event.eventType) {
      case 1:
        eventName = 'navigationStarting';
        break;
      case 2:
        eventName = 'frameCreated';
        break;
      case 3:
        eventName = 'frameDestroyed';
        break;
      case 4:
        eventName = 'contentLoading';
        break;
      case 5:
        eventName = 'navigationCompleted';
        break;
      case 6:
        eventName = 'titleChanged';
        break;
      case 7:
        eventName = 'historyChanged';
        break;
      case 8:
        eventName = 'requestBlocked';
        break;
      case 9:
        eventName = 'navigationFailed';
        break;
      case 10:
        eventName = 'urlChanged';
        break;
      case 11:
        eventName = 'loadFinished';
        break;
      case 12:
        eventName = 'faviconChanged';
        break;
      case 13:
        eventName = 'tabCrashed';
        break;
      default:
        return; // Ignore
    }

    RustBridge.instance._browserEvents.add({
      'type': eventName,
      'tabId': tabId,
      'faviconUrl': faviconUrl,
      'sequenceNumber': seq,
      'tabState': tabState,
    });
  }

  /// Returns the current browser-state snapshot owned by the Rust core.
  Future<RustBrowserStateSnapshot> getBrowserState() async {
    final pointer = _getBrowserStateJson();
    if (pointer == nullptr) {
      throw StateError('Rust getBrowserState failed.');
    }

    try {
      return RustBrowserStateSnapshot.fromJsonString(pointer.toDartString());
    } finally {
      _stringFree(pointer);
    }
  }

  /// Disposes the bridge-side event stream.
  Future<void> dispose() async {
    _eventListener.close();
    await _engineEvents.close();
    await _browserEvents.close();
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

  void _invokeStringOperation({
    required String operationName,
    required String value,
    required _StringOperationDart operation,
  }) {
    final pointer = value.toNativeUtf8();

    try {
      final status = operation(pointer);
      if (status != 1) {
        throw StateError('Rust $operationName failed for `$value`.');
      }
    } finally {
      malloc.free(pointer);
    }
  }
}

/// Initializes the Rust engine bridge for the current process.
Future<void> initEngine() async {
  RustBridge.instance._initDispatcher();
}
