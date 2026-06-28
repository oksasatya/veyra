import 'package:veyra_mobile/features/auth/domain/entities/user.dart';

/// Wire model for the API user payload. Hand-mapped to the [User] domain entity.
class UserDto {
  const UserDto({
    required this.id,
    required this.email,
    required this.name,
    this.preferredLanguage,
  });

  factory UserDto.fromJson(Map<String, dynamic> json) => UserDto(
    id: json['id'] as String,
    email: json['email'] as String,
    name: json['name'] as String,
    preferredLanguage: json['preferred_language'] as String?,
  );

  final String id;
  final String email;
  final String name;
  final String? preferredLanguage;

  User toDomain() => User(
    id: id,
    email: email,
    name: name,
    preferredLanguage: preferredLanguage,
  );
}
