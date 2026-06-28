import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:veyra_mobile/core/config/app_config.dart';
import 'package:veyra_mobile/core/network/auth_events.dart';
import 'package:veyra_mobile/core/network/auth_interceptor.dart';
import 'package:veyra_mobile/core/network/refresh_interceptor.dart';
import 'package:veyra_mobile/core/storage/token_store.dart';

final dioProvider = Provider<Dio>((ref) {
  final config = ref.watch(appConfigProvider);
  final store = ref.watch(tokenStoreProvider);

  final dio = Dio(
    BaseOptions(
      baseUrl: config.apiBaseUrl,
      connectTimeout: const Duration(seconds: 10),
      receiveTimeout: const Duration(seconds: 10),
      contentType: 'application/json',
    ),
  )..interceptors.add(AuthInterceptor(store));

  dio.interceptors.add(
    RefreshInterceptor(
      dio: dio,
      store: store,
      onSessionExpired: () =>
          ref.read(authEventsProvider.notifier).sessionExpired(),
    ),
  );

  return dio;
});
