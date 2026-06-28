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

  @override
  String get authWelcomeBack => 'Welcome back';

  @override
  String get authSubtitle =>
      'Sign in to track your vehicles, fuel, and services.';

  @override
  String get authEmailHint => 'Email';

  @override
  String get authEmailLabel => 'Email';

  @override
  String get authPasswordHint => 'Password';

  @override
  String get authPasswordLabel => 'Password';

  @override
  String get authShow => 'Show';

  @override
  String get authHide => 'Hide';

  @override
  String get authForgotPassword => 'Forgot password?';

  @override
  String get authLogIn => 'Log in';

  @override
  String get authNewToVeyra => 'New to Veyra? ';

  @override
  String get authCreateAccount => 'Create account';

  @override
  String get authCreateTitle => 'Create your account';

  @override
  String get authCreateSubtitle =>
      'One account keeps every vehicle, log, and reminder in sync.';

  @override
  String get authNameLabel => 'Name';

  @override
  String get authNameHint => 'Your name';

  @override
  String get authPasswordHelp => 'Use 8 or more characters.';

  @override
  String get authAlreadyHaveAccount => 'Already have an account? Log in';

  @override
  String get authEnterName => 'Enter your name.';

  @override
  String get commonRetry => 'Retry';

  @override
  String get commonServices => 'Services';

  @override
  String get commonExpenses => 'Expenses';

  @override
  String get garageTitle => 'Garage';

  @override
  String get garageNavGarage => 'Garage';

  @override
  String get garageNavReminders => 'Reminders';

  @override
  String get garageNavDocs => 'Docs';

  @override
  String get garageNavSettings => 'Settings';

  @override
  String get garageOverviewVehicles => 'Vehicles';

  @override
  String get garageOverviewDueSoon => 'Due soon';

  @override
  String get garageOverviewSpent => 'Spent';

  @override
  String get garageCardFuel => 'Fuel';

  @override
  String garageCardOdometer(String value) {
    return 'Odometer $value km';
  }

  @override
  String garageDueBadge(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count due',
      one: '1 due',
    );
    return '$_temp0';
  }

  @override
  String get garageAddVehicle => 'Add vehicle';

  @override
  String get garageEmptyTitle => 'Add your first vehicle';

  @override
  String get garageEmptyBody =>
      'Track services, fuel, expenses, and reminders once your car or bike is in the garage.';

  @override
  String get garageErrorTitle => 'Can\'t reach Veyra';

  @override
  String get garageErrorBody =>
      'We couldn\'t load your garage. Check your connection, then try again.';

  @override
  String get garageErrorTryAgain => 'Try again';

  @override
  String get garageErrorCheckSettings => 'Check server settings';

  @override
  String get settingsLogOut => 'Log out';

  @override
  String get vehicleDetailTabOverview => 'Overview';

  @override
  String get vehicleDetailTabFuel => 'Fuel';

  @override
  String get vehicleDetailTabService => 'Service';

  @override
  String get vehicleDetailTabExpenses => 'Expenses';

  @override
  String get vehicleDetailTabDocs => 'Docs';

  @override
  String get vehicleDetailStatServices => 'Services';

  @override
  String get vehicleDetailStatServiceCost => 'Service cost';

  @override
  String get vehicleDetailStatRefuels => 'Refuels';

  @override
  String get vehicleDetailStatFuelCost => 'Fuel cost';

  @override
  String get vehicleDetailOdometerLabel => 'Odometer';

  @override
  String get vehicleDetailDueSoon => 'Due soon';

  @override
  String vehicleDetailReminders(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count reminders',
      one: '1 reminder',
    );
    return '$_temp0';
  }

  @override
  String get vehicleDetailAddFuel => 'Log fuel';

  @override
  String get vehicleDetailAddService => 'Add service';

  @override
  String get vehicleDetailAddExpense => 'Add expense';

  @override
  String get vehicleDetailAddDocument => 'Add document';

  @override
  String get vehicleDetailActivityHint =>
      'Open the Fuel, Service, Expenses, or Docs tab to see entries.';

  @override
  String get vehicleDetailErrorRetry => 'Retry';

  @override
  String get vehicleAddTitle => 'Add vehicle';

  @override
  String get vehicleAddFieldBrand => 'Brand';

  @override
  String get vehicleAddFieldModel => 'Model';

  @override
  String get vehicleAddFieldYear => 'Year';

  @override
  String get vehicleAddFieldPlate => 'Plate number';

  @override
  String get vehicleAddFieldFuelType => 'Fuel type';

  @override
  String get vehicleAddFieldOdometer => 'Odometer (km)';

  @override
  String get vehicleAddFieldColor => 'Color (optional)';

  @override
  String get vehicleAddSave => 'Save vehicle';

  @override
  String get vehicleAddErrorBrandModel => 'Enter the brand and model.';

  @override
  String get vehicleAddErrorYear => 'Enter a valid year.';

  @override
  String get vehicleAddErrorPlate => 'Enter a plate number.';

  @override
  String get vehicleAddErrorOdometer => 'Enter a valid odometer reading.';

  @override
  String get fuelTypePetrol => 'Petrol';

  @override
  String get fuelTypeDiesel => 'Diesel';

  @override
  String get fuelTypeElectric => 'Electric';

  @override
  String get fuelTypeHybrid => 'Hybrid';

  @override
  String get reminderTitle => 'Reminders';

  @override
  String get reminderSectionOverdue => 'Overdue';

  @override
  String get reminderSectionDueSoon => 'Due Soon';

  @override
  String get reminderSectionUpcoming => 'Upcoming';

  @override
  String get reminderSectionCompleted => 'Completed';

  @override
  String get reminderMarkComplete => 'Mark complete';

  @override
  String reminderDaysLate(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count days late',
      one: '1 day late',
    );
    return '$_temp0';
  }

  @override
  String reminderDaysUntil(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'in $count days',
      one: 'in 1 day',
    );
    return '$_temp0';
  }

  @override
  String reminderAtKm(int km) {
    return 'at $km km';
  }

  @override
  String reminderMetaVehicleDate(String vehicle, String date) {
    return '$vehicle · due $date';
  }

  @override
  String reminderMetaVehicleKm(String vehicle, int km) {
    return '$vehicle · at $km km';
  }

  @override
  String get reminderEmpty => 'No reminders yet';

  @override
  String get reminderError => 'Could not load reminders';

  @override
  String get reminderTypeDate => 'Date';

  @override
  String get reminderTypeOdometer => 'Odometer';

  @override
  String get reminderTypeBoth => 'Date & Odometer';

  @override
  String get reminderFormTitle => 'Add Reminder';

  @override
  String get reminderFormTitleLabel => 'Title';

  @override
  String get reminderFormTitleHint => 'Road tax renewal';

  @override
  String get reminderFormTrigger => 'Trigger';

  @override
  String get reminderFormNotes => 'Notes (optional)';

  @override
  String get reminderFormNotesHint => 'Anything to note';

  @override
  String get reminderFormPickDate => 'Pick a date';

  @override
  String get reminderFormDueDate => 'Due Date';

  @override
  String get reminderFormDueOdometer => 'Due Odometer (km)';

  @override
  String get reminderFormSave => 'Save';

  @override
  String get reminderAdd => 'Add reminder';

  @override
  String get reminderPickVehicle => 'Choose a vehicle';

  @override
  String get commonTryAgain => 'Try again';

  @override
  String get documentTitle => 'Documents';

  @override
  String get documentAllVehicles => 'All vehicles';

  @override
  String documentCountAcrossVehicles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count across all vehicles',
      one: '1 across all vehicles',
    );
    return '$_temp0';
  }

  @override
  String get documentErrorTitle => 'Can\'t load documents';

  @override
  String get documentEmptyTitle => 'No documents yet';

  @override
  String get documentEmptyBody =>
      'No documents yet. Add STNK, BPKB, insurance, and more from a vehicle.';

  @override
  String get documentEmptyBodyDetail =>
      'Add the STNK, BPKB, insurance, or any document you want to keep with this vehicle.';

  @override
  String get documentStatusExpired => 'Expired';

  @override
  String get documentStatusExpiringSoon => 'Expiring soon';

  @override
  String get documentStatusValid => 'Valid';

  @override
  String get documentStatusOnFile => 'On file';

  @override
  String documentDaysLeft(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count days left',
      one: '1 day left',
    );
    return '$_temp0';
  }

  @override
  String get documentNoExpiry => 'No expiry';

  @override
  String documentValidUntil(String date) {
    return 'Valid until $date';
  }

  @override
  String documentExpires(String date) {
    return 'Expires $date';
  }

  @override
  String documentNoExpiryMeta(String docType) {
    return 'No expiry · $docType';
  }

  @override
  String documentExpiresMeta(String date) {
    return 'Expires $date';
  }

  @override
  String get documentAddTitle => 'Add document';

  @override
  String get documentFieldType => 'Type';

  @override
  String get documentFieldTitle => 'Title';

  @override
  String get documentFieldTitleHint => 'STNK';

  @override
  String get documentFieldExpiry => 'Expiry date (optional)';

  @override
  String get documentFieldFileUrl => 'File URL (optional)';

  @override
  String get documentFieldFileUrlHint => 'https://…';

  @override
  String get documentFieldNotes => 'Notes (optional)';

  @override
  String get documentFieldNotesHint => 'Anything to note';

  @override
  String get documentSave => 'Save document';

  @override
  String get documentErrorEnterTitle => 'Enter a document title.';

  @override
  String get docTypeStnk => 'STNK';

  @override
  String get docTypeBpkb => 'BPKB';

  @override
  String get docTypeInsurance => 'Insurance';

  @override
  String get docTypeOther => 'Other';

  @override
  String get fuelLogTitle => 'Log fuel';

  @override
  String get fuelLogFieldDate => 'Date';

  @override
  String get fuelLogFieldOdometer => 'Odometer';

  @override
  String get fuelLogFieldLiters => 'Liters';

  @override
  String get fuelLogFieldPricePerLiter => 'Price / liter';

  @override
  String get fuelLogFieldStation => 'Station (optional)';

  @override
  String get fuelLogFieldTotalCost => 'Total cost';

  @override
  String get fuelLogFieldFullTank => 'Full tank';

  @override
  String get fuelLogFieldFullTankHint => 'Used to compute consumption';

  @override
  String get fuelLogSave => 'Save fuel log';

  @override
  String get fuelLogErrorOdometer => 'Enter a valid odometer reading.';

  @override
  String get fuelLogErrorLiters => 'Enter the liters filled.';

  @override
  String get fuelLogErrorPricePerLiter => 'Enter the price per liter.';

  @override
  String get fuelLogEmpty =>
      'No fuel logs yet. Tap \"Log fuel\" to add your first fill-up.';

  @override
  String get fuelLogLoadError => 'Could not load fuel logs.';

  @override
  String get serviceRecordTitle => 'Log service';

  @override
  String get serviceRecordFieldDate => 'Date';

  @override
  String get serviceRecordFieldOdometer => 'Odometer (km)';

  @override
  String get serviceRecordFieldDescription => 'Description';

  @override
  String get serviceRecordFieldWorkshop => 'Workshop (optional)';

  @override
  String get serviceRecordFieldCost => 'Cost (optional)';

  @override
  String get serviceRecordFieldNotes => 'Notes (optional)';

  @override
  String get serviceRecordSave => 'Save service';

  @override
  String get serviceRecordErrorDescription => 'Enter a description.';

  @override
  String get serviceRecordErrorOdometer => 'Enter a valid odometer reading.';

  @override
  String get serviceRecordErrorCost => 'Enter a valid cost.';

  @override
  String get serviceRecordEmpty => 'No service records yet. Log the first one.';

  @override
  String get serviceRecordLoadError => 'Could not load service records.';

  @override
  String get expenseAddTitle => 'Add expense';

  @override
  String get expenseCategoryBattery => 'Battery';

  @override
  String get expenseCategoryInsurance => 'Insurance';

  @override
  String get expenseCategoryOther => 'Other';

  @override
  String get expenseCategoryTax => 'Tax';

  @override
  String get expenseCategoryTire => 'Tire';

  @override
  String get expenseEmpty => 'No expenses yet. Add your first one.';

  @override
  String get expenseErrorEnterDescription => 'Enter a description.';

  @override
  String get expenseErrorInvalidAmount => 'Enter a valid amount.';

  @override
  String get expenseFieldAmount => 'Amount (Rp)';

  @override
  String get expenseFieldCategory => 'Category';

  @override
  String get expenseFieldDate => 'Date';

  @override
  String get expenseFieldDescription => 'Description';

  @override
  String get expenseFieldDescriptionHint => 'Annual premium';

  @override
  String get expenseLoadError => 'Could not load expenses.';

  @override
  String get expenseSave => 'Save expense';
}
