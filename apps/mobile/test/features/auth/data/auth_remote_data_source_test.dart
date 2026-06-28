import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:veyra_mobile/features/auth/data/datasources/auth_remote_data_source.dart';

class _MockDio extends Mock implements Dio {}

void main() {
  late _MockDio dio;
  late AuthRemoteDataSource dataSource;

  setUp(() {
    dio = _MockDio();
    dataSource = AuthRemoteDataSource(dio);
  });

  setUpAll(() {
    registerFallbackValue(<String, dynamic>{});
  });

  group('updatePreferences', () {
    test('calls PATCH /me with preferred_language', () async {
      when(
        () => dio.patch<void>(
          '/me',
          data: any(named: 'data'),
        ),
      ).thenAnswer(
        (_) async => Response<void>(
          requestOptions: RequestOptions(path: '/me'),
          statusCode: 200,
        ),
      );

      await dataSource.updatePreferences('id');

      verify(
        () => dio.patch<void>(
          '/me',
          data: {'preferred_language': 'id'},
        ),
      ).called(1);
    });
  });
}
