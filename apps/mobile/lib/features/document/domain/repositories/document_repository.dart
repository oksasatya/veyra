import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/features/document/domain/entities/document.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';

/// Validated input for creating a document (the sheet builds this).
class CreateDocumentInput {
  const CreateDocumentInput({
    required this.vehicleId,
    required this.docType,
    required this.title,
    this.expiryDate,
    this.fileUrl,
    this.notes,
  });

  final String vehicleId;
  final DocType docType;
  final String title;
  final DateTime? expiryDate;
  final String? fileUrl;
  final String? notes;
}

/// Port: the document boundary the domain depends on.
abstract interface class DocumentRepository {
  Future<Either<Failure, List<Document>>> list(String vehicleId);
  Future<Either<Failure, Document>> create(CreateDocumentInput input);
}
