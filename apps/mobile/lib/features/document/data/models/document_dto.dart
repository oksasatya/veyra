import 'package:veyra_mobile/features/document/domain/entities/document.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';

/// Wire model for `DocumentResponse`. Hand-mapped to the [Document] entity.
class DocumentDto {
  const DocumentDto({
    required this.id,
    required this.vehicleId,
    required this.docType,
    required this.title,
    this.expiryDate,
    this.fileUrl,
  });

  factory DocumentDto.fromJson(Map<String, dynamic> json) => DocumentDto(
    id: json['id'] as String,
    vehicleId: json['vehicle_id'] as String,
    docType: json['doc_type'] as String,
    title: json['title'] as String,
    expiryDate: json['expiry_date'] as String?,
    fileUrl: json['file_url'] as String?,
  );

  final String id;
  final String vehicleId;
  final String docType;
  final String title;
  final String? expiryDate;
  final String? fileUrl;

  Document toDomain() => Document(
    id: id,
    vehicleId: vehicleId,
    docType: DocType.fromApi(docType),
    title: title,
    expiryDate: expiryDate == null ? null : DateTime.parse(expiryDate!),
    fileUrl: fileUrl,
  );
}
