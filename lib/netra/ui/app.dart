import 'package:flutter/material.dart';

import 'screens/browser_shell.dart';

/// Defines the root Flutter application for the Netra UI shell.
///
/// This widget owns only top-level application composition concerns such as
/// `MaterialApp` setup, the shared light theme, and the initial route into the
/// browser shell. It intentionally avoids browser business logic so feature
/// coordination remains outside the UI root.
class NetraApp extends StatelessWidget {
  /// Creates the root Netra application widget.
  const NetraApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Netra Browser',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF214B6B)),
        scaffoldBackgroundColor: const Color(0xFFF2F5F3),
        brightness: Brightness.light,
        useMaterial3: true,
      ),
      home: const BrowserShell(),
    );
  }
}
