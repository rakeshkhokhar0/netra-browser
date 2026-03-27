import 'package:netra_browser/netra/core/entities/engine_event.dart';

import 'i_request_handler.dart';

/// Defines the platform-agnostic contract for a single browser frame.
abstract class IFrame {
  /// Unique identifier assigned to the frame.
  String get id;

  /// Emits frame-scoped events such as loading and title changes.
  Stream<EngineEvent> get events;

  /// Navigates the frame to the provided URL.
  Future<void> navigate(String url);

  /// Reloads the current document.
  Future<void> reload();

  /// Navigates backward in history when available.
  Future<void> goBack();

  /// Navigates forward in history when available.
  Future<void> goForward();

  /// Stops the current loading operation.
  Future<void> stopLoading();

  /// Returns the current visible URL for the frame.
  Future<String> getUrl();

  /// Returns the current document title when available.
  Future<String?> getTitle();

  /// Executes JavaScript inside the frame.
  Future<String> executeScript(String script);

  /// Assigns the request handler used for request interception.
  Future<void> setRequestHandler(IRequestHandler handler);

  /// Releases all frame resources.
  Future<void> dispose();
}
