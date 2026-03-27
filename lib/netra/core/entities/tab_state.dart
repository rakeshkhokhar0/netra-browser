class TabState {
  const TabState({
    required this.id,
    required this.url,
    required this.title,
    required this.isLoading,
    required this.canGoBack,
    required this.canGoForward,
    required this.isSuspended,
    required this.blockedCount,
    this.errorState,
  });

  final String id;
  final String url;
  final String title;
  final bool isLoading;
  final bool canGoBack;
  final bool canGoForward;
  final bool isSuspended;
  final int blockedCount;
  final String? errorState;

  TabState copyWith({
    String? id,
    String? url,
    String? title,
    bool? isLoading,
    bool? canGoBack,
    bool? canGoForward,
    bool? isSuspended,
    int? blockedCount,
    String? errorState,
  }) {
    return TabState(
      id: id ?? this.id,
      url: url ?? this.url,
      title: title ?? this.title,
      isLoading: isLoading ?? this.isLoading,
      canGoBack: canGoBack ?? this.canGoBack,
      canGoForward: canGoForward ?? this.canGoForward,
      isSuspended: isSuspended ?? this.isSuspended,
      blockedCount: blockedCount ?? this.blockedCount,
      errorState: errorState ?? this.errorState,
    );
  }
}
