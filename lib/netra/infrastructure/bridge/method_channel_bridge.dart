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

  /// Creates a native browser tab for the provided tab identifier.
  ///
  /// The [tabId] value is forwarded to the native bridge as the `tab_id`
  /// argument expected by the C++ method handler.
  static Future<void> createTab(String tabId) {
    return _invoke('createTab', <String, dynamic>{'tab_id': tabId});
  }

  /// Closes the native browser tab associated with the provided identifier.
  ///
  /// The [tabId] value is forwarded to the native bridge as the `tab_id`
  /// argument expected by the C++ method handler.
  static Future<void> closeTab(String tabId) {
    return _invoke('closeTab', <String, dynamic>{'tab_id': tabId});
  }

  /// Navigates the specified tab to the requested URL.
  ///
  /// The [tabId] identifies the target tab and [url] provides the destination
  /// forwarded to the native `navigate` command.
  static Future<void> navigate(String tabId, String url) {
    return _invoke('navigate', <String, dynamic>{'tab_id': tabId, 'url': url});
  }

  /// Requests backward navigation for the specified tab.
  ///
  /// The [tabId] value is forwarded as the `tab_id` argument to the native
  /// `goBack` command.
  static Future<void> goBack(String tabId) {
    return _invoke('goBack', <String, dynamic>{'tab_id': tabId});
  }

  /// Requests forward navigation for the specified tab.
  ///
  /// The [tabId] value is forwarded as the `tab_id` argument to the native
  /// `goForward` command.
  static Future<void> goForward(String tabId) {
    return _invoke('goForward', <String, dynamic>{'tab_id': tabId});
  }

  /// Requests a reload of the currently loaded page for the specified tab.
  ///
  /// The [tabId] value is forwarded as the `tab_id` argument to the native
  /// `reload` command.
  static Future<void> reload(String tabId) {
    return _invoke('reload', <String, dynamic>{'tab_id': tabId});
  }

  /// Stops the current load operation for the specified tab.
  ///
  /// The [tabId] value is forwarded as the `tab_id` argument to the native
  /// `stopLoading` command.
  static Future<void> stopLoading(String tabId) {
    return _invoke('stopLoading', <String, dynamic>{'tab_id': tabId});
  }

  /// Marks the specified tab as the active visible native tab.
  ///
  /// The [tabId] value is forwarded as the `tab_id` argument to the native
  /// `setActiveTab` command.
  static Future<void> setActiveTab(String tabId) {
    return _invoke('setActiveTab', <String, dynamic>{'tab_id': tabId});
  }

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

  /// Clears native browser data owned by the WebView2 bridge.
  ///
  /// This forwards directly to the native `clearData` command with no
  /// arguments.
  static Future<void> clearData() {
    return _invoke('clearData', <String, dynamic>{});
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
