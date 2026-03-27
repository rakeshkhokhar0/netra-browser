enum DownloadStatus { pending, downloading, done, failed, cancelled }

class DownloadState {
  const DownloadState({
    required this.id,
    required this.url,
    required this.filename,
    required this.savePath,
    required this.downloadedBytes,
    required this.status,
    required this.startedAt,
    this.totalBytes,
  });

  final String id;
  final String url;
  final String filename;
  final String savePath;
  final int? totalBytes;
  final int downloadedBytes;
  final DownloadStatus status;
  final DateTime startedAt;
}
