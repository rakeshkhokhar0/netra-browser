import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/tab_state.dart';
import '../providers/browser_provider.dart';
import 'blocked_counter.dart';
import 'address_bar.dart';

/// Renders the browser toolbar shown at the top of the main shell screen.
///
/// This widget owns only the presentational composition of toolbar controls:
/// backward navigation, forward navigation, reload, and the address bar. It
/// forwards user actions into the UI provider layer and intentionally avoids
/// storing browser state or embedding business logic in the component itself.
class Toolbar extends ConsumerWidget {
  /// Creates the browser toolbar.
  const Toolbar({
    this.isEnabled = true,
    super.key,
  });

  /// Whether toolbar actions should currently allow user interaction.
  final bool isEnabled;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final provider = ref.read(browserProvider);
    final browserState = ref.watch(browserStateProvider);
    final activeTabId = browserState.activeTabId;
    final activeTabState = browserState.tabs.firstWhereOrNull(
      (tab) => tab.id == browserState.activeTabId,
    );
    final canGoBack = isEnabled && (activeTabState?.canGoBack ?? false);
    final canGoForward = isEnabled && (activeTabState?.canGoForward ?? false);

    return Material(
      color: theme.colorScheme.surface,
      elevation: 1,
      child: SafeArea(
        bottom: false,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          child: Row(
            children: [
              IconButton(
                onPressed: canGoBack
                    ? () => _runCommand(
                          context,
                          () async {
                            if (activeTabId != null) {
                              await provider.goBack(activeTabId);
                            }
                          },
                        )
                    : null,
                icon: const Icon(Icons.arrow_back),
                tooltip: 'Back',
              ),
              IconButton(
                onPressed:
                    canGoForward
                        ? () => _runCommand(
                              context,
                              () async {
                                if (activeTabId != null) {
                                  await provider.goForward(activeTabId);
                                }
                              },
                            )
                        : null,
                icon: const Icon(Icons.arrow_forward),
                tooltip: 'Forward',
              ),
              IconButton(
                onPressed: isEnabled
                    ? () => _runCommand(
                          context,
                          () async {
                            if (activeTabId != null) {
                              await provider.reload(activeTabId);
                            }
                          },
                        )
                    : null,
                icon: const Icon(Icons.refresh),
                tooltip: 'Reload',
              ),
              const SizedBox(width: 8),
              Expanded(
                child: AddressBar(
                  isEnabled: isEnabled,
                ),
              ),
              const SizedBox(width: 12),
              const BlockedCounter(),
            ],
          ),
        ),
      ),
    );
  }

  /// Executes a toolbar command and surfaces native bridge failures safely.
  ///
  /// The toolbar remains presentation-focused while still ensuring that async
  /// platform failures do not escape as unhandled exceptions in the Flutter
  /// runtime.
  Future<void> _runCommand(
    BuildContext context,
    Future<void> Function() action,
  ) async {
    try {
      await action();
    } on PlatformException catch (error) {
      if (!context.mounted) {
        return;
      }

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            'Browser action failed: ${error.message ?? error.code}',
          ),
        ),
      );
    }
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
