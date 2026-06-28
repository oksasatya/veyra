import 'package:flutter_test/flutter_test.dart';
import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';

void main() {
  group('DocType', () {
    test('maps known api strings', () {
      expect(DocType.fromApi('stnk'), DocType.stnk);
      expect(DocType.fromApi('bpkb'), DocType.bpkb);
      expect(DocType.fromApi('insurance'), DocType.insurance);
      expect(DocType.fromApi('other'), DocType.other);
    });

    test('falls back to other for unknown', () {
      expect(DocType.fromApi('passport'), DocType.other);
    });

    test('apiValue round-trips the wire string', () {
      expect(DocType.stnk.apiValue, 'stnk');
      expect(DocType.insurance.apiValue, 'insurance');
    });

    test('label is human-readable', () {
      expect(DocType.stnk.label, 'STNK');
      expect(DocType.bpkb.label, 'BPKB');
      expect(DocType.insurance.label, 'Insurance');
      expect(DocType.other.label, 'Other');
    });
  });
}
