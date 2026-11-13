import 'dart:convert';

import 'package:flutter/services.dart';

import 'nextcloud.dart';

abstract interface class Vault {
  Future<Json?> read();
  Future<void> write(Json value);
  Future<void> clear();
}

/// Android Keystore AES-GCM + AtomicFile; acknowledgement follows fsync, not a queued write.
class DeviceVault implements Vault {
  DeviceVault({this._namespace = 'mealshop.phone.v1'});
  static const _channel = MethodChannel('org.mealshop/device_vault');
  final String _namespace;
  @override
  Future<Json?> read() async {
    final value = await _channel.invokeMethod<String>('read', {
      'namespace': _namespace,
    });
    if (value == null) return null;
    if (utf8.encode(value).length > 1048576) {
      throw const ApiFailure('storage_failed');
    }
    final parsed = object(jsonDecode(value));
    if (parsed['version'] != 1) throw const ApiFailure('storage_version');
    return parsed;
  }

  @override
  Future<void> write(Json value) async {
    final encoded = jsonEncode(value);
    if (utf8.encode(encoded).length > 1048576) {
      throw const ApiFailure('cache_capacity');
    }
    try {
      await _channel.invokeMethod<void>('write', {
        'namespace': _namespace,
        'value': encoded,
      });
    } catch (_) {
      throw const ApiFailure('storage_failed');
    }
  }

  @override
  Future<void> clear() =>
      _channel.invokeMethod<void>('clear', {'namespace': _namespace});
}
