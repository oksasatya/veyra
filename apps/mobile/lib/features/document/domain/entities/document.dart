import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';

/// Domain entity — pure Dart. The data layer maps a DTO into this.
class Document {
  const Document({
    required this.id,
    required this.vehicleId,
    required this.docType,
    required this.title,
    this.expiryDate,
    this.fileUrl,
  });

  final String id;
  final String vehicleId;
  final DocType docType;
  final String title;
  final DateTime? expiryDate;
  final String? fileUrl;
}
