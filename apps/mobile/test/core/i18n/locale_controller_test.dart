import 'dart:ui' show Locale;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:veyra_mobile/core/i18n/locale_controller.dart';

class _MockStorage extends Mock implements FlutterSecureStorage {}

/// Builds a [ProviderContainer] backed by [storage] and returns both.
(ProviderContainer, _MockStorage) _makeContainer({_MockStorage? storage}) {
  final s = storage ?? _MockStorage();
  final container = ProviderContainer(
    overrides: [
      localeControllerProvider.overrideWith(() => LocaleController(storage: s)),
    ],
  );
  addTearDown(container.dispose);
  return (container, s);
}

void main() {
  // Silence "bad state: no element" from Riverpod's internal dispose checks
  // in test environments where the container is torn down early.
  setUpAll(() {
    // mocktail needs no fallback values here since we only stub read/write/delete.
  });

  group('adoptBackendLanguage', () {
    test('local override beats backend: persisted override wins', () async {
      final (container, storage) = _makeContainer();

      // Simulate a persisted local override ('en').
      when(() => storage.read(key: any(named: 'key')))
          .thenAnswer((_) async => 'en');

      // Let build() settle (restore reads the same stub → sets state to 'en').
      await Future<void>.delayed(Duration.zero);

      // Now try to adopt a different backend language.
      await container
          .read(localeControllerProvider.notifier)
          .adoptBackendLanguage('id');

      // State must remain 'en' (local override wins).
      expect(
        container.read(localeControllerProvider),
        const Locale('en'),
      );
    });

    test('adopt when no local override exists', () async {
      final (container, storage) = _makeContainer();

      // No persisted override.
      when(() => storage.read(key: any(named: 'key')))
          .thenAnswer((_) async => null);

      // Let build() settle (restore → null → state stays null).
      await Future<void>.delayed(Duration.zero);

      await container
          .read(localeControllerProvider.notifier)
          .adoptBackendLanguage('id');

      expect(
        container.read(localeControllerProvider),
        const Locale('id'),
      );
    });

    test('unsupported code is a no-op', () async {
      final (container, storage) = _makeContainer();

      when(() => storage.read(key: any(named: 'key')))
          .thenAnswer((_) async => null);

      await Future<void>.delayed(Duration.zero);

      await container
          .read(localeControllerProvider.notifier)
          .adoptBackendLanguage('fr'); // not in supportedLanguageCodes

      expect(container.read(localeControllerProvider), isNull);
    });
  });

  group('setLocale', () {
    test('persists the chosen locale', () async {
      final (container, storage) = _makeContainer();

      when(() => storage.read(key: any(named: 'key')))
          .thenAnswer((_) async => null);
      when(
        () => storage.write(
          key: any(named: 'key'),
          value: any(named: 'value'),
        ),
      ).thenAnswer((_) async {});

      await Future<void>.delayed(Duration.zero);

      await container
          .read(localeControllerProvider.notifier)
          .setLocale(const Locale('id'));

      verify(
        () => storage.write(key: 'veyra_locale', value: 'id'),
      ).called(1);
      expect(
        container.read(localeControllerProvider),
        const Locale('id'),
      );
    });

    test('clears storage when setting null (follow device)', () async {
      final (container, storage) = _makeContainer();

      when(() => storage.read(key: any(named: 'key')))
          .thenAnswer((_) async => null);
      when(() => storage.delete(key: any(named: 'key')))
          .thenAnswer((_) async {});

      await Future<void>.delayed(Duration.zero);

      await container
          .read(localeControllerProvider.notifier)
          .setLocale(null);

      verify(() => storage.delete(key: 'veyra_locale')).called(1);
      expect(container.read(localeControllerProvider), isNull);
    });
  });
}
