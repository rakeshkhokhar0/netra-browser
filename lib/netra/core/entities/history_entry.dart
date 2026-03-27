class HistoryEntry {
  const HistoryEntry({
    required this.id,
    required this.url,
    required this.title,
    required this.lastVisitedAt,
    required this.visitCount,
  });

  final String id;
  final String url;
  final String title;
  final DateTime lastVisitedAt;
  final int visitCount;
}
