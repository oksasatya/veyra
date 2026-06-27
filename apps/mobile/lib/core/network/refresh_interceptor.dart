import 'package:dio/dio.dart';
import 'package:veyra_mobile/core/storage/token_store.dart';

/// On 401, performs ONE `POST /auth/refresh` even under concurrent failures
/// (single-flight via a shared Future), persists the rotated tokens, and retries
/// the original request. If refresh fails, clears tokens and signals expiry.
class RefreshInterceptor extends Interceptor {
  RefreshInterceptor({
    required this.dio,
    required this.store,
    required this.onSessionExpired,
  });

  final Dio dio;
  final TokenStore store;
  final void Function() onSessionExpired;

  Future<bool>? _inFlight;

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    final is401 = err.response?.statusCode == 401;
    final isRefreshCall = err.requestOptions.path == '/auth/refresh';
    if (!is401 || isRefreshCall) {
      handler.next(err);
      return;
    }

    final ok = await (_inFlight ??= _refresh());
    _inFlight = null;
    if (!ok) {
      onSessionExpired();
      handler.next(err);
      return;
    }

    try {
      final tokens = await store.read();
      final req = err.requestOptions;
      req.headers['Authorization'] = 'Bearer ${tokens!.access}';
      final response = await dio.fetch<dynamic>(req);
      handler.resolve(response);
    } on DioException catch (e) {
      handler.next(e);
    }
  }

  Future<bool> _refresh() async {
    final tokens = await store.read();
    if (tokens == null) return false;
    try {
      final res = await dio.post<Map<String, dynamic>>(
        '/auth/refresh',
        data: {'refresh_token': tokens.refresh},
        options: Options(headers: {'X-Auth-Mode': 'bearer'}),
      );
      final t = res.data!['tokens'] as Map<String, dynamic>;
      await store.save(
        Tokens(
          access: t['access_token'] as String,
          refresh: t['refresh_token'] as String,
        ),
      );
      return true;
    } on DioException {
      await store.clear();
      return false;
    }
  }
}
