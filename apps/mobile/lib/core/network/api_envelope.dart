import 'package:dio/dio.dart';

/// Reads the standardized API success envelope:
/// `{ "meta": { "request_id": ... }, "data": <payload> }` (backend ADR-0008).
///
/// Every successful JSON response wraps its payload under `data`; these helpers
/// unwrap it so data sources work with the payload directly. `data` and `error`
/// are mutually exclusive — error bodies are handled by `mapDioError`.
extension ApiEnvelope on Response<Map<String, dynamic>> {
  /// The `data` payload as a single object (single-resource responses).
  Map<String, dynamic> get dataObject => data!['data'] as Map<String, dynamic>;

  /// The `data` payload as a list (collection responses).
  List<dynamic> get dataList => data!['data'] as List<dynamic>;

  /// The request correlation id from `meta.request_id`, when present.
  String? get requestId {
    final meta = data?['meta'];
    return meta is Map<String, dynamic> ? meta['request_id'] as String? : null;
  }
}
