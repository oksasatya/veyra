import 'dart:io' show Platform;

import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Runtime configuration. Override the base URL at build time:
/// `flutter run --dart-define=API_BASE_URL=https://api.veyra.dev`.
class AppConfig {
  const AppConfig({required this.apiBaseUrl});

  factory AppConfig.fromEnv() {
    const fromDefine = String.fromEnvironment('API_BASE_URL');
    if (fromDefine.isNotEmpty) return const AppConfig(apiBaseUrl: fromDefine);
    // The Android emulator can't reach the host's `localhost` — 10.0.2.2 is its
    // alias for the host loopback. iOS simulator shares the host network.
    final host = Platform.isAndroid ? '10.0.2.2' : 'localhost';
    return AppConfig(apiBaseUrl: 'http://$host:8080');
  }

  final String apiBaseUrl;
}

final appConfigProvider = Provider<AppConfig>((ref) => AppConfig.fromEnv());
