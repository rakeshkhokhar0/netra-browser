import '../../../infrastructure/bridge/method_channel_bridge.dart';

/// Executes WebView2-native layout commands through the adapter layer.
///
/// Layout synchronization still originates from Flutter because the embedded
/// WebView surface must follow Flutter-rendered bounds. This helper keeps that
/// narrow native transport inside the adapter layer so UI/providers do not
/// import infrastructure bridge transports directly.
class WebView2NativeExecutor {
  WebView2NativeExecutor._();

  /// Updates the native bounds for the specified tab host surface.
  static Future<void> setBounds(
    String tabId,
    double x,
    double y,
    double width,
    double height,
  ) {
    return MethodChannelBridge.setBounds(tabId, x, y, width, height);
  }
}
