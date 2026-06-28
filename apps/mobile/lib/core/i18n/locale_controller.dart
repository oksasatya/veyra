import 'dart:async' show unawaited;
import 'dart:ui' show Locale;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Language codes the app ships translations for (mirrors the ARB files).
const supportedLanguageCodes = {'en', 'id'};

const _kLocaleKey = 'veyra_locale';

/// Holds the active language override. `null` means "follow the device locale"
/// (`MaterialApp.locale = null` → system). The override is persisted locally so
/// it survives restarts.
///
/// This controller stays free of any feature dependency — syncing the choice to
/// the backend (`PATCH /me`) is the settings layer's responsibility, keeping
/// `core` decoupled from `auth` (mirrors `auth_events`).
class LocaleController extends Notifier<Locale?> {
  final FlutterSecureStorage _storage = const FlutterSecureStorage();

  @override
  Locale? build() {
    unawaited(_restore());
    return null;
  }

  Future<void> _restore() async {
    final code = await _storage.read(key: _kLocaleKey);
    if (code != null && supportedLanguageCodes.contains(code)) {
      state = Locale(code);
    }
  }

  /// Override the language (or pass `null` to follow the device locale).
  /// Persists the choice locally.
  Future<void> setLocale(Locale? locale) async {
    state = locale;
    if (locale == null) {
      await _storage.delete(key: _kLocaleKey);
    } else {
      await _storage.write(key: _kLocaleKey, value: locale.languageCode);
    }
  }
}

final localeControllerProvider = NotifierProvider<LocaleController, Locale?>(
  LocaleController.new,
);
