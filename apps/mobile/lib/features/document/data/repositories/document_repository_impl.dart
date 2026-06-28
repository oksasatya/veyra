import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fpdart/fpdart.dart';
import 'package:veyra_mobile/core/error/dio_error_mapper.dart';
import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/core/network/dio_client.dart';
import 'package:veyra_mobile/features/document/data/datasources/document_remote_data_source.dart';
import 'package:veyra_mobile/features/document/domain/entities/document.dart';
import 'package:veyra_mobile/features/document/domain/repositories/document_repository.dart';
import 'package:veyra_mobile/features/document/domain/usecases/create_document_usecase.dart';
import 'package:veyra_mobile/features/document/domain/usecases/list_documents_usecase.dart';

class DocumentRepositoryImpl implements DocumentRepository {
  DocumentRepositoryImpl(this.remote);
  final DocumentRemoteDataSource remote;

  @override
  Future<Either<Failure, List<Document>>> list(String vehicleId) async {
    try {
      final dtos = await remote.list(vehicleId);
      return Right(dtos.map((d) => d.toDomain()).toList());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  @override
  Future<Either<Failure, Document>> create(CreateDocumentInput input) async {
    try {
      final dto = await remote.create(input.vehicleId, {
        'doc_type': input.docType.apiValue,
        'title': input.title,
        'expiry_date': _isoDate(input.expiryDate),
        'file_url': input.fileUrl,
        'notes': input.notes,
      });
      return Right(dto.toDomain());
    } on DioException catch (e) {
      return Left(mapDioError(e));
    }
  }

  /// Date-only ISO string (`YYYY-MM-DD`) the backend expects, or null.
  String? _isoDate(DateTime? date) {
    if (date == null) return null;
    final m = date.month.toString().padLeft(2, '0');
    final d = date.day.toString().padLeft(2, '0');
    return '${date.year}-$m-$d';
  }
}

// ── Providers (DI) ───────────────────────────────────────────────────────────

final documentRepositoryProvider = Provider<DocumentRepository>(
  (ref) =>
      DocumentRepositoryImpl(DocumentRemoteDataSource(ref.watch(dioProvider))),
);

final listDocumentsUseCaseProvider = Provider<ListDocumentsUseCase>(
  (ref) => ListDocumentsUseCase(ref.watch(documentRepositoryProvider)),
);

final createDocumentUseCaseProvider = Provider<CreateDocumentUseCase>(
  (ref) => CreateDocumentUseCase(ref.watch(documentRepositoryProvider)),
);

/// Per-vehicle documents, keyed by vehicle id. A thrown [Failure] surfaces as
/// the `AsyncValue` error state for the UI. Creation goes through
/// [CreateDocumentUseCase] directly from the sheet, which invalidates this on
/// success to refetch.
final documentListProvider = FutureProvider.family<List<Document>, String>((
  ref,
  vehicleId,
) async {
  final result = await ref.read(listDocumentsUseCaseProvider)(vehicleId);
  return result.fold((failure) => throw failure, (docs) => docs);
});
