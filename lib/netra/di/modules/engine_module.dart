import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:netra_browser/netra/engine/interfaces/i_engine.dart';
import 'package:netra_browser/netra/engine/adapters/webview2/webview2_adapter.dart';

/// Single provider for the engine instance.
///
/// This is the ONLY file in lib/ (besides webview2_adapter.dart itself) that
/// is permitted to import the adapter directly. All other files must consume
/// the engine through this provider using the [IEngine] interface.
final engineProvider = Provider<IEngine>((ref) {
  return const WebView2Adapter();
});
