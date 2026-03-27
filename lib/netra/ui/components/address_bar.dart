import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/browser_provider.dart';
import '../providers/tab_provider.dart';

/// Renders the browser address bar used for URL navigation input.
///
/// This component owns the URL text field and go button that together form the
/// shell-side address bar. It delegates navigation requests to the UI provider
/// layer and keeps the visible URL in sync with typed native browser events.
/// It intentionally avoids calling bridge code directly or containing broader
/// browser logic.
class AddressBar extends ConsumerStatefulWidget {
  /// Creates the address bar for the provided active tab identifier.
  const AddressBar({
    required this.tabId,
    this.isEnabled = true,
    super.key,
  });

  /// Active tab identifier that should receive navigation commands.
  final String tabId;

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
    final activeTabState = ref.watch(activeTabStateProvider).valueOrNull;
    final currentUrl = activeTabState?.url ?? '';

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
            decoration: const InputDecoration(
              hintText: 'Enter a URL',
              border: OutlineInputBorder(),
              isDense: true,
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

    final url = value.trim();
    if (url.isEmpty || widget.tabId.isEmpty) {
      return;
    }

    try {
      await ref.read(browserProvider).navigate(widget.tabId, url);
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
}
