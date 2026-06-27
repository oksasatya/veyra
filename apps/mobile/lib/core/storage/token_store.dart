import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Persisted bearer tokens. `refresh` is the opaque `{family_id}.{raw_secret}`
/// string — stored and replayed verbatim, never parsed on the client.
class Tokens {
  const Tokens({required this.access, required this.refresh});
  final String access;
  final String refresh;
}

class TokenStore {
  TokenStore(this._storage);
  final FlutterSecureStorage _storage;

  static const _kAccess = 'veyra_access';
  static const _kRefresh = 'veyra_refresh';

  Future<Tokens?> read() async {
    final access = await _storage.read(key: _kAccess);
    final refresh = await _storage.read(key: _kRefresh);
    if (access == null || refresh == null) return null;
    return Tokens(access: access, refresh: refresh);
  }

  Future<void> save(Tokens tokens) async {
    await _storage.write(key: _kAccess, value: tokens.access);
    await _storage.write(key: _kRefresh, value: tokens.refresh);
  }

  Future<void> clear() async {
    await _storage.delete(key: _kAccess);
    await _storage.delete(key: _kRefresh);
  }
}

final tokenStoreProvider = Provider<TokenStore>(
  (ref) => TokenStore(const FlutterSecureStorage()),
);
