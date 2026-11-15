import 'package:flutter/services.dart';

import '../connection/nextcloud.dart';

class CaptureDevice {
  const CaptureDevice();
  static const channel = MethodChannel('org.mealshop/capture');
  Future<Json> recognize(
    Uint8List bytes, {
    int rotation = 0,
    bool barcode = false,
  }) async {
    try {
      return object(
        await channel.invokeMapMethod<String, dynamic>('recognize', {
          'bytes': bytes,
          'rotation': rotation,
          'barcode': barcode,
        }),
      );
    } on MissingPluginException {
      throw const ApiFailure('capture_unavailable');
    } on PlatformException {
      throw const ApiFailure('capture_failed');
    }
  }

  Future<String?> incoming() async {
    try {
      return await channel.invokeMethod<String>('peekShare');
    } on MissingPluginException {
      return null;
    } on PlatformException {
      throw const ApiFailure('storage_failed');
    }
  }

  Future<void> clearIncoming() => channel.invokeMethod<void>('clearShare');
  Future<void> share(String text) async {
    try {
      await channel.invokeMethod<void>('shareText', {'text': text});
    } on MissingPluginException {
      throw const ApiFailure('capture_unavailable');
    } on PlatformException {
      throw const ApiFailure('capture_failed');
    }
  }

  Future<void> discardPicked(String path) async {
    try {
      await channel.invokeMethod<void>('discardPicked', {'path': path});
    } on MissingPluginException {
      /* Other platforms use the picker's cache lifetime. */
    }
  }
}
