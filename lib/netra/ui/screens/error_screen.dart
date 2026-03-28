import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/browser_provider.dart';

/// Displays a centered browser error view derived from provider state.
///
/// This screen is presentation-only. It reads the current error message from
/// [browserStateProvider], classifies the message into a browser-friendly
/// category such as network, DNS, or SSL, and renders a concise fallback UI.
class ErrorScreen extends ConsumerWidget {
  /// Creates the browser error screen.
  const ErrorScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final errorMessage = ref.watch(
      browserStateProvider.select((state) => state.errorMessage),
    );
    if (errorMessage == null || errorMessage.trim().isEmpty) {
      return const SizedBox.shrink();
    }

    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final details = _classifyError(errorMessage);

    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 420),
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: colorScheme.surfaceContainerLow,
              borderRadius: BorderRadius.circular(20),
              border: Border.all(color: colorScheme.outlineVariant),
            ),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    details.icon,
                    size: 40,
                    color: colorScheme.error,
                  ),
                  const SizedBox(height: 16),
                  Text(
                    details.title,
                    textAlign: TextAlign.center,
                    style: theme.textTheme.headlineSmall?.copyWith(
                      color: colorScheme.onSurface,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  const SizedBox(height: 12),
                  Text(
                    details.message,
                    textAlign: TextAlign.center,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: colorScheme.onSurfaceVariant,
                      height: 1.45,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  _ErrorDetails _classifyError(String? rawMessage) {
    final message = (rawMessage ?? '').trim();
    final normalized = message.toLowerCase();

    if (normalized.contains('dns') ||
        normalized.contains('host') ||
        normalized.contains('name not resolved') ||
        normalized.contains('resolve')) {
      return _ErrorDetails(
        title: 'DNS Error',
        message: message.isEmpty
            ? 'The browser could not resolve the requested address.'
            : message,
        icon: Icons.dns_outlined,
      );
    }

    if (normalized.contains('ssl') ||
        normalized.contains('tls') ||
        normalized.contains('certificate') ||
        normalized.contains('cert')) {
      return _ErrorDetails(
        title: 'SSL Error',
        message: message.isEmpty
            ? 'The browser could not establish a secure connection.'
            : message,
        icon: Icons.lock_outline,
      );
    }

    if (normalized.contains('network') ||
        normalized.contains('connection') ||
        normalized.contains('timeout') ||
        normalized.contains('unreachable') ||
        normalized.contains('offline')) {
      return _ErrorDetails(
        title: 'Network Error',
        message: message.isEmpty
            ? 'The browser could not connect to the requested destination.'
            : message,
        icon: Icons.wifi_off_outlined,
      );
    }

    return _ErrorDetails(
      title: 'Browser Error',
      message: message.isEmpty ? 'An unexpected browser error occurred.' : message,
      icon: Icons.error_outline,
    );
  }
}

/// Immutable UI details used to render the error screen consistently.
class _ErrorDetails {
  /// Creates a set of derived error-screen presentation values.
  const _ErrorDetails({
    required this.title,
    required this.message,
    required this.icon,
  });

  /// Human-readable error category title.
  final String title;

  /// User-facing message shown in the center of the screen.
  final String message;

  /// Icon representing the error category.
  final IconData icon;
}
