class Bookmark {
  const Bookmark({
    required this.id,
    required this.url,
    required this.title,
    required this.createdAt,
    this.folderId,
    this.faviconPath,
    this.sortOrder = 0,
  });

  final String id;
  final String url;
  final String title;
  final String? folderId;
  final String? faviconPath;
  final DateTime createdAt;
  final int sortOrder;
}
