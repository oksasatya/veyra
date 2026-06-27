import 'package:dio/dio.dart';
import 'package:veyra_mobile/core/storage/token_store.dart';

const _authPaths = {
  '/auth/register',
  '/auth/login',
  '/auth/refresh',
  '/auth/logout',
};

/// Attaches `Authorization: Bearer <access>` to protected requests and
/// `X-Auth-Mode: bearer` to the four auth endpoints.
class AuthInterceptor extends Interceptor {
  AuthInterceptor(this._store);
  final TokenStore _store;

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    if (_authPaths.contains(options.path)) {
      options.headers['X-Auth-Mode'] = 'bearer';
    } else {
      final tokens = await _store.read();
      if (tokens != null) {
        options.headers['Authorization'] = 'Bearer ${tokens.access}';
      }
    }
    handler.next(options);
  }
}
