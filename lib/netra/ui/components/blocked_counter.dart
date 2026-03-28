import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/browser_provider.dart';

/// Displays the number of blocked requests reported by the browser core.
///
/// This widget is intentionally presentation-only. It reads the blocked
/// request count from [browserStateProvider] and renders a compact status chip
/// without storing state or invoking browser commands.
class BlockedCounter extends ConsumerWidget {
  /// Creates the blocked-request counter widget.
  const BlockedCounter({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final blockedCount = ref.watch(
      browserStateProvider.select((state) => state.blockedRequestCount),
    );
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: colorScheme.outlineVariant),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.shield_outlined,
            size: 16,
            color: colorScheme.primary,
          ),
          const SizedBox(width: 6),
          Text(
            '$blockedCount blocked',
            style: theme.textTheme.labelMedium?.copyWith(
              color: colorScheme.onSurface,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }
}
