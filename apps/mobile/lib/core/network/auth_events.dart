import 'package:flutter_riverpod/flutter_riverpod.dart';

/// A signal raised by the network layer (refresh interceptor) when a session
/// has definitively expired. The auth layer listens and flips to logged-out;
/// keeps `core` free of any dependency on the auth feature.
class AuthEvents extends Notifier<int> {
  @override
  int build() => 0;

  /// Raise the "session expired" signal.
  void sessionExpired() => state = state + 1;
}

final authEventsProvider = NotifierProvider<AuthEvents, int>(AuthEvents.new);
