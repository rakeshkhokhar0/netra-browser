/// Immutable UI-facing representation of a browser tab.
///
/// This model mirrors the Rust `Tab` entity fields that are required by the
/// Flutter layer. It is intentionally a plain data object:
/// - no business logic
/// - no FFI integration
/// - no provider dependencies
class TabState {
  /// Creates an immutable browser tab snapshot.
  const TabState({
    required this.id,
    required this.title,
    required this.url,
    required this.isActive,
    required this.isLoading,
    required this.canGoBack,
    required this.canGoForward,
    required this.isSuspended,
    required this.blockedCount,
  });

  /// Stable tab identifier originating from the Rust core.
  final String id;

  /// Current document title associated with the tab.
  final String title;

  /// Current visible URL for the tab.
  final String url;

  /// Whether this tab is the active tab in the browser state.
  final bool isActive;

  /// Whether the tab is currently loading content.
  final bool isLoading;

  /// Whether backward navigation is available for the tab.
  final bool canGoBack;

  /// Whether forward navigation is available for the tab.
  final bool canGoForward;

  /// Whether the tab is currently suspended by the Rust core.
  final bool isSuspended;

  /// Number of requests blocked for this tab by the Rust core.
  final int blockedCount;

  /// Creates a [TabState] from a decoded JSON or map payload.
  ///
  /// Missing values fall back to safe defaults so the UI layer can decode
  /// partial payloads without embedding business rules.
  factory TabState.fromMap(Map<String, dynamic> map) {
    return TabState(
      id: map['id'] as String? ?? '',
      title: map['title'] as String? ?? '',
      url: map['url'] as String? ?? '',
      isActive: map['is_active'] as bool? ?? map['isActive'] as bool? ?? false,
      isLoading:
          map['is_loading'] as bool? ?? map['isLoading'] as bool? ?? false,
      canGoBack:
          map['can_go_back'] as bool? ?? map['canGoBack'] as bool? ?? false,
      canGoForward:
          map['can_go_forward'] as bool? ??
          map['canGoForward'] as bool? ??
          false,
      isSuspended:
          map['is_suspended'] as bool? ??
          map['isSuspended'] as bool? ??
          false,
      blockedCount:
          map['blocked_count'] as int? ?? map['blockedCount'] as int? ?? 0,
    );
  }

  /// Returns a new immutable tab snapshot with selected fields replaced.
  TabState copyWith({
    String? id,
    String? title,
    String? url,
    bool? isActive,
    bool? isLoading,
    bool? canGoBack,
    bool? canGoForward,
    bool? isSuspended,
    int? blockedCount,
  }) {
    return TabState(
      id: id ?? this.id,
      title: title ?? this.title,
      url: url ?? this.url,
      isActive: isActive ?? this.isActive,
      isLoading: isLoading ?? this.isLoading,
      canGoBack: canGoBack ?? this.canGoBack,
      canGoForward: canGoForward ?? this.canGoForward,
      isSuspended: isSuspended ?? this.isSuspended,
      blockedCount: blockedCount ?? this.blockedCount,
    );
  }
}
