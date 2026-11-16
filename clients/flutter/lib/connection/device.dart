import 'dart:convert';

import 'package:flutter/services.dart';

import 'errors.dart';

class DeviceReply {
  DeviceReply(Json value)
    : status = value['status'] as int,
      contentType = value['content_type'] as String,
      bytes = base64Decode(value['body'] as String);
  final int status;
  final String contentType;
  final Uint8List bytes;
}

/// A private native channel, with no HTTP fallback or installation URL.
class DeviceGateway {
  DeviceGateway({MethodChannel? channel})
    : channel = channel ?? const MethodChannel('org.mealshop/device_engine');
  final MethodChannel channel;
  Future<String> notices() async =>
      (await channel.invokeMethod<String>('notices'))!;

  Future<Json> command(Json value) async {
    try {
      final result = await channel.invokeMethod<String>('command', {
        'value': jsonEncode(value),
      });
      if (result == null || utf8.encode(result).length > 16777216) {
        throw const ApiFailure('device_response_too_large');
      }
      return object(jsonDecode(result));
    } on PlatformException catch (error) {
      throw ApiFailure(error.code);
    } on MissingPluginException {
      throw const ApiFailure('device_unavailable');
    } on ApiFailure {
      rethrow;
    } catch (_) {
      throw const ApiFailure('device_engine_failed');
    }
  }

  Future<Json> status() => command({'operation': 'status'});
  Future<void> stop() async {
    await command({'operation': 'shutdown'});
  }

  Future<DeviceReply> request(
    String path, {
    required String? generation,
    String method = 'GET',
    Uint8List? bytes,
    String? mime,
  }) async {
    final result = await command({
      'operation': 'request',
      'generation': generation,
      'path': path,
      'method': method,
      'body': bytes == null ? null : base64Encode(bytes),
      'content_type': mime,
    });
    try {
      return DeviceReply(result);
    } catch (_) {
      throw const ApiFailure('device_engine_failed');
    }
  }
}

class BackupDocuments {
  const BackupDocuments();
  static const channel = MethodChannel('org.mealshop/device_backups');
  Future<T?> _call<T>(String method, [Json? arguments]) async {
    try {
      return await channel.invokeMethod<T>(method, arguments);
    } on PlatformException catch (error) {
      throw ApiFailure(error.code);
    } on MissingPluginException {
      throw const ApiFailure('device_unavailable');
    }
  }

  Future<bool> export(String id) async =>
      await _call<bool>('export', {'id': id}) == true;
  Future<String?> select() => _call<String>('import');
}
