import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/tab_state.dart';
import '../providers/browser_provider.dart';

/// Renders the browser address bar used for URL navigation input.
///
/// This component owns the URL text field and go button that together form the
/// shell-side address bar. It delegates navigation requests to the UI provider
/// layer and keeps the visible URL in sync with typed native browser events.
/// It intentionally avoids calling bridge code directly or containing broader
/// browser logic.
class AddressBar extends ConsumerStatefulWidget {
  /// Creates the address bar for the currently active tab.
  const AddressBar({
    this.isEnabled = true,
    super.key,
  });

  /// Whether the address bar should currently allow user interaction.
  final bool isEnabled;

  @override
  ConsumerState<AddressBar> createState() => _AddressBarState();
}

/// Holds the local text editing state used by the browser address bar.
class _AddressBarState extends ConsumerState<AddressBar> {
  late final TextEditingController _controller;
  late final FocusNode _focusNode;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
    _focusNode = FocusNode();
  }

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final browserState = ref.watch(browserStateProvider);
    final activeTabState = browserState.tabs.firstWhereOrNull(
      (tab) => tab.id == browserState.activeTabId,
    );
    final currentUrl = _displayUrl(activeTabState?.url ?? '');
    final isLoading = activeTabState?.isLoading ?? false;

    if (!_focusNode.hasFocus && _controller.text != currentUrl) {
      _controller.value = TextEditingValue(
        text: currentUrl,
        selection: TextSelection.collapsed(offset: currentUrl.length),
      );
    }

    return Row(
      children: [
        Expanded(
          child: TextField(
            controller: _controller,
            focusNode: _focusNode,
            enabled: widget.isEnabled,
            textInputAction: TextInputAction.go,
            decoration: InputDecoration(
              hintText: 'Search or enter address',
              border: const OutlineInputBorder(),
              isDense: true,
              suffixIcon: isLoading
                  ? const Padding(
                      padding: EdgeInsets.all(10),
                      child: SizedBox(
                        width: 18,
                        height: 18,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      ),
                    )
                  : null,
            ),
            onSubmitted: _submit,
          ),
        ),
        const SizedBox(width: 12),
        FilledButton(
          onPressed: widget.isEnabled ? () => _submit(_controller.text) : null,
          style: FilledButton.styleFrom(
            backgroundColor: theme.colorScheme.primary,
          ),
          child: const Text('Go'),
        ),
      ],
    );
  }

  /// Forwards the current address bar value to the browser provider.
  ///
  /// Empty input is ignored so the component remains a thin presentation
  /// wrapper without inventing navigation behavior.
  Future<void> _submit(String value) async {
    if (!widget.isEnabled) {
      return;
    }

    final activeTabId = ref.watch(browserStateProvider).activeTabId ?? '';
    final url = value.trim();
    if (url.isEmpty || activeTabId.isEmpty) {
      return;
    }

    try {
      await ref.read(browserProvider).navigate(activeTabId, url);
    } on PlatformException catch (error) {
      if (!mounted) {
        return;
      }

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            'Navigation failed: ${error.message ?? error.code}',
          ),
        ),
      );
    }
  }

  String _displayUrl(String url) {
    final normalizedUrl = url.trim().toLowerCase();
    if (normalizedUrl == 'about' || normalizedUrl == 'about:blank') {
      return '';
    }

    return url;
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
