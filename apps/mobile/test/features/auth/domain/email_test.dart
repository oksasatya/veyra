import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';

void main() {
  test('valid email accepted and lowercased', () {
    final result = Email.create('Alice@Example.COM');
    expect(result.isRight(), isTrue);
    expect(result.toNullable()?.value, 'alice@example.com');
  });

  test('email without @ rejected', () {
    expect(Email.create('nope').isLeft(), isTrue);
  });

  test('empty email rejected', () {
    expect(Email.create('').isLeft(), isTrue);
  });

  test('email without domain dot rejected', () {
    expect(Email.create('a@nodot').isLeft(), isTrue);
  });
}
