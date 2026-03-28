import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/tab_state.dart';
import '../providers/browser_provider.dart';
import '../providers/tab_provider.dart';

/// Renders a single browser tab item inside the tab strip.
///
/// This component is intentionally presentation-focused. It displays the tab
/// title, active styling, and suspended styling, then forwards user actions to
/// provider-owned browser commands without storing local browser state.
class TabItem extends ConsumerWidget {
  /// Creates a tab item for the provided immutable [tab] snapshot.
  const TabItem({
    required this.tab,
    super.key,
  });

  /// Immutable browser tab snapshot used to render this item.
  final TabState tab;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isActive = tab.isActive;
    final isSuspended = tab.isSuspended;

    final backgroundColor = isActive
        ? colorScheme.primaryContainer
        : isSuspended
            ? colorScheme.surfaceContainerHighest
            : colorScheme.surfaceContainer;
    final foregroundColor = isActive
        ? colorScheme.onPrimaryContainer
        : isSuspended
            ? colorScheme.onSurfaceVariant
            : colorScheme.onSurface;
    final borderColor = isActive
        ? colorScheme.primary
        : colorScheme.outlineVariant;

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 6),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(14),
          onTap: () => ref.read(browserProvider).setActiveTab(tab.id),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 180),
            curve: Curves.easeOut,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: backgroundColor,
              borderRadius: BorderRadius.circular(14),
              border: Border.all(color: borderColor),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Expanded(
                  child: Text(
                    _displayTitle,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: foregroundColor,
                      fontWeight: isActive ? FontWeight.w600 : FontWeight.w500,
                      fontStyle: isSuspended ? FontStyle.italic : FontStyle.normal,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton(
                  onPressed: () => ref.read(tabProvider).closeTab(tab.id),
                  tooltip: 'Close tab',
                  visualDensity: VisualDensity.compact,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints.tightFor(width: 28, height: 28),
                  iconSize: 18,
                  color: foregroundColor,
                  icon: const Icon(Icons.close),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  String get _displayTitle {
    if (tab.title.trim().isNotEmpty) {
      return tab.title.trim();
    }
    if (tab.url.trim().isNotEmpty) {
      return tab.url.trim();
    }
    return 'New Tab';
  }
}
