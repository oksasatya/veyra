/// Application-facing failures. Use cases return `Either<Failure, T>`; the UI
/// renders these, never a raw transport error. Implements [Exception] so it can
/// be thrown into an `AsyncValue` error state (satisfies `only_throw_errors`).
sealed class Failure implements Exception {
  const Failure(this.message);
  final String message;
}

class NetworkFailure extends Failure {
  const NetworkFailure([super.message = 'No connection. Check your network.']);
}

class ServerFailure extends Failure {
  const ServerFailure([super.message = 'Something went wrong on the server.']);
}

class UnauthorizedFailure extends Failure {
  const UnauthorizedFailure([super.message = 'Your session has expired.']);
}

class NotFoundFailure extends Failure {
  const NotFoundFailure([super.message = 'Not found.']);
}

class ConflictFailure extends Failure {
  const ConflictFailure([super.message = 'That already exists.']);
}

class ValidationFailure extends Failure {
  const ValidationFailure(super.message, {this.field});
  final String? field;
}
