import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// A non-negative odometer reading (kilometres).
class Odometer {
  const Odometer._(this.value);
  final int value;

  static Either<ValidationFailure, Odometer> create(int raw) {
    if (raw < 0) {
      return const Left(
        ValidationFailure('Odometer cannot be negative.', field: 'odometer'),
      );
    }
    return Right(Odometer._(raw));
  }
}
