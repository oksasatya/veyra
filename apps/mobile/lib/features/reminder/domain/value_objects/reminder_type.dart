/// Reminder trigger type — mirrors the backend's `'date' | 'odometer' | 'both'`
/// string. `name` is the wire value (`date`, …).
enum ReminderType {
  date,
  odometer,
  both;

  static ReminderType fromApi(String raw) => switch (raw) {
    'odometer' => ReminderType.odometer,
    'both' => ReminderType.both,
    _ => ReminderType.date,
  };

  String get apiValue => name;

  String get label => switch (this) {
    ReminderType.date => 'By date',
    ReminderType.odometer => 'By odometer',
    ReminderType.both => 'Date & odometer',
  };
}
