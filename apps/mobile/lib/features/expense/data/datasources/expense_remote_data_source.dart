import 'package:dio/dio.dart';
import 'package:veyra_mobile/features/expense/data/models/expense_dto.dart';

/// Raw HTTP calls for expenses. Throws [DioException] on failure (mapped to a
/// Failure in the repository). Auth header is added by the dio interceptor.
class ExpenseRemoteDataSource {
  ExpenseRemoteDataSource(this._dio);
  final Dio _dio;

  Future<List<ExpenseDto>> list(String vehicleId) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/vehicles/$vehicleId/expenses',
    );
    final rows = res.data!['expenses'] as List<dynamic>;
    return rows
        .map((e) => ExpenseDto.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<ExpenseDto> create(
    String vehicleId,
    Map<String, dynamic> body,
  ) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/vehicles/$vehicleId/expenses',
      data: body,
    );
    return ExpenseDto.fromJson(res.data!);
  }
}
