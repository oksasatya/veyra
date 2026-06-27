import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// A password that satisfies the minimum-length policy (8 characters).
class Password {
  const Password._(this.value);
  final String value;

  static Either<ValidationFailure, Password> create(String raw) {
    if (raw.length < 8) {
      return const Left(
        ValidationFailure(
          'Password must be at least 8 characters.',
          field: 'password',
        ),
      );
    }
    return Right(Password._(raw));
  }
}
