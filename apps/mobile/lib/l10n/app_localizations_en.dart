// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'Veyra';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get settingsLanguage => 'Language';

  @override
  String get languageSystem => 'System default';

  @override
  String get languageEnglish => 'English';

  @override
  String get languageIndonesian => 'Indonesian';

  @override
  String get errorNetwork => 'No connection. Check your network.';

  @override
  String get errorServer => 'Something went wrong on the server.';

  @override
  String get errorUnauthorized =>
      'Your session has expired. Please sign in again.';

  @override
  String get errorForbidden => 'You do not have access to this.';

  @override
  String get errorNotFound => 'Not found.';

  @override
  String get errorConflict => 'That already exists.';

  @override
  String get errorValidation => 'Please check your input.';

  @override
  String get errorServiceUnavailable =>
      'Service temporarily unavailable. Please try again.';

  @override
  String get errorInvalidEmail => 'Enter a valid email address.';

  @override
  String get errorPasswordTooShort => 'Password must be at least 8 characters.';

  @override
  String get errorEmailAlreadyExists => 'That email is already registered.';

  @override
  String get errorInvalidLanguage => 'Unsupported language.';

  @override
  String get errorInvalidPlateNumber => 'Enter a valid plate number.';

  @override
  String get errorOdometerDecreased =>
      'Odometer cannot be lower than the current value.';

  @override
  String get errorInvalidFuelType => 'Choose a valid fuel type.';

  @override
  String get errorInvalidReminderType => 'Choose a valid reminder type.';

  @override
  String get errorMissingDueDate => 'A due date is required for this reminder.';

  @override
  String get errorMissingDueOdometer =>
      'A due odometer is required for this reminder.';

  @override
  String get errorInvalidCategory => 'Choose a valid category.';

  @override
  String get errorInvalidDocType => 'Choose a valid document type.';
}
