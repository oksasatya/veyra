import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

void main() {
  test('password of 8+ chars accepted', () {
    expect(Password.create('password123').isRight(), isTrue);
  });

  test('password under 8 chars rejected', () {
    expect(Password.create('short').isLeft(), isTrue);
  });
}
