import 'dart:async';

import 'package:flutter/services.dart';

import '../connection/nextcloud.dart';

class DeviceTime {
  const DeviceTime(this.wall, this.elapsed, this.boot, [this.timezone = 'UTC']);
  final int wall, elapsed;
  final String boot, timezone;
  Json toJson() => {
    'wall_ms': wall,
    'elapsed_ms': elapsed,
    'boot': boot,
    'timezone': timezone,
  };
  factory DeviceTime.parse(Json value) => DeviceTime(
    value['wall_ms'] as int,
    value['elapsed_ms'] as int,
    value['boot'] as String,
    value['timezone'] as String? ?? 'UTC',
  );
}

abstract interface class ReminderDevice {
  Future<DeviceTime> clock();
  Future<Json> status();
  Future<Json> requestPermission();
  Future<Json> requestPrecision();
  Future<Json> replace(List<Json> entries, {String? generation});
  Future<void> clear();
}

class AndroidReminders implements ReminderDevice {
  static const channel = MethodChannel('org.mealshop/reminders');
  final _fallbackClock = Stopwatch()..start();
  final _process = DateTime.now().microsecondsSinceEpoch.toString();
  Future<Json> _call(String method, [Json? input]) async {
    try {
      final result = await channel.invokeMapMethod<String, dynamic>(
        method,
        input,
      );
      return object(result);
    } on MissingPluginException {
      return {'permission': 'unsupported', 'precision': 'unsupported'};
    } on PlatformException {
      throw const ApiFailure('reminders_failed');
    }
  }

  @override
  Future<DeviceTime> clock() async {
    final value = await _call('clock');
    if (value['boot'] == null) {
      return DeviceTime(
        DateTime.now().millisecondsSinceEpoch,
        _fallbackClock.elapsedMilliseconds,
        'process-$_process',
      );
    }
    return DeviceTime.parse(value);
  }

  @override
  Future<Json> status() => _call('status');
  @override
  Future<Json> requestPermission() => _call('requestPermission');
  @override
  Future<Json> requestPrecision() => _call('requestPrecision');
  @override
  Future<Json> replace(List<Json> entries, {String? generation}) =>
      _call('replace', {'entries': entries, 'generation': generation});
  @override
  Future<void> clear() async {
    await _call('clear');
  }
}

/// A server anchor advanced by boot-relative time; a wall-clock edit cannot restart a timer.
class CookingClock {
  CookingClock(this.device);
  final ReminderDevice device;
  Json? anchor;
  DeviceTime? sample;
  final _elapsed = Stopwatch();
  bool get trusted =>
      anchor != null &&
      sample != null &&
      anchor!['boot'] == sample!.boot &&
      sample!.boot != 'unknown' &&
      sample!.elapsed >= (anchor!['elapsed_ms'] as int);
  int? get now {
    if (!trusted) return null;
    return (anchor!['server_now'] as int) +
        (sample!.elapsed -
                (anchor!['elapsed_ms'] as int) +
                _elapsed.elapsedMilliseconds) ~/
            1000;
  }

  Future<void> observe([int? serverNow]) async {
    sample = await device.clock();
    _elapsed
      ..reset()
      ..start();
    if (serverNow != null) {
      anchor = {...sample!.toJson(), 'server_now': serverNow};
    }
  }
}

List<Json> reminderProjection(
  Json detail,
  String scope,
  String language,
  int expires,
) {
  final run = object(detail['run']);
  if (run['status'] != 'active') return [];
  final entries = <Json>[];
  for (final step in (run['steps'] as List).map(object)) {
    final stage = object(step['stage']), timer = object(step['timer']);
    final events = <String, int>{};
    if (timer['state'] == 'pending') {
      final timing = (run['schedule'] as List)
          .map(object)
          .where((t) => t['id'] == stage['id'])
          .firstOrNull;
      if (timing != null) {
        events['prepare'] =
            (timing['start'] as int) - (stage['lead_seconds'] as int);
        events['start'] = timing['start'] as int;
      }
    } else if (timer['state'] == 'running') {
      if ((timer['active_until'] as int) > (timer['started_at'] as int) &&
          (timer['active_until'] as int) < (timer['deadline'] as int)) {
        events['active_ended'] = timer['active_until'] as int;
      }
      // Untimed brewing work has no fabricated elapsed notification.
      if ((timer['deadline'] as int) > (timer['started_at'] as int)) {
        events['timer_ended'] = timer['deadline'] as int;
      }
    }
    for (final event in events.entries) {
      if (event.value > 0 && event.value <= expires) {
        entries.add({
          'key':
              '$scope:${run['id']}:${stage['id']}:${step['generation']}:${event.key}',
          'due': event.value,
          'expires_at': expires,
          'language': language,
        });
      }
    }
  }
  return entries;
}
