import 'package:dio/dio.dart';
import 'package:veyra_mobile/features/vehicle/data/models/vehicle_dto.dart';

/// Raw HTTP calls for vehicles. Throws [DioException] on failure (mapped to a
/// Failure in the repository). Auth header is added by the dio interceptor.
class VehicleRemoteDataSource {
  VehicleRemoteDataSource(this._dio);
  final Dio _dio;

  Future<List<VehicleDto>> list() async {
    final res = await _dio.get<Map<String, dynamic>>('/vehicles');
    final rows = res.data!['vehicles'] as List<dynamic>;
    return rows
        .map((e) => VehicleDto.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<VehicleDto> create(Map<String, dynamic> body) async {
    final res = await _dio.post<Map<String, dynamic>>('/vehicles', data: body);
    return VehicleDto.fromJson(res.data!);
  }
}
