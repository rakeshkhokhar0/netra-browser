import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/browser_provider.dart';
import '../providers/tab_provider.dart';
import 'tab_item.dart';

/// Renders the browser tab strip as a horizontally scrollable list.
///
/// This component is purely presentational. It observes the browser state
/// exposed through Riverpod, renders each known tab in order, and relies on
/// [TabItem] to display active and suspended visual states.
class TabStrip extends ConsumerWidget {
  /// Creates the tab strip widget.
  const TabStrip({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final browserState = ref.watch(browserStateProvider);
    final tabs = browserState.tabs;

    return Material(
      color: theme.colorScheme.surface,
      elevation: 1,
      child: SizedBox(
        height: 60,
        child: ListView.separated(
          scrollDirection: Axis.horizontal,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          itemCount: tabs.length + 1,
          separatorBuilder: (_, _) => const SizedBox(width: 4),
          itemBuilder: (context, index) {
            if (index == tabs.length) {
              return IconButton(
                onPressed: () async {
                  if (tabs.length >= TabProvider.maxTabs) {
                    _showTabLimitBanner(context);
                    return;
                  }

                  try {
                    await ref.read(tabProvider).createTab();
                  } on TabLimitReachedException catch (error) {
                    _showTabLimitBanner(context, message: error.toString());
                  } catch (_) {
                    if (!context.mounted) {
                      return;
                    }

                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Unable to open a new tab.'),
                      ),
                    );
                  }
                },
                tooltip: 'New tab',
                icon: const Icon(Icons.add),
              );
            }

            final tab = tabs[index];
            return ConstrainedBox(
              constraints: const BoxConstraints(
                minWidth: 180,
                maxWidth: 260,
              ),
              child: TabItem(tab: tab),
            );
          },
        ),
      ),
    );
  }

  void _showTabLimitBanner(BuildContext context, {String? message}) {
    if (!context.mounted) {
      return;
    }

    final messenger = ScaffoldMessenger.of(context);
    messenger
      ..clearMaterialBanners()
      ..showMaterialBanner(
        MaterialBanner(
          content: Text(message ?? 'Tab limit reached (20 tabs).'),
          leading: const Icon(Icons.info_outline),
          actions: [
            TextButton(
              onPressed: messenger.hideCurrentMaterialBanner,
              child: const Text('Dismiss'),
            ),
          ],
        ),
      );
  }
}
