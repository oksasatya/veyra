/// Domain entity — pure Dart, no JSON. The data layer maps a DTO into this.
class User {
  const User({required this.id, required this.email, required this.name});
  final String id;
  final String email;
  final String name;
}
