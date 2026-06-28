import 'package:dio/dio.dart';
import 'package:veyra_mobile/features/fuel_log/data/models/fuel_log_dto.dart';

/// Raw HTTP calls for fuel logs. Throws [DioException] on failure (mapped to a
/// Failure in the repository). Auth header is added by the dio interceptor.
class FuelLogRemoteDataSource {
  FuelLogRemoteDataSource(this._dio);
  final Dio _dio;

  Future<List<FuelLogDto>> list(String vehicleId) async {
    final res = await _dio
        .get<Map<String, dynamic>>('/vehicles/$vehicleId/fuel-logs');
    final rows = res.data!['logs'] as List<dynamic>;
    return rows
        .map((e) => FuelLogDto.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<FuelLogDto> create(
    String vehicleId,
    Map<String, dynamic> body,
  ) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/vehicles/$vehicleId/fuel-logs',
      data: body,
    );
    return FuelLogDto.fromJson(res.data!);
  }
}
