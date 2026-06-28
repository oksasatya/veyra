import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_id.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('id'),
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'Veyra'**
  String get appTitle;

  /// No description provided for @settingsTitle.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// No description provided for @settingsLanguage.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get settingsLanguage;

  /// No description provided for @languageSystem.
  ///
  /// In en, this message translates to:
  /// **'System default'**
  String get languageSystem;

  /// No description provided for @languageEnglish.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get languageEnglish;

  /// No description provided for @languageIndonesian.
  ///
  /// In en, this message translates to:
  /// **'Indonesian'**
  String get languageIndonesian;

  /// No description provided for @errorNetwork.
  ///
  /// In en, this message translates to:
  /// **'No connection. Check your network.'**
  String get errorNetwork;

  /// No description provided for @errorServer.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong on the server.'**
  String get errorServer;

  /// No description provided for @errorUnauthorized.
  ///
  /// In en, this message translates to:
  /// **'Your session has expired. Please sign in again.'**
  String get errorUnauthorized;

  /// No description provided for @errorForbidden.
  ///
  /// In en, this message translates to:
  /// **'You do not have access to this.'**
  String get errorForbidden;

  /// No description provided for @errorNotFound.
  ///
  /// In en, this message translates to:
  /// **'Not found.'**
  String get errorNotFound;

  /// No description provided for @errorConflict.
  ///
  /// In en, this message translates to:
  /// **'That already exists.'**
  String get errorConflict;

  /// No description provided for @errorValidation.
  ///
  /// In en, this message translates to:
  /// **'Please check your input.'**
  String get errorValidation;

  /// No description provided for @errorServiceUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Service temporarily unavailable. Please try again.'**
  String get errorServiceUnavailable;

  /// No description provided for @errorInvalidEmail.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid email address.'**
  String get errorInvalidEmail;

  /// No description provided for @errorPasswordTooShort.
  ///
  /// In en, this message translates to:
  /// **'Password must be at least 8 characters.'**
  String get errorPasswordTooShort;

  /// No description provided for @errorEmailAlreadyExists.
  ///
  /// In en, this message translates to:
  /// **'That email is already registered.'**
  String get errorEmailAlreadyExists;

  /// No description provided for @errorInvalidLanguage.
  ///
  /// In en, this message translates to:
  /// **'Unsupported language.'**
  String get errorInvalidLanguage;

  /// No description provided for @errorInvalidPlateNumber.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid plate number.'**
  String get errorInvalidPlateNumber;

  /// No description provided for @errorOdometerDecreased.
  ///
  /// In en, this message translates to:
  /// **'Odometer cannot be lower than the current value.'**
  String get errorOdometerDecreased;

  /// No description provided for @errorInvalidFuelType.
  ///
  /// In en, this message translates to:
  /// **'Choose a valid fuel type.'**
  String get errorInvalidFuelType;

  /// No description provided for @errorInvalidReminderType.
  ///
  /// In en, this message translates to:
  /// **'Choose a valid reminder type.'**
  String get errorInvalidReminderType;

  /// No description provided for @errorMissingDueDate.
  ///
  /// In en, this message translates to:
  /// **'A due date is required for this reminder.'**
  String get errorMissingDueDate;

  /// No description provided for @errorMissingDueOdometer.
  ///
  /// In en, this message translates to:
  /// **'A due odometer is required for this reminder.'**
  String get errorMissingDueOdometer;

  /// No description provided for @errorInvalidCategory.
  ///
  /// In en, this message translates to:
  /// **'Choose a valid category.'**
  String get errorInvalidCategory;

  /// No description provided for @errorInvalidDocType.
  ///
  /// In en, this message translates to:
  /// **'Choose a valid document type.'**
  String get errorInvalidDocType;

  /// No description provided for @authWelcomeBack.
  ///
  /// In en, this message translates to:
  /// **'Welcome back'**
  String get authWelcomeBack;

  /// No description provided for @authSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Sign in to track your vehicles, fuel, and services.'**
  String get authSubtitle;

  /// No description provided for @authEmailHint.
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get authEmailHint;

  /// No description provided for @authEmailLabel.
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get authEmailLabel;

  /// No description provided for @authPasswordHint.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get authPasswordHint;

  /// No description provided for @authPasswordLabel.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get authPasswordLabel;

  /// No description provided for @authShow.
  ///
  /// In en, this message translates to:
  /// **'Show'**
  String get authShow;

  /// No description provided for @authHide.
  ///
  /// In en, this message translates to:
  /// **'Hide'**
  String get authHide;

  /// No description provided for @authForgotPassword.
  ///
  /// In en, this message translates to:
  /// **'Forgot password?'**
  String get authForgotPassword;

  /// No description provided for @authLogIn.
  ///
  /// In en, this message translates to:
  /// **'Log in'**
  String get authLogIn;

  /// No description provided for @authNewToVeyra.
  ///
  /// In en, this message translates to:
  /// **'New to Veyra? '**
  String get authNewToVeyra;

  /// No description provided for @authCreateAccount.
  ///
  /// In en, this message translates to:
  /// **'Create account'**
  String get authCreateAccount;

  /// No description provided for @authCreateTitle.
  ///
  /// In en, this message translates to:
  /// **'Create your account'**
  String get authCreateTitle;

  /// No description provided for @authCreateSubtitle.
  ///
  /// In en, this message translates to:
  /// **'One account keeps every vehicle, log, and reminder in sync.'**
  String get authCreateSubtitle;

  /// No description provided for @authNameLabel.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get authNameLabel;

  /// No description provided for @authNameHint.
  ///
  /// In en, this message translates to:
  /// **'Your name'**
  String get authNameHint;

  /// No description provided for @authPasswordHelp.
  ///
  /// In en, this message translates to:
  /// **'Use 8 or more characters.'**
  String get authPasswordHelp;

  /// No description provided for @authAlreadyHaveAccount.
  ///
  /// In en, this message translates to:
  /// **'Already have an account? Log in'**
  String get authAlreadyHaveAccount;

  /// No description provided for @authEnterName.
  ///
  /// In en, this message translates to:
  /// **'Enter your name.'**
  String get authEnterName;

  /// No description provided for @commonRetry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get commonRetry;

  /// No description provided for @commonServices.
  ///
  /// In en, this message translates to:
  /// **'Services'**
  String get commonServices;

  /// No description provided for @commonExpenses.
  ///
  /// In en, this message translates to:
  /// **'Expenses'**
  String get commonExpenses;

  /// No description provided for @garageTitle.
  ///
  /// In en, this message translates to:
  /// **'Garage'**
  String get garageTitle;

  /// No description provided for @garageNavGarage.
  ///
  /// In en, this message translates to:
  /// **'Garage'**
  String get garageNavGarage;

  /// No description provided for @garageNavReminders.
  ///
  /// In en, this message translates to:
  /// **'Reminders'**
  String get garageNavReminders;

  /// No description provided for @garageNavDocs.
  ///
  /// In en, this message translates to:
  /// **'Docs'**
  String get garageNavDocs;

  /// No description provided for @garageNavSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get garageNavSettings;

  /// No description provided for @garageOverviewVehicles.
  ///
  /// In en, this message translates to:
  /// **'Vehicles'**
  String get garageOverviewVehicles;

  /// No description provided for @garageOverviewDueSoon.
  ///
  /// In en, this message translates to:
  /// **'Due soon'**
  String get garageOverviewDueSoon;

  /// No description provided for @garageOverviewSpent.
  ///
  /// In en, this message translates to:
  /// **'Spent'**
  String get garageOverviewSpent;

  /// No description provided for @garageCardFuel.
  ///
  /// In en, this message translates to:
  /// **'Fuel'**
  String get garageCardFuel;

  /// No description provided for @garageCardOdometer.
  ///
  /// In en, this message translates to:
  /// **'Odometer {value} km'**
  String garageCardOdometer(String value);

  /// No description provided for @garageDueBadge.
  ///
  /// In en, this message translates to:
  /// **'{count,plural, =1{1 due} other{{count} due}}'**
  String garageDueBadge(int count);

  /// No description provided for @garageAddVehicle.
  ///
  /// In en, this message translates to:
  /// **'Add vehicle'**
  String get garageAddVehicle;

  /// No description provided for @garageEmptyTitle.
  ///
  /// In en, this message translates to:
  /// **'Add your first vehicle'**
  String get garageEmptyTitle;

  /// No description provided for @garageEmptyBody.
  ///
  /// In en, this message translates to:
  /// **'Track services, fuel, expenses, and reminders once your car or bike is in the garage.'**
  String get garageEmptyBody;

  /// No description provided for @garageErrorTitle.
  ///
  /// In en, this message translates to:
  /// **'Can\'t reach Veyra'**
  String get garageErrorTitle;

  /// No description provided for @garageErrorBody.
  ///
  /// In en, this message translates to:
  /// **'We couldn\'t load your garage. Check your connection, then try again.'**
  String get garageErrorBody;

  /// No description provided for @garageErrorTryAgain.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get garageErrorTryAgain;

  /// No description provided for @garageErrorCheckSettings.
  ///
  /// In en, this message translates to:
  /// **'Check server settings'**
  String get garageErrorCheckSettings;

  /// No description provided for @settingsLogOut.
  ///
  /// In en, this message translates to:
  /// **'Log out'**
  String get settingsLogOut;

  /// No description provided for @vehicleDetailTabOverview.
  ///
  /// In en, this message translates to:
  /// **'Overview'**
  String get vehicleDetailTabOverview;

  /// No description provided for @vehicleDetailTabFuel.
  ///
  /// In en, this message translates to:
  /// **'Fuel'**
  String get vehicleDetailTabFuel;

  /// No description provided for @vehicleDetailTabService.
  ///
  /// In en, this message translates to:
  /// **'Service'**
  String get vehicleDetailTabService;

  /// No description provided for @vehicleDetailTabExpenses.
  ///
  /// In en, this message translates to:
  /// **'Expenses'**
  String get vehicleDetailTabExpenses;

  /// No description provided for @vehicleDetailTabDocs.
  ///
  /// In en, this message translates to:
  /// **'Docs'**
  String get vehicleDetailTabDocs;

  /// No description provided for @vehicleDetailStatServices.
  ///
  /// In en, this message translates to:
  /// **'Services'**
  String get vehicleDetailStatServices;

  /// No description provided for @vehicleDetailStatServiceCost.
  ///
  /// In en, this message translates to:
  /// **'Service cost'**
  String get vehicleDetailStatServiceCost;

  /// No description provided for @vehicleDetailStatRefuels.
  ///
  /// In en, this message translates to:
  /// **'Refuels'**
  String get vehicleDetailStatRefuels;

  /// No description provided for @vehicleDetailStatFuelCost.
  ///
  /// In en, this message translates to:
  /// **'Fuel cost'**
  String get vehicleDetailStatFuelCost;

  /// No description provided for @vehicleDetailOdometerLabel.
  ///
  /// In en, this message translates to:
  /// **'Odometer'**
  String get vehicleDetailOdometerLabel;

  /// No description provided for @vehicleDetailDueSoon.
  ///
  /// In en, this message translates to:
  /// **'Due soon'**
  String get vehicleDetailDueSoon;

  /// No description provided for @vehicleDetailReminders.
  ///
  /// In en, this message translates to:
  /// **'{count,plural, =1{1 reminder} other{{count} reminders}}'**
  String vehicleDetailReminders(int count);

  /// No description provided for @vehicleDetailAddFuel.
  ///
  /// In en, this message translates to:
  /// **'Log fuel'**
  String get vehicleDetailAddFuel;

  /// No description provided for @vehicleDetailAddService.
  ///
  /// In en, this message translates to:
  /// **'Add service'**
  String get vehicleDetailAddService;

  /// No description provided for @vehicleDetailAddExpense.
  ///
  /// In en, this message translates to:
  /// **'Add expense'**
  String get vehicleDetailAddExpense;

  /// No description provided for @vehicleDetailAddDocument.
  ///
  /// In en, this message translates to:
  /// **'Add document'**
  String get vehicleDetailAddDocument;

  /// No description provided for @vehicleDetailActivityHint.
  ///
  /// In en, this message translates to:
  /// **'Open the Fuel, Service, Expenses, or Docs tab to see entries.'**
  String get vehicleDetailActivityHint;

  /// No description provided for @vehicleDetailErrorRetry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get vehicleDetailErrorRetry;

  /// No description provided for @vehicleActivityTitle.
  ///
  /// In en, this message translates to:
  /// **'Recent activity'**
  String get vehicleActivityTitle;

  /// No description provided for @vehicleActivityExpense.
  ///
  /// In en, this message translates to:
  /// **'Expense'**
  String get vehicleActivityExpense;

  /// No description provided for @vehicleActivityEmpty.
  ///
  /// In en, this message translates to:
  /// **'No activity yet. Log fuel, a service, or an expense.'**
  String get vehicleActivityEmpty;

  /// No description provided for @vehicleAddTitle.
  ///
  /// In en, this message translates to:
  /// **'Add vehicle'**
  String get vehicleAddTitle;

  /// No description provided for @vehicleAddFieldBrand.
  ///
  /// In en, this message translates to:
  /// **'Brand'**
  String get vehicleAddFieldBrand;

  /// No description provided for @vehicleAddFieldModel.
  ///
  /// In en, this message translates to:
  /// **'Model'**
  String get vehicleAddFieldModel;

  /// No description provided for @vehicleAddFieldYear.
  ///
  /// In en, this message translates to:
  /// **'Year'**
  String get vehicleAddFieldYear;

  /// No description provided for @vehicleAddFieldPlate.
  ///
  /// In en, this message translates to:
  /// **'Plate number'**
  String get vehicleAddFieldPlate;

  /// No description provided for @vehicleAddFieldFuelType.
  ///
  /// In en, this message translates to:
  /// **'Fuel type'**
  String get vehicleAddFieldFuelType;

  /// No description provided for @vehicleAddFieldOdometer.
  ///
  /// In en, this message translates to:
  /// **'Odometer (km)'**
  String get vehicleAddFieldOdometer;

  /// No description provided for @vehicleAddFieldColor.
  ///
  /// In en, this message translates to:
  /// **'Color (optional)'**
  String get vehicleAddFieldColor;

  /// No description provided for @vehicleAddSave.
  ///
  /// In en, this message translates to:
  /// **'Save vehicle'**
  String get vehicleAddSave;

  /// No description provided for @vehicleAddErrorBrandModel.
  ///
  /// In en, this message translates to:
  /// **'Enter the brand and model.'**
  String get vehicleAddErrorBrandModel;

  /// No description provided for @vehicleAddErrorYear.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid year.'**
  String get vehicleAddErrorYear;

  /// No description provided for @vehicleAddErrorPlate.
  ///
  /// In en, this message translates to:
  /// **'Enter a plate number.'**
  String get vehicleAddErrorPlate;

  /// No description provided for @vehicleAddErrorOdometer.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid odometer reading.'**
  String get vehicleAddErrorOdometer;

  /// No description provided for @fuelTypePetrol.
  ///
  /// In en, this message translates to:
  /// **'Petrol'**
  String get fuelTypePetrol;

  /// No description provided for @fuelTypeDiesel.
  ///
  /// In en, this message translates to:
  /// **'Diesel'**
  String get fuelTypeDiesel;

  /// No description provided for @fuelTypeElectric.
  ///
  /// In en, this message translates to:
  /// **'Electric'**
  String get fuelTypeElectric;

  /// No description provided for @fuelTypeHybrid.
  ///
  /// In en, this message translates to:
  /// **'Hybrid'**
  String get fuelTypeHybrid;

  /// No description provided for @reminderTitle.
  ///
  /// In en, this message translates to:
  /// **'Reminders'**
  String get reminderTitle;

  /// No description provided for @reminderSectionOverdue.
  ///
  /// In en, this message translates to:
  /// **'Overdue'**
  String get reminderSectionOverdue;

  /// No description provided for @reminderSectionDueSoon.
  ///
  /// In en, this message translates to:
  /// **'Due Soon'**
  String get reminderSectionDueSoon;

  /// No description provided for @reminderSectionUpcoming.
  ///
  /// In en, this message translates to:
  /// **'Upcoming'**
  String get reminderSectionUpcoming;

  /// No description provided for @reminderSectionCompleted.
  ///
  /// In en, this message translates to:
  /// **'Completed'**
  String get reminderSectionCompleted;

  /// No description provided for @reminderMarkComplete.
  ///
  /// In en, this message translates to:
  /// **'Mark complete'**
  String get reminderMarkComplete;

  /// No description provided for @reminderDaysLate.
  ///
  /// In en, this message translates to:
  /// **'{count,plural, =1{1 day late} other{{count} days late}}'**
  String reminderDaysLate(int count);

  /// No description provided for @reminderDaysUntil.
  ///
  /// In en, this message translates to:
  /// **'{count,plural, =1{in 1 day} other{in {count} days}}'**
  String reminderDaysUntil(int count);

  /// No description provided for @reminderAtKm.
  ///
  /// In en, this message translates to:
  /// **'at {km} km'**
  String reminderAtKm(int km);

  /// No description provided for @reminderMetaVehicleDate.
  ///
  /// In en, this message translates to:
  /// **'{vehicle} · due {date}'**
  String reminderMetaVehicleDate(String vehicle, String date);

  /// No description provided for @reminderMetaVehicleKm.
  ///
  /// In en, this message translates to:
  /// **'{vehicle} · at {km} km'**
  String reminderMetaVehicleKm(String vehicle, int km);

  /// No description provided for @reminderEmpty.
  ///
  /// In en, this message translates to:
  /// **'No reminders yet'**
  String get reminderEmpty;

  /// No description provided for @reminderError.
  ///
  /// In en, this message translates to:
  /// **'Could not load reminders'**
  String get reminderError;

  /// No description provided for @reminderTypeDate.
  ///
  /// In en, this message translates to:
  /// **'Date'**
  String get reminderTypeDate;

  /// No description provided for @reminderTypeOdometer.
  ///
  /// In en, this message translates to:
  /// **'Odometer'**
  String get reminderTypeOdometer;

  /// No description provided for @reminderTypeBoth.
  ///
  /// In en, this message translates to:
  /// **'Date & Odometer'**
  String get reminderTypeBoth;

  /// No description provided for @reminderFormTitle.
  ///
  /// In en, this message translates to:
  /// **'Add Reminder'**
  String get reminderFormTitle;

  /// No description provided for @reminderFormTitleLabel.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get reminderFormTitleLabel;

  /// No description provided for @reminderFormTitleHint.
  ///
  /// In en, this message translates to:
  /// **'Road tax renewal'**
  String get reminderFormTitleHint;

  /// No description provided for @reminderFormTrigger.
  ///
  /// In en, this message translates to:
  /// **'Trigger'**
  String get reminderFormTrigger;

  /// No description provided for @reminderFormNotes.
  ///
  /// In en, this message translates to:
  /// **'Notes (optional)'**
  String get reminderFormNotes;

  /// No description provided for @reminderFormNotesHint.
  ///
  /// In en, this message translates to:
  /// **'Anything to note'**
  String get reminderFormNotesHint;

  /// No description provided for @reminderFormPickDate.
  ///
  /// In en, this message translates to:
  /// **'Pick a date'**
  String get reminderFormPickDate;

  /// No description provided for @reminderFormDueDate.
  ///
  /// In en, this message translates to:
  /// **'Due Date'**
  String get reminderFormDueDate;

  /// No description provided for @reminderFormDueOdometer.
  ///
  /// In en, this message translates to:
  /// **'Due Odometer (km)'**
  String get reminderFormDueOdometer;

  /// No description provided for @reminderFormSave.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get reminderFormSave;

  /// No description provided for @reminderAdd.
  ///
  /// In en, this message translates to:
  /// **'Add reminder'**
  String get reminderAdd;

  /// No description provided for @reminderPickVehicle.
  ///
  /// In en, this message translates to:
  /// **'Choose a vehicle'**
  String get reminderPickVehicle;

  /// No description provided for @commonTryAgain.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get commonTryAgain;

  /// No description provided for @documentTitle.
  ///
  /// In en, this message translates to:
  /// **'Documents'**
  String get documentTitle;

  /// No description provided for @documentAllVehicles.
  ///
  /// In en, this message translates to:
  /// **'All vehicles'**
  String get documentAllVehicles;

  /// No description provided for @documentCountAcrossVehicles.
  ///
  /// In en, this message translates to:
  /// **'{count,plural, =1{1 across all vehicles} other{{count} across all vehicles}}'**
  String documentCountAcrossVehicles(int count);

  /// No description provided for @documentErrorTitle.
  ///
  /// In en, this message translates to:
  /// **'Can\'t load documents'**
  String get documentErrorTitle;

  /// No description provided for @documentEmptyTitle.
  ///
  /// In en, this message translates to:
  /// **'No documents yet'**
  String get documentEmptyTitle;

  /// No description provided for @documentEmptyBody.
  ///
  /// In en, this message translates to:
  /// **'No documents yet. Add STNK, BPKB, insurance, and more from a vehicle.'**
  String get documentEmptyBody;

  /// No description provided for @documentEmptyBodyDetail.
  ///
  /// In en, this message translates to:
  /// **'Add the STNK, BPKB, insurance, or any document you want to keep with this vehicle.'**
  String get documentEmptyBodyDetail;

  /// No description provided for @documentStatusExpired.
  ///
  /// In en, this message translates to:
  /// **'Expired'**
  String get documentStatusExpired;

  /// No description provided for @documentStatusExpiringSoon.
  ///
  /// In en, this message translates to:
  /// **'Expiring soon'**
  String get documentStatusExpiringSoon;

  /// No description provided for @documentStatusValid.
  ///
  /// In en, this message translates to:
  /// **'Valid'**
  String get documentStatusValid;

  /// No description provided for @documentStatusOnFile.
  ///
  /// In en, this message translates to:
  /// **'On file'**
  String get documentStatusOnFile;

  /// No description provided for @documentDaysLeft.
  ///
  /// In en, this message translates to:
  /// **'{count,plural, =1{1 day left} other{{count} days left}}'**
  String documentDaysLeft(int count);

  /// No description provided for @documentNoExpiry.
  ///
  /// In en, this message translates to:
  /// **'No expiry'**
  String get documentNoExpiry;

  /// No description provided for @documentValidUntil.
  ///
  /// In en, this message translates to:
  /// **'Valid until {date}'**
  String documentValidUntil(String date);

  /// No description provided for @documentExpires.
  ///
  /// In en, this message translates to:
  /// **'Expires {date}'**
  String documentExpires(String date);

  /// No description provided for @documentNoExpiryMeta.
  ///
  /// In en, this message translates to:
  /// **'No expiry · {docType}'**
  String documentNoExpiryMeta(String docType);

  /// No description provided for @documentExpiresMeta.
  ///
  /// In en, this message translates to:
  /// **'Expires {date}'**
  String documentExpiresMeta(String date);

  /// No description provided for @documentAddTitle.
  ///
  /// In en, this message translates to:
  /// **'Add document'**
  String get documentAddTitle;

  /// No description provided for @documentFieldType.
  ///
  /// In en, this message translates to:
  /// **'Type'**
  String get documentFieldType;

  /// No description provided for @documentFieldTitle.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get documentFieldTitle;

  /// No description provided for @documentFieldTitleHint.
  ///
  /// In en, this message translates to:
  /// **'STNK'**
  String get documentFieldTitleHint;

  /// No description provided for @documentFieldExpiry.
  ///
  /// In en, this message translates to:
  /// **'Expiry date (optional)'**
  String get documentFieldExpiry;

  /// No description provided for @documentFieldFileUrl.
  ///
  /// In en, this message translates to:
  /// **'File URL (optional)'**
  String get documentFieldFileUrl;

  /// No description provided for @documentFieldFileUrlHint.
  ///
  /// In en, this message translates to:
  /// **'https://…'**
  String get documentFieldFileUrlHint;

  /// No description provided for @documentFieldNotes.
  ///
  /// In en, this message translates to:
  /// **'Notes (optional)'**
  String get documentFieldNotes;

  /// No description provided for @documentFieldNotesHint.
  ///
  /// In en, this message translates to:
  /// **'Anything to note'**
  String get documentFieldNotesHint;

  /// No description provided for @documentSave.
  ///
  /// In en, this message translates to:
  /// **'Save document'**
  String get documentSave;

  /// No description provided for @documentErrorEnterTitle.
  ///
  /// In en, this message translates to:
  /// **'Enter a document title.'**
  String get documentErrorEnterTitle;

  /// No description provided for @docTypeStnk.
  ///
  /// In en, this message translates to:
  /// **'STNK'**
  String get docTypeStnk;

  /// No description provided for @docTypeBpkb.
  ///
  /// In en, this message translates to:
  /// **'BPKB'**
  String get docTypeBpkb;

  /// No description provided for @docTypeInsurance.
  ///
  /// In en, this message translates to:
  /// **'Insurance'**
  String get docTypeInsurance;

  /// No description provided for @docTypeOther.
  ///
  /// In en, this message translates to:
  /// **'Other'**
  String get docTypeOther;

  /// No description provided for @fuelLogTitle.
  ///
  /// In en, this message translates to:
  /// **'Log fuel'**
  String get fuelLogTitle;

  /// No description provided for @fuelLogFieldDate.
  ///
  /// In en, this message translates to:
  /// **'Date'**
  String get fuelLogFieldDate;

  /// No description provided for @fuelLogFieldOdometer.
  ///
  /// In en, this message translates to:
  /// **'Odometer'**
  String get fuelLogFieldOdometer;

  /// No description provided for @fuelLogFieldLiters.
  ///
  /// In en, this message translates to:
  /// **'Liters'**
  String get fuelLogFieldLiters;

  /// No description provided for @fuelLogFieldPricePerLiter.
  ///
  /// In en, this message translates to:
  /// **'Price / liter'**
  String get fuelLogFieldPricePerLiter;

  /// No description provided for @fuelLogFieldStation.
  ///
  /// In en, this message translates to:
  /// **'Station (optional)'**
  String get fuelLogFieldStation;

  /// No description provided for @fuelLogFieldTotalCost.
  ///
  /// In en, this message translates to:
  /// **'Total cost'**
  String get fuelLogFieldTotalCost;

  /// No description provided for @fuelLogFieldFullTank.
  ///
  /// In en, this message translates to:
  /// **'Full tank'**
  String get fuelLogFieldFullTank;

  /// No description provided for @fuelLogFieldFullTankHint.
  ///
  /// In en, this message translates to:
  /// **'Used to compute consumption'**
  String get fuelLogFieldFullTankHint;

  /// No description provided for @fuelLogSave.
  ///
  /// In en, this message translates to:
  /// **'Save fuel log'**
  String get fuelLogSave;

  /// No description provided for @fuelLogErrorOdometer.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid odometer reading.'**
  String get fuelLogErrorOdometer;

  /// No description provided for @fuelLogErrorLiters.
  ///
  /// In en, this message translates to:
  /// **'Enter the liters filled.'**
  String get fuelLogErrorLiters;

  /// No description provided for @fuelLogErrorPricePerLiter.
  ///
  /// In en, this message translates to:
  /// **'Enter the price per liter.'**
  String get fuelLogErrorPricePerLiter;

  /// No description provided for @fuelLogEmpty.
  ///
  /// In en, this message translates to:
  /// **'No fuel logs yet. Tap \"Log fuel\" to add your first fill-up.'**
  String get fuelLogEmpty;

  /// No description provided for @fuelLogLoadError.
  ///
  /// In en, this message translates to:
  /// **'Could not load fuel logs.'**
  String get fuelLogLoadError;

  /// No description provided for @serviceRecordTitle.
  ///
  /// In en, this message translates to:
  /// **'Log service'**
  String get serviceRecordTitle;

  /// No description provided for @serviceRecordFieldDate.
  ///
  /// In en, this message translates to:
  /// **'Date'**
  String get serviceRecordFieldDate;

  /// No description provided for @serviceRecordFieldOdometer.
  ///
  /// In en, this message translates to:
  /// **'Odometer (km)'**
  String get serviceRecordFieldOdometer;

  /// No description provided for @serviceRecordFieldDescription.
  ///
  /// In en, this message translates to:
  /// **'Description'**
  String get serviceRecordFieldDescription;

  /// No description provided for @serviceRecordFieldWorkshop.
  ///
  /// In en, this message translates to:
  /// **'Workshop (optional)'**
  String get serviceRecordFieldWorkshop;

  /// No description provided for @serviceRecordFieldCost.
  ///
  /// In en, this message translates to:
  /// **'Cost (optional)'**
  String get serviceRecordFieldCost;

  /// No description provided for @serviceRecordFieldNotes.
  ///
  /// In en, this message translates to:
  /// **'Notes (optional)'**
  String get serviceRecordFieldNotes;

  /// No description provided for @serviceRecordSave.
  ///
  /// In en, this message translates to:
  /// **'Save service'**
  String get serviceRecordSave;

  /// No description provided for @serviceRecordErrorDescription.
  ///
  /// In en, this message translates to:
  /// **'Enter a description.'**
  String get serviceRecordErrorDescription;

  /// No description provided for @serviceRecordErrorOdometer.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid odometer reading.'**
  String get serviceRecordErrorOdometer;

  /// No description provided for @serviceRecordErrorCost.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid cost.'**
  String get serviceRecordErrorCost;

  /// No description provided for @serviceRecordEmpty.
  ///
  /// In en, this message translates to:
  /// **'No service records yet. Log the first one.'**
  String get serviceRecordEmpty;

  /// No description provided for @serviceRecordLoadError.
  ///
  /// In en, this message translates to:
  /// **'Could not load service records.'**
  String get serviceRecordLoadError;

  /// No description provided for @expenseAddTitle.
  ///
  /// In en, this message translates to:
  /// **'Add expense'**
  String get expenseAddTitle;

  /// No description provided for @expenseCategoryBattery.
  ///
  /// In en, this message translates to:
  /// **'Battery'**
  String get expenseCategoryBattery;

  /// No description provided for @expenseCategoryInsurance.
  ///
  /// In en, this message translates to:
  /// **'Insurance'**
  String get expenseCategoryInsurance;

  /// No description provided for @expenseCategoryOther.
  ///
  /// In en, this message translates to:
  /// **'Other'**
  String get expenseCategoryOther;

  /// No description provided for @expenseCategoryTax.
  ///
  /// In en, this message translates to:
  /// **'Tax'**
  String get expenseCategoryTax;

  /// No description provided for @expenseCategoryTire.
  ///
  /// In en, this message translates to:
  /// **'Tire'**
  String get expenseCategoryTire;

  /// No description provided for @expenseEmpty.
  ///
  /// In en, this message translates to:
  /// **'No expenses yet. Add your first one.'**
  String get expenseEmpty;

  /// No description provided for @expenseErrorEnterDescription.
  ///
  /// In en, this message translates to:
  /// **'Enter a description.'**
  String get expenseErrorEnterDescription;

  /// No description provided for @expenseErrorInvalidAmount.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid amount.'**
  String get expenseErrorInvalidAmount;

  /// No description provided for @expenseFieldAmount.
  ///
  /// In en, this message translates to:
  /// **'Amount (Rp)'**
  String get expenseFieldAmount;

  /// No description provided for @expenseFieldCategory.
  ///
  /// In en, this message translates to:
  /// **'Category'**
  String get expenseFieldCategory;

  /// No description provided for @expenseFieldDate.
  ///
  /// In en, this message translates to:
  /// **'Date'**
  String get expenseFieldDate;

  /// No description provided for @expenseFieldDescription.
  ///
  /// In en, this message translates to:
  /// **'Description'**
  String get expenseFieldDescription;

  /// No description provided for @expenseFieldDescriptionHint.
  ///
  /// In en, this message translates to:
  /// **'Annual premium'**
  String get expenseFieldDescriptionHint;

  /// No description provided for @expenseLoadError.
  ///
  /// In en, this message translates to:
  /// **'Could not load expenses.'**
  String get expenseLoadError;

  /// No description provided for @expenseSave.
  ///
  /// In en, this message translates to:
  /// **'Save expense'**
  String get expenseSave;

  /// No description provided for @expenseTotalThisYear.
  ///
  /// In en, this message translates to:
  /// **'Total this year'**
  String get expenseTotalThisYear;

  /// No description provided for @expenseSummaryCount.
  ///
  /// In en, this message translates to:
  /// **'{count,plural, =1{Across 1 expense} other{Across {count} expenses}}'**
  String expenseSummaryCount(int count);

  /// No description provided for @expenseSummaryHighest.
  ///
  /// In en, this message translates to:
  /// **'highest: {category}'**
  String expenseSummaryHighest(String category);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'id'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'id':
      return AppLocalizationsId();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
