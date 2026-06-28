import 'package:decimal/decimal.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';

/// A strictly-positive [Decimal] parsed from user input. Reused for liters and
/// price-per-liter, where zero or negative makes no sense.
class PositiveDecimal {
  const PositiveDecimal._(this.value);
  final Decimal value;

  static Either<ValidationFailure, Decimal> create(
    String raw, {
    String field = 'amount',
  }) {
    final parsed = Decimal.tryParse(raw.trim());
    if (parsed == null) {
      return Left(ValidationFailure('Enter a valid number.', field: field));
    }
    if (parsed <= Decimal.zero) {
      return Left(
        ValidationFailure('Must be greater than zero.', field: field),
      );
    }
    return Right(parsed);
  }
}
