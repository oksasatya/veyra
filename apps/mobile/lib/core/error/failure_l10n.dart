import 'package:veyra_mobile/core/error/failure.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Localize a [Failure] for display. Prefers the stable backend error `code`
/// (the i18n contract — ADR-0008); falls back to the failure TYPE when no code
/// applies. The English `Failure.message` is never shown to the user.
String localizedFailure(AppLocalizations l10n, Failure failure) {
  final code = failure.code;
  final byCode = code == null ? null : _messageForCode(l10n, code);
  if (byCode != null) return byCode;

  return switch (failure) {
    NetworkFailure() => l10n.errorNetwork,
    ServerFailure() => l10n.errorServer,
    UnauthorizedFailure() => l10n.errorUnauthorized,
    NotFoundFailure() => l10n.errorNotFound,
    ConflictFailure() => l10n.errorConflict,
    ValidationFailure() => l10n.errorValidation,
  };
}

/// Map a backend error code to its localized message, or null if unmapped.
String? _messageForCode(AppLocalizations l10n, String code) => switch (code) {
  'INVALID_EMAIL' => l10n.errorInvalidEmail,
  'PASSWORD_TOO_SHORT' => l10n.errorPasswordTooShort,
  'EMAIL_ALREADY_EXISTS' => l10n.errorEmailAlreadyExists,
  'INVALID_LANGUAGE' => l10n.errorInvalidLanguage,
  'INVALID_PLATE_NUMBER' => l10n.errorInvalidPlateNumber,
  'ODOMETER_DECREASED' => l10n.errorOdometerDecreased,
  'INVALID_FUEL_TYPE' => l10n.errorInvalidFuelType,
  'INVALID_REMINDER_TYPE' => l10n.errorInvalidReminderType,
  'MISSING_DUE_DATE' => l10n.errorMissingDueDate,
  'MISSING_DUE_ODOMETER' => l10n.errorMissingDueOdometer,
  'INVALID_CATEGORY' => l10n.errorInvalidCategory,
  'INVALID_DOC_TYPE' => l10n.errorInvalidDocType,
  _ => null,
};
