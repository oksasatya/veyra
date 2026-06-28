import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Runtime configuration. Override the base URL at build time:
/// `flutter run --dart-define=API_BASE_URL=https://api.veyra.dev`.
class AppConfig {
  const AppConfig({required this.apiBaseUrl});

  factory AppConfig.fromEnv() => const AppConfig(
    apiBaseUrl: String.fromEnvironment(
      'API_BASE_URL',
      defaultValue: 'http://localhost:8080',
    ),
  );

  final String apiBaseUrl;
}

final appConfigProvider = Provider<AppConfig>((ref) => AppConfig.fromEnv());
