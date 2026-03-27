class FilterList {
  const FilterList({
    required this.id,
    required this.name,
    required this.filePath,
    required this.enabled,
    required this.ruleCount,
    this.url,
    this.lastUpdatedAt,
  });

  final String id;
  final String name;
  final String? url;
  final String filePath;
  final bool enabled;
  final int ruleCount;
  final DateTime? lastUpdatedAt;
}
