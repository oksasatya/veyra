import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// A non-empty, trimmed + upper-cased plate number.
class PlateNumber {
  const PlateNumber._(this.value);
  final String value;

  static Either<ValidationFailure, PlateNumber> create(String raw) {
    final v = raw.trim().toUpperCase();
    if (v.isEmpty) {
      return const Left(
        ValidationFailure('Enter a plate number.', field: 'plate'),
      );
    }
    return Right(PlateNumber._(v));
  }
}
