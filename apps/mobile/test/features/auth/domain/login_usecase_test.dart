import 'package:flutter_test/flutter_test.dart';
import 'package:fpdart/fpdart.dart';
import 'package:mocktail/mocktail.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/auth/domain/entities/user.dart';
import 'package:veyra_mobile/features/auth/domain/repositories/auth_repository.dart';
import 'package:veyra_mobile/features/auth/domain/usecases/login_usecase.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/email.dart';
import 'package:veyra_mobile/features/auth/domain/value_objects/password.dart';

class _MockRepo extends Mock implements AuthRepository {}

void main() {
  late _MockRepo repo;
  late LoginUseCase useCase;
  final email = Email.create('a@b.com').toNullable()!;
  final password = Password.create('password123').toNullable()!;

  setUp(() {
    repo = _MockRepo();
    useCase = LoginUseCase(repo);
  });

  test('returns the user on success', () async {
    const user = User(id: '1', email: 'a@b.com', name: 'A');
    when(() => repo.login(email: email, password: password))
        .thenAnswer((_) async => const Right(user));

    final result = await useCase(email, password);

    expect(result.toNullable(), user);
  });

  test('propagates a failure', () async {
    when(() => repo.login(email: email, password: password))
        .thenAnswer((_) async => const Left(UnauthorizedFailure()));

    final result = await useCase(email, password);

    expect(result.isLeft(), isTrue);
  });
}
