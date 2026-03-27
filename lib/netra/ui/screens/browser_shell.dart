import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../components/toolbar.dart';
import '../layouts/browser_layout.dart';
import '../providers/tab_provider.dart';

/// Presents the primary browser shell screen for the Netra UI layer.
///
/// This screen owns only the top-level composition of the browser interface.
/// It combines the toolbar region with the main browser content layout while
/// intentionally avoiding browser logic, engine orchestration, or provider
/// coordination.
class BrowserShell extends ConsumerWidget {
  /// Creates the browser shell screen.
  const BrowserShell({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final activeTabId = ref.watch(activeTabIdProvider);
    final tabId = activeTabId.valueOrNull ?? '';

    return Scaffold(
      body: Column(
        children: [
          Toolbar(
            tabId: tabId,
            isEnabled: tabId.isNotEmpty,
          ),
          const Expanded(
            child: BrowserLayout(),
          ),
        ],
      ),
    );
  }
}
