import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/tab_state.dart';
import '../providers/browser_provider.dart';

/// Displays a top-level loading bar for the currently active browser tab.
///
/// This component is fully presentation-focused. It reads the browser state
/// from Riverpod, determines whether the active tab is currently loading, and
/// shows a [LinearProgressIndicator] only while that loading state is true.
class LoadingIndicator extends ConsumerWidget {
  /// Creates the loading indicator widget.
  const LoadingIndicator({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final browserState = ref.watch(browserStateProvider);
    final activeTabId = browserState.activeTabId;
    final activeTab = browserState.tabs.firstWhereOrNull(
      (tab) => tab.id == activeTabId,
    );
    final isActiveTabLoading = activeTab?.isLoading ?? false;

    if (!isActiveTabLoading) {
      return const SizedBox.shrink();
    }

    return const SizedBox(
      height: 3,
      child: LinearProgressIndicator(),
    );
  }
}

extension on Iterable<TabState> {
  /// Returns the first matching tab or `null` when no match is found.
  TabState? firstWhereOrNull(bool Function(TabState tab) test) {
    for (final tab in this) {
      if (test(tab)) {
        return tab;
      }
    }
    return null;
  }
}
