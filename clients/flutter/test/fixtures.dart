import 'dart:convert';

import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/connection/vault.dart';

class MemoryVault implements Vault {
  Json? value;
  bool fail = false;
  @override
  Future<Json?> read() async =>
      value == null ? null : object(jsonDecode(jsonEncode(value)));
  @override
  Future<void> write(Json next) async {
    if (fail) throw const ApiFailure('storage_failed');
    value = object(jsonDecode(jsonEncode(next)));
  }

  @override
  Future<void> clear() async {
    value = null;
  }
}

Json shopping(int revision) => {
  'revision': revision,
  'items': <Object?>[],
  'sources': <Object?>[],
  'products': <String, Object?>{},
  'shops': {
    'entries': <Object?>[],
    'preferences': <String, Object?>{},
    'assignments': <String, Object?>{},
    'manual_order': <String, Object?>{},
    'learning': false,
    'sessions': <Object?>[],
  },
};
Json membership({String role = 'owner'}) => {
  'user': 'alice',
  'households': [
    {'id': 'home-1', 'name': 'Synthetic household', 'role': role},
  ],
  'invitations': <Object?>[],
};
Json capability(int revision, {String role = 'owner'}) => {
  'version': 1,
  'max_operation_age_seconds': 604800,
  'server_now': DateTime.now().millisecondsSinceEpoch ~/ 1000,
  'role': role,
  'shopping': shopping(revision),
};
