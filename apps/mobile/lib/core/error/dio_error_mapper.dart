import 'package:dio/dio.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// Map a [DioException] to a [Failure], reading the backend error envelope
/// `{ "meta": ..., "error": { "code", "message" } }` (ADR-0008). The HTTP status
/// selects the Failure type; the stable `code` is carried on [ValidationFailure]
/// (the granular 422 cases) so the UI can localize it via `localizedFailure`.
Failure mapDioError(DioException e) {
  final status = e.response?.statusCode;
  final (code, message) = _extractError(e.response?.data);
  return switch (status) {
    401 => UnauthorizedFailure(message ?? 'Your session has expired.'),
    403 => UnauthorizedFailure(message ?? 'You do not have access.'),
    404 => NotFoundFailure(message ?? 'Not found.'),
    409 => ConflictFailure(message ?? 'That already exists.'),
    422 => ValidationFailure(message ?? 'Please check your input.', code: code),
    final int s when s >= 500 => ServerFailure(message ?? 'Server error.'),
    null => const NetworkFailure(),
    _ => ServerFailure(message ?? 'Unexpected error.'),
  };
}

/// Extract `(code, message)` from the error envelope, when present.
(String?, String?) _extractError(Object? data) {
  if (data is Map && data['error'] is Map) {
    final error = data['error'] as Map;
    final code = error['code'];
    final message = error['message'];
    return (code is String ? code : null, message is String ? message : null);
  }
  return (null, null);
}
