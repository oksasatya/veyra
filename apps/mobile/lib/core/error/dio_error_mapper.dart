import 'package:dio/dio.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// Map a [DioException] to a [Failure], reading the backend's flat error body
/// `{"error":"<message>"}` for a human message when present.
Failure mapDioError(DioException e) {
  final status = e.response?.statusCode;
  final message = _extractMessage(e.response?.data);
  return switch (status) {
    401 => UnauthorizedFailure(message ?? 'Your session has expired.'),
    404 => NotFoundFailure(message ?? 'Not found.'),
    409 => ConflictFailure(message ?? 'That already exists.'),
    422 => ValidationFailure(message ?? 'Please check your input.'),
    final int s when s >= 500 => ServerFailure(message ?? 'Server error.'),
    null => const NetworkFailure(),
    _ => ServerFailure(message ?? 'Unexpected error.'),
  };
}

String? _extractMessage(Object? data) {
  if (data is Map && data['error'] is String) {
    return data['error'] as String;
  }
  return null;
}
