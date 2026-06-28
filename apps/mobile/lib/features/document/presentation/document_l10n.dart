import 'package:veyra_mobile/features/document/domain/value_objects/doc_type.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Map a [DocType] to its localized label.
String localizedDocType(AppLocalizations l10n, DocType type) => switch (type) {
  DocType.stnk => l10n.docTypeStnk,
  DocType.bpkb => l10n.docTypeBpkb,
  DocType.insurance => l10n.docTypeInsurance,
  DocType.other => l10n.docTypeOther,
};
