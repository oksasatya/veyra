/// Domain entity — pure Dart, no JSON. The data layer maps a DTO into this.
class User {
  const User({
    required this.id,
    required this.email,
    required this.name,
    this.preferredLanguage,
  });
  final String id;
  final String email;
  final String name;

  /// ISO 639-1 language code from the server (e.g. `"en"`, `"id"`).
  /// Null when the backend has not set a preference yet.
  final String? preferredLanguage;
}
