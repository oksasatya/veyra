import 'package:dio/dio.dart';
import 'package:veyra_mobile/core/network/api_envelope.dart';
import 'package:veyra_mobile/features/reminder/data/models/reminder_dto.dart';

/// Raw HTTP calls for reminders. Throws [DioException] on failure (mapped to a
/// Failure in the repository). Auth header is added by the dio interceptor.
class ReminderRemoteDataSource {
  ReminderRemoteDataSource(this._dio);
  final Dio _dio;

  Future<List<ReminderDto>> list(String vehicleId) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/vehicles/$vehicleId/reminders',
    );
    return res.dataList
        .map((e) => ReminderDto.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<ReminderDto> create(
    String vehicleId,
    Map<String, dynamic> body,
  ) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/vehicles/$vehicleId/reminders',
      data: body,
    );
    return ReminderDto.fromJson(res.dataObject);
  }

  Future<ReminderDto> complete(String vehicleId, String reminderId) async {
    final res = await _dio.patch<Map<String, dynamic>>(
      '/vehicles/$vehicleId/reminders/$reminderId',
      data: const {'is_completed': true},
    );
    return ReminderDto.fromJson(res.dataObject);
  }
}
