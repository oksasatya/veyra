import 'package:dio/dio.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// Map a [DioException] to a [Failure], reading the backend error envelope
/// `{ "meta": ..., "error": { "code", "message" } }` (ADR-0008) for a developer
/// message. The HTTP status selects the Failure type. (Code-based localization —
/// mapping `error.code` to a localized string — is a separate i18n follow-up.)
Failure mapDioError(DioException e) {
  final status = e.response?.statusCode;
  final message = _extractMessage(e.response?.data);
  return switch (status) {
    401 => UnauthorizedFailure(message ?? 'Your session has expired.'),
    403 => UnauthorizedFailure(message ?? 'You do not have access.'),
    404 => NotFoundFailure(message ?? 'Not found.'),
    409 => ConflictFailure(message ?? 'That already exists.'),
    422 => ValidationFailure(message ?? 'Please check your input.'),
    final int s when s >= 500 => ServerFailure(message ?? 'Server error.'),
    null => const NetworkFailure(),
    _ => ServerFailure(message ?? 'Unexpected error.'),
  };
}

/// Read the developer message from the error envelope's `error.message`.
String? _extractMessage(Object? data) {
  if (data is Map && data['error'] is Map) {
    final message = (data['error'] as Map)['message'];
    return message is String ? message : null;
  }
  return null;
}
