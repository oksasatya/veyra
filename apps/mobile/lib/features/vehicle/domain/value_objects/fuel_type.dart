/// Vehicle fuel type — mirrors the backend's `'petrol' | 'diesel' | 'electric'
/// | 'hybrid'` string. `name` is the wire value (`petrol`, …).
enum FuelType {
  petrol,
  diesel,
  electric,
  hybrid;

  static FuelType fromApi(String raw) => switch (raw) {
    'diesel' => FuelType.diesel,
    'electric' => FuelType.electric,
    'hybrid' => FuelType.hybrid,
    _ => FuelType.petrol,
  };

  String get apiValue => name;

  String get label => switch (this) {
    FuelType.petrol => 'Petrol',
    FuelType.diesel => 'Diesel',
    FuelType.electric => 'Electric',
    FuelType.hybrid => 'Hybrid',
  };
}
