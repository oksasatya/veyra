import 'package:dio/dio.dart';
import 'package:veyra_mobile/features/auth/data/models/auth_tokens_dto.dart';
import 'package:veyra_mobile/features/auth/data/models/user_dto.dart';

/// One register/login result: the user plus the freshly-issued bearer tokens.
typedef AuthPayload = ({UserDto user, AuthTokensDto tokens});

/// Raw HTTP calls. Throws [DioException] on failure (mapped to a Failure in the
/// repository). The dio interceptors add `X-Auth-Mode: bearer` automatically.
class AuthRemoteDataSource {
  AuthRemoteDataSource(this._dio);
  final Dio _dio;

  Future<AuthPayload> login(String email, String password) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/auth/login',
      data: {'email': email, 'password': password},
    );
    return _parseAuth(res.data!);
  }

  Future<AuthPayload> register(
    String email,
    String password,
    String name,
  ) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/auth/register',
      data: {'email': email, 'password': password, 'name': name},
    );
    return _parseAuth(res.data!);
  }

  Future<void> logout(String refreshToken) =>
      _dio.post<void>('/auth/logout', data: {'refresh_token': refreshToken});

  Future<UserDto> me() async {
    final res = await _dio.get<Map<String, dynamic>>('/me');
    return UserDto.fromJson(res.data!);
  }

  AuthPayload _parseAuth(Map<String, dynamic> data) => (
    user: UserDto.fromJson(data['user'] as Map<String, dynamic>),
    tokens: AuthTokensDto.fromJson(data['tokens'] as Map<String, dynamic>),
  );
}
