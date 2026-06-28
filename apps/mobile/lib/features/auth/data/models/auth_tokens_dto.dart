import 'package:veyra_mobile/core/storage/token_store.dart';

/// Bearer-mode `tokens` payload. `refresh_token` is the opaque
/// `{family_id}.{raw_secret}` string, stored verbatim.
class AuthTokensDto {
  const AuthTokensDto({required this.accessToken, required this.refreshToken});

  factory AuthTokensDto.fromJson(Map<String, dynamic> json) => AuthTokensDto(
    accessToken: json['access_token'] as String,
    refreshToken: json['refresh_token'] as String,
  );

  final String accessToken;
  final String refreshToken;

  Tokens toStore() => Tokens(access: accessToken, refresh: refreshToken);
}
