import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/tab_state.dart';
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
          onTap: () => ref.read(tabProvider).switchTab(tab.id),
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
                _TabFavicon(
                  faviconUrl: tab.faviconUrl,
                  isLoading: tab.isLoading,
                  color: foregroundColor,
                ),
                const SizedBox(width: 10),
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
    final normalizedUrl = tab.url.trim().toLowerCase();
    if (normalizedUrl == 'about' || normalizedUrl == 'about:blank') {
      return 'New Tab';
    }
    if (tab.url.trim().isNotEmpty) {
      return tab.url.trim();
    }
    return 'New Tab';
  }
}

class _TabFavicon extends StatelessWidget {
  const _TabFavicon({
    required this.faviconUrl,
    required this.isLoading,
    required this.color,
  });

  final String? faviconUrl;
  final bool isLoading;
  final Color color;

  @override
  Widget build(BuildContext context) {
    if (isLoading) {
      return const SizedBox(
        width: 16,
        height: 16,
        child: CircularProgressIndicator(strokeWidth: 2),
      );
    }

    final normalizedUrl = faviconUrl?.trim() ?? '';
    if (normalizedUrl.isEmpty) {
      return Icon(Icons.language, size: 16, color: color);
    }

    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: Image.network(
        normalizedUrl,
        width: 16,
        height: 16,
        fit: BoxFit.cover,
        errorBuilder: (_, __, ___) {
          return Icon(Icons.language, size: 16, color: color);
        },
      ),
    );
  }
}
