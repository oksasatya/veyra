import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/document/domain/entities/document.dart';
import 'package:veyra_mobile/features/document/domain/repositories/document_repository.dart';

class CreateDocumentUseCase {
  const CreateDocumentUseCase(this._repo);
  final DocumentRepository _repo;

  Future<Either<Failure, Document>> call(CreateDocumentInput input) =>
      _repo.create(input);
}
