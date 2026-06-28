import 'package:veyra_mobile/features/vehicle/domain/value_objects/fuel_type.dart';
import 'package:veyra_mobile/l10n/app_localizations.dart';

/// Map a [FuelType] to its localized label. Kept in the presentation layer so
/// the domain enum stays free of Flutter/l10n dependencies.
String localizedFuelType(AppLocalizations l10n, FuelType fuelType) =>
    switch (fuelType) {
      FuelType.petrol => l10n.fuelTypePetrol,
      FuelType.diesel => l10n.fuelTypeDiesel,
      FuelType.electric => l10n.fuelTypeElectric,
      FuelType.hybrid => l10n.fuelTypeHybrid,
    };
