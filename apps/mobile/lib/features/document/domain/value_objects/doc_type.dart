/// Document type — mirrors the backend's `'stnk' | 'bpkb' | 'insurance'
/// | 'other'` string. `name` is the wire value (`stnk`, …).
enum DocType {
  stnk,
  bpkb,
  insurance,
  other;

  static DocType fromApi(String raw) => switch (raw) {
    'stnk' => DocType.stnk,
    'bpkb' => DocType.bpkb,
    'insurance' => DocType.insurance,
    _ => DocType.other,
  };

  String get apiValue => name;

  String get label => switch (this) {
    DocType.stnk => 'STNK',
    DocType.bpkb => 'BPKB',
    DocType.insurance => 'Insurance',
    DocType.other => 'Other',
  };
}
