import 'package:flutter/services.dart';

import '../../shared/config/app_config.dart';

/// Wraps the native browser MethodChannel used by the Flutter shell.
///
/// This infrastructure adapter owns only the command transport boundary from
/// Dart into the native C++ WebView2 bridge. It serializes command arguments
/// into `Map<String, dynamic>` payloads and forwards them asynchronously over a
/// single shared [MethodChannel].
///
/// The bridge does not contain business logic, maintain engine state, or
/// interpret native responses beyond forwarding invocation completion back to
/// callers.
class MethodChannelBridge {
  MethodChannelBridge._();

  /// Shared MethodChannel used for all browser bridge commands.
  ///
  /// The channel name is sourced from [AppConfig] so callers and native code
  /// can depend on a single stable command transport identifier.
  static final MethodChannel _channel = MethodChannel(
    AppConfig.browserMethodChannelName,
  );

  /// Updates the native bounds for the specified tab host surface.
  ///
  /// The [tabId] identifies the target tab while [x], [y], [width], and
  /// [height] are forwarded to the native `setBounds` command inside the
  /// nested `bounds` payload.
  static Future<void> setBounds(
    String tabId,
    double x,
    double y,
    double width,
    double height,
  ) {
    return _invoke('setBounds', <String, dynamic>{
      'tab_id': tabId,
      'bounds': <String, dynamic>{
        'left': x.round(),
        'top': y.round(),
        'right': (x + width).round(),
        'bottom': (y + height).round(),
      },
    });
  }

  /// Forwards a method invocation to the shared native MethodChannel.
  ///
  /// The [method] argument must match the method names handled by the C++
  /// bridge exactly, and [arguments] is serialized as a `Map<String, dynamic>`
  /// payload for the native side.
  static Future<void> _invoke(String method, Map<String, dynamic> arguments) {
    return _channel.invokeMethod<void>(method, arguments);
  }
}
