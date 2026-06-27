import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// A validated, normalized (trimmed + lowercased) email address.
class Email {
  const Email._(this.value);
  final String value;

  static Either<ValidationFailure, Email> create(String raw) {
    final v = raw.trim().toLowerCase();
    if (v.isEmpty || !v.contains('@')) {
      return const Left(
        ValidationFailure('Enter a valid email address.', field: 'email'),
      );
    }
    final parts = v.split('@');
    if (parts.length != 2 || parts.first.isEmpty || !parts.last.contains('.')) {
      return const Left(
        ValidationFailure('Enter a valid email address.', field: 'email'),
      );
    }
    return Right(Email._(v));
  }
}
