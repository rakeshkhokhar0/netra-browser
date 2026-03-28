import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../components/loading_indicator.dart';
import '../components/tab_strip.dart';
import '../components/toolbar.dart';
import '../layouts/browser_layout.dart';
import '../providers/browser_provider.dart';
import '../providers/tab_provider.dart';
import 'error_screen.dart';

/// Presents the primary browser shell screen for the Netra UI layer.
///
/// This screen owns only the top-level composition of the browser interface.
/// It combines the toolbar region with the main browser content layout while
/// intentionally avoiding browser logic, engine orchestration, or provider
/// coordination.
class BrowserShell extends ConsumerStatefulWidget {
  /// Creates the browser shell screen.
  const BrowserShell({super.key});

  @override
  ConsumerState<BrowserShell> createState() => _BrowserShellState();
}

class _BrowserShellState extends ConsumerState<BrowserShell> {
  bool _initialTabRequested = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (_initialTabRequested) {
      return;
    }

    _initialTabRequested = true;
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final browserState = ref.read(browserStateProvider);
      if (browserState.tabs.isNotEmpty) {
        return;
      }

      try {
        await ref.read(tabProvider).createTab();
      } catch (error, stackTrace) {
        FlutterError.reportError(
          FlutterErrorDetails(
            exception: error,
            stack: stackTrace,
            library: 'netra browser shell',
            context: ErrorDescription('while creating the initial browser tab'),
          ),
        );
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final browserState = ref.watch(browserStateProvider);
    final hasError = (browserState.errorMessage ?? '').trim().isNotEmpty;

    return Scaffold(
      body: Column(
        children: [
          const TabStrip(),
          Toolbar(
            isEnabled: (browserState.activeTabId ?? '').isNotEmpty,
          ),
          const LoadingIndicator(),
          Expanded(
            child: Stack(
              fit: StackFit.expand,
              children: [
                const BrowserLayout(),
                if (hasError) const ErrorScreen(),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
