import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/browser_provider.dart';

/// Provides the main content area for the browser shell.
///
/// This layout widget defines the in-window region that the native WebView
/// surface should occupy. It still does not render the WebView itself, but it
/// measures its own bounds and forwards that geometry through the UI provider
/// layer so the native host can be embedded into the same application window.
class BrowserLayout extends ConsumerStatefulWidget {
  /// Creates the primary browser content layout widget.
  const BrowserLayout({super.key});

  @override
  ConsumerState<BrowserLayout> createState() => _BrowserLayoutState();
}

/// Tracks the Flutter layout region that should host the native WebView area.
class _BrowserLayoutState extends ConsumerState<BrowserLayout> {
  final GlobalKey _layoutKey = GlobalKey();
  Rect? _lastReportedBounds;
  String? _lastReportedTabId;
  bool _syncInProgress = false;
  bool _resyncRequested = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final tabId = ref.watch(
          browserStateProvider.select((state) => state.activeTabId),
        ) ??
        '';

    WidgetsBinding.instance.addPostFrameCallback((_) {
      _syncNativeBounds(tabId);
    });

    return Container(
      key: _layoutKey,
      width: double.infinity,
      height: double.infinity,
      color: theme.colorScheme.surfaceContainerLowest,
    );
  }

  /// Synchronizes the native WebView host bounds with the Flutter layout area.
  ///
  /// The method measures the current widget region in logical pixels, converts
  /// the geometry to physical pixels for the native Windows bridge, and
  /// forwards the result through the UI provider layer.
  Future<void> _syncNativeBounds(String tabId) async {
    if (!mounted || tabId.isEmpty) {
      return;
    }

    if (_syncInProgress) {
      _resyncRequested = true;
      return;
    }

    final renderObject = _layoutKey.currentContext?.findRenderObject();
    final renderBox = renderObject is RenderBox ? renderObject : null;
    if (renderBox == null || !renderBox.hasSize) {
      return;
    }

    final offset = renderBox.localToGlobal(Offset.zero);
    final size = renderBox.size;
    final devicePixelRatio = MediaQuery.of(context).devicePixelRatio;
    final nativeBounds = Rect.fromLTWH(
      offset.dx * devicePixelRatio,
      offset.dy * devicePixelRatio,
      size.width * devicePixelRatio,
      size.height * devicePixelRatio,
    );

    if (_lastReportedTabId == tabId && _lastReportedBounds == nativeBounds) {
      return;
    }

    _lastReportedTabId = tabId;
    _lastReportedBounds = nativeBounds;

    final provider = ref.read(browserProvider);
    try {
      _syncInProgress = true;
      await provider.setBounds(
        tabId,
        nativeBounds.left,
        nativeBounds.top,
        nativeBounds.width,
        nativeBounds.height,
      );
    } catch (error, stackTrace) {
      FlutterError.reportError(
        FlutterErrorDetails(
          exception: error,
          stack: stackTrace,
          library: 'netra browser layout',
          context: ErrorDescription(
            'while synchronizing the embedded native WebView bounds',
          ),
        ),
      );
    } finally {
      _syncInProgress = false;
      if (_resyncRequested && mounted) {
        _resyncRequested = false;
        WidgetsBinding.instance.addPostFrameCallback((_) {
          _syncNativeBounds(
            ref.read(browserStateProvider).activeTabId ?? '',
          );
        });
      }
    }
  }
}
