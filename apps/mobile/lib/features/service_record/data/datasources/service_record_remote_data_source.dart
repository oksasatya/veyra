import 'package:dio/dio.dart';
import 'package:veyra_mobile/features/service_record/data/models/service_record_dto.dart';

/// Raw HTTP calls for service records. Throws [DioException] on failure (mapped
/// to a Failure in the repository). Auth header is added by the dio interceptor.
class ServiceRecordRemoteDataSource {
  ServiceRecordRemoteDataSource(this._dio);
  final Dio _dio;

  Future<List<ServiceRecordDto>> list(String vehicleId) async {
    final res = await _dio
        .get<Map<String, dynamic>>('/vehicles/$vehicleId/services');
    final rows = res.data!['records'] as List<dynamic>;
    return rows
        .map((e) => ServiceRecordDto.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<ServiceRecordDto> create(
    String vehicleId,
    Map<String, dynamic> body,
  ) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/vehicles/$vehicleId/services',
      data: body,
    );
    return ServiceRecordDto.fromJson(res.data!);
  }
}
