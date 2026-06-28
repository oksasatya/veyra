import 'package:dio/dio.dart';
import 'package:veyra_mobile/features/document/data/models/document_dto.dart';

/// Raw HTTP calls for documents. Throws [DioException] on failure (mapped to a
/// Failure in the repository). Auth header is added by the dio interceptor.
class DocumentRemoteDataSource {
  DocumentRemoteDataSource(this._dio);
  final Dio _dio;

  Future<List<DocumentDto>> list(String vehicleId) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/vehicles/$vehicleId/documents',
    );
    final rows = res.data!['documents'] as List<dynamic>;
    return rows
        .map((e) => DocumentDto.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<DocumentDto> create(
    String vehicleId,
    Map<String, dynamic> body,
  ) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/vehicles/$vehicleId/documents',
      data: body,
    );
    return DocumentDto.fromJson(res.data!);
  }
}
