import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/document/data/models/document_dto.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';

void main() {
  group('DocumentDto.fromJson → toDomain', () {
    test('maps a full row with expiry and file url', () {
      final dto = DocumentDto.fromJson({
        'id': 'doc-1',
        'vehicle_id': 'veh-1',
        'doc_type': 'insurance',
        'title': 'Insurance policy',
        'expiry_date': '2026-07-05',
        'file_url': 'https://files.veyra.dev/policy.pdf',
      });
      final doc = dto.toDomain();

      expect(doc.id, 'doc-1');
      expect(doc.vehicleId, 'veh-1');
      expect(doc.docType, DocType.insurance);
      expect(doc.title, 'Insurance policy');
      expect(doc.expiryDate, DateTime.parse('2026-07-05'));
      expect(doc.fileUrl, 'https://files.veyra.dev/policy.pdf');
    });

    test('maps a row with null expiry and null file url', () {
      final dto = DocumentDto.fromJson({
        'id': 'doc-2',
        'vehicle_id': 'veh-1',
        'doc_type': 'bpkb',
        'title': 'BPKB',
        'expiry_date': null,
        'file_url': null,
      });
      final doc = dto.toDomain();

      expect(doc.docType, DocType.bpkb);
      expect(doc.expiryDate, isNull);
      expect(doc.fileUrl, isNull);
    });

    test('maps a row with omitted optional keys', () {
      final dto = DocumentDto.fromJson({
        'id': 'doc-3',
        'vehicle_id': 'veh-1',
        'doc_type': 'stnk',
        'title': 'STNK',
      });
      final doc = dto.toDomain();

      expect(doc.docType, DocType.stnk);
      expect(doc.expiryDate, isNull);
      expect(doc.fileUrl, isNull);
    });

    test('unknown doc_type falls back to other', () {
      final dto = DocumentDto.fromJson({
        'id': 'doc-4',
        'vehicle_id': 'veh-1',
        'doc_type': 'passport',
        'title': 'Driver passport',
      });

      expect(dto.toDomain().docType, DocType.other);
    });
  });
}
