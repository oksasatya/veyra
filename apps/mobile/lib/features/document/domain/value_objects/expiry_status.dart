/// Expiry status of a document relative to a reference date.
///
/// `onFile` means the document has no expiry (e.g. an ownership book). The
/// other three are derived from the gap between the expiry date and today.
enum ExpiryStatus {
  expired,
  expiringSoon,
  valid,
  onFile;

  String get label => switch (this) {
    ExpiryStatus.expired => 'Expired',
    ExpiryStatus.expiringSoon => 'Expiring soon',
    ExpiryStatus.valid => 'Valid',
    ExpiryStatus.onFile => 'On file',
  };
}

/// Documents within this many days of expiry are flagged as expiring soon.
const expirySoonWindowDays = 30;

/// Classify a document's expiry against [today]. Pure: `today` is passed in so
/// the rule is deterministic and testable — it never reads the clock here.
///
/// A null [expiry] means the document never expires → [ExpiryStatus.onFile].
/// Comparison is date-only (time-of-day ignored) so a document expiring "today"
/// is still [ExpiryStatus.expiringSoon], not [ExpiryStatus.expired].
ExpiryStatus expiryStatusFor({
  required DateTime? expiry,
  required DateTime today,
}) {
  if (expiry == null) return ExpiryStatus.onFile;
  final due = DateTime(expiry.year, expiry.month, expiry.day);
  final ref = DateTime(today.year, today.month, today.day);
  final daysLeft = due.difference(ref).inDays;
  if (daysLeft < 0) return ExpiryStatus.expired;
  if (daysLeft <= expirySoonWindowDays) return ExpiryStatus.expiringSoon;
  return ExpiryStatus.valid;
}
