/// Describes a single resource request observed by the browser engine.
class ResourceRequest {
  /// Creates a request description for interception.
  const ResourceRequest({
    required this.url,
    required this.method,
    required this.headers,
    required this.type,
  });

  /// Target request URL.
  final String url;

  /// HTTP method associated with the request.
  final String method;

  /// Request headers supplied by the engine.
  final Map<String, String> headers;

  /// Resource classification used by privacy and filtering layers.
  final ResourceType type;
}

/// Enumerates the resource categories handled by the request pipeline.
enum ResourceType {
  document,
  image,
  script,
  stylesheet,
  xhr,
  media,
  other,
}

/// Describes the shell-side response decision for a resource request.
class ResourceResponse {
  /// Creates a request decision payload.
  const ResourceResponse({
    required this.allow,
    required this.block,
    this.customBody,
    this.headers,
    this.statusCode,
  });

  /// Indicates that the engine should allow the request to continue.
  final bool allow;

  /// Indicates that the engine should block the request.
  final bool block;

  /// Optional synthetic response body when a custom response is returned.
  final String? customBody;

  /// Optional response headers for a synthetic response.
  final Map<String, String>? headers;

  /// Optional HTTP status code for a synthetic response.
  final int? statusCode;
}

/// Defines the platform-agnostic request interception contract.
abstract class IRequestHandler {
  /// Handles a resource request and returns the allow or block decision.
  Future<ResourceResponse> handleRequest(ResourceRequest request);
}
