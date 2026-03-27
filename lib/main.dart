import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'netra/ffi/bridge_initializer.dart';
import 'netra/ui/app.dart';

/// Launches the Netra Flutter shell after initializing the Rust bridge.
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await BridgeInitializer.init();
  runApp(
    const ProviderScope(
      child: NetraApp(),
    ),
  );
}
