import 'package:dio/dio.dart';
import 'package:veyra_mobile/core/network/api_envelope.dart';
import 'package:veyra_mobile/features/vehicle/data/models/vehicle_dto.dart';
import 'package:veyra_mobile/features/vehicle/data/models/vehicle_summary_dto.dart';

/// Raw HTTP calls for vehicles. Throws [DioException] on failure (mapped to a
/// Failure in the repository). Auth header is added by the dio interceptor.
class VehicleRemoteDataSource {
  VehicleRemoteDataSource(this._dio);
  final Dio _dio;

  Future<List<VehicleDto>> list() async {
    final res = await _dio.get<Map<String, dynamic>>('/vehicles');
    return res.dataList
        .map((e) => VehicleDto.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<VehicleDto> create(Map<String, dynamic> body) async {
    final res = await _dio.post<Map<String, dynamic>>('/vehicles', data: body);
    return VehicleDto.fromJson(res.dataObject);
  }

  Future<VehicleSummaryDto> summary(String vehicleId) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/vehicles/$vehicleId/summary',
    );
    return VehicleSummaryDto.fromJson(res.dataObject);
  }
}
