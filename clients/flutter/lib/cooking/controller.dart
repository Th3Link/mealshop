import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'device.dart';
import 'cache.dart';

/// Owns only one captured account/household. Domain transitions still belong to Rust.
class CookingController extends ChangeNotifier {
  CookingController(
    this.api, {
    ReminderDevice? device,
    Future<void>? scopeReady,
  }) : device = device ?? AndroidReminders(),
       scopeReady = scopeReady ?? Future<void>.value() {
    clock = CookingClock(this.device);
  }
  final HouseholdApi api;
  final ReminderDevice device;
  final Future<void> scopeReady;
  late final CookingClock clock;
  Json state = {
    'version': 1,
    'sessions': <String, dynamic>{},
    'pending': <Json>[],
    'enabled': <String, dynamic>{},
  };
  Json deviceStatus = {'permission': 'unsupported'};
  bool recoveryBlocked = false;
  bool initialized = false,
      busy = false,
      offline = false,
      limitedReminders = false,
      _closed = false;
  String? error;
  String? lastResult;
  Future<void>? _refreshing;
  Future<void>? _initializing;
  Json? get list => state['list'] as Json?;
  Json get sessions => object(state['sessions']);
  List<Json> get pending => objects(state['pending']);
  bool get current => !_closed && api.current;
  bool get writable => current && !recoveryBlocked && api.writable;
  bool get protocol => list?['action_protocol'] == 1;
  bool get blocked => pending.any((p) => p['status'] != 'pending');
  bool enabled(String id) => object(state['enabled'])[id] == true;
  Json? detail(String id) => sessions[id] as Json?;

  void _notify() {
    if (current) notifyListeners();
  }

  Future<void> initialize() => _initializing ??= _initialize();
  Future<void> _initialize() async {
    await scopeReady;
    if (!current) return;
    final saved = api.controller.draft('cooking.v1');
    if (saved != null) {
      if (!validCookingCache(saved)) {
        recoveryBlocked = true;
        error = 'cooking_recovery_blocked';
        initialized = true;
        await device.clear();
        _notify();
        return;
      } else {
        state = clone(saved);
        clock.anchor = state['clock'] as Json?;
      }
    }
    await clock.observe();
    if (!current) return;
    initialized = true;
    final authorized = state['authorized_at'] as int?;
    final instant = clock.now ?? (clock.sample!.wall ~/ 1000);
    if (authorized != null &&
        (instant < authorized - 300 || instant >= authorized + 604800)) {
      // Expire private snapshots, but retain immutable uncertain envelopes for exact server reconciliation.
      state['sessions'] = <String, dynamic>{};
      state['enabled'] = <String, dynamic>{};
      state.remove('list');
      error = 'cooking_cache_expired';
      await _save();
    }
    await reconcileReminders();
    _notify();
    if (list != null && api.controller.online) await refresh();
  }

  Future<void> _save() async {
    if (!current) throw const ApiFailure('access_removed', 403);
    if (recoveryBlocked) throw const ApiFailure('cooking_recovery_blocked');
    final candidate = clone(state);
    candidate['clock'] = clock.anchor;
    if (utf8.encode(jsonEncode(candidate)).length > 245760) {
      throw const ApiFailure('cache_capacity');
    }
    await api.controller.saveDraft('cooking.v1', candidate);
    if (!current) throw const ApiFailure('access_removed', 403);
  }

  Future<void> refresh() {
    if (_refreshing != null) return _refreshing!;
    final future = _refresh();
    _refreshing = future;
    return future.whenComplete(() {
      if (identical(_refreshing, future)) _refreshing = null;
    });
  }

  Future<void> _refresh() async {
    if (!current || recoveryBlocked) return;
    busy = true;
    _notify();
    try {
      final value = await api.household('/cooking');
      if (value['revision'] is! int ||
          value['server_now'] is! int ||
          value['runs'] is! List) {
        throw const ApiFailure('invalid_response');
      }
      await clock.observe(value['server_now'] as int);
      if (!current) return;
      state['list'] = value;
      state['authorized_at'] = value['server_now'];
      offline = false;
      error = null;
      await _save();
      if (writable && await _flush()) {
        state['list'] = await api.household('/cooking');
        await clock.observe(list!['server_now'] as int);
        state['authorized_at'] = list!['server_now'];
        await _save();
      }
      for (final id in sessions.keys.toList()) {
        try {
          await load(id);
        } on ApiFailure catch (failure) {
          if (failure.status != 404) rethrow;
        }
      }
      await reconcileReminders();
    } on ApiFailure catch (failure) {
      if (!current) return;
      error = failure.code;
      offline = !failure.denied;
      if (failure.denied) await revoke();
    } catch (_) {
      if (current) {
        offline = true;
        error = 'connection_failed';
      }
    } finally {
      if (current) {
        busy = false;
        _notify();
      }
    }
  }

  Future<Json> load(String id, {bool brewing = false}) async {
    if (recoveryBlocked) throw const ApiFailure('cooking_recovery_blocked');
    final knownKind = (detail(id)?['run'] as Json?)?['kind'];
    final brew =
        brewing || ['beer', 'cider', 'wine', 'mead'].contains(knownKind);
    Json value;
    try {
      value = await api.household(
        brew ? '/brewing/batches/$id' : '/cooking/$id',
      );
    } on ApiFailure catch (failure) {
      if (failure.denied) {
        await revoke();
      } else if (failure.status == 404 && current) {
        sessions.remove(id);
        object(state['enabled']).remove(id);
        await _save();
        await reconcileReminders();
      }
      rethrow;
    }
    if (value['run'] is! Json ||
        object(value['run'])['id'] != id ||
        (brew ? value['recipe'] is! Json : value['ingredients'] is! List) ||
        value['revision'] is! int) {
      throw const ApiFailure('invalid_response');
    }
    if (list != null &&
        (value['revision'] as int) < (list!['revision'] as int)) {
      throw const ApiFailure('cooking_revision_conflict', 409);
    }
    await clock.observe(value['server_now'] as int);
    if (!current) throw const ApiFailure('access_removed', 403);
    final before = clone(state);
    if (!sessions.containsKey(id) && sessions.length >= 5) {
      final removable = sessions.keys
          .where(
            (key) =>
                !enabled(key) &&
                !pending.any(
                  (p) =>
                      object(object(p['request'])['command'])['run_id'] == key,
                ),
          )
          .firstOrNull;
      if (removable == null) throw const ApiFailure('cooking_cache_full');
      sessions.remove(removable);
    }
    sessions[id] = value;
    try {
      await _save();
    } catch (_) {
      state = before;
      rethrow;
    }
    offline = false;
    await reconcileReminders();
    _notify();
    return value;
  }

  Future<void> enqueue(Json command, {int? expectedRevision}) async {
    if (!protocol) throw const ApiFailure('mobile_upgrade_required');
    if (!writable || busy || blocked) {
      throw const ApiFailure('cooking_pending_review');
    }
    if (!api.controller.online && command['operation'] != 'timer') {
      throw const ApiFailure('online_required');
    }
    if (pending.length >= 20) throw const ApiFailure('outbox_capacity');
    await clock.observe();
    if (!current || !clock.trusted) {
      throw const ApiFailure('cooking_clock_unverified');
    }
    if (state['authorized_at'] == null ||
        clock.now! >= (state['authorized_at'] as int) + 604800) {
      throw const ApiFailure('cooking_cache_expired');
    }
    if (command['operation'] == 'timer' &&
        pending.any((p) {
          final old = object(object(p['request'])['command']);
          return old['run_id'] == command['run_id'] &&
              old['step_id'] == command['step_id'];
        })) {
      throw const ApiFailure('cooking_pending_review');
    }
    final revision = pending.isEmpty
        ? expectedRevision ?? list!['revision'] as int
        : (object(pending.last['request'])['expected_revision'] as int) + 1;
    final request = <String, dynamic>{
      'expected_revision': revision,
      'idempotency_key': newId('cook-'),
      'command': clone(command),
      if (command['operation'] == 'timer') 'occurred_at': clock.now!,
    };
    final previous = clone(state);
    state['pending'] = [
      ...pending,
      {'request': request, 'status': 'pending', 'attempted': false},
    ];
    try {
      await _save();
    } catch (_) {
      state = previous;
      rethrow;
    }
    _notify();
    if (api.controller.online) await refresh();
  }

  Future<bool> _flush() async {
    var changed = false;
    while (current && writable && pending.isNotEmpty) {
      final entry = pending.first;
      if (!['pending', 'unknown'].contains(entry['status'])) break;
      entry['attempted'] = true;
      entry['status'] = 'unknown';
      await _save(); // Immutable key, body and uncertain outcome survive before network dispatch.
      try {
        final request = object(entry['request']);
        final brewing = entry['domain'] == 'brewing';
        final result = await api.household(
          brewing ? '/brewing' : '/cooking',
          method: 'POST',
          body: request,
        );
        final revision = brewing
            ? object(result['brewing'])['revision']
            : result['revision'];
        if ((brewing
                ? result['operation_id'] is! String ||
                      revision is! int ||
                      revision < (request['expected_revision'] as int) + 1
                : result['applied_revision'] !=
                      (request['expected_revision'] as int) + 1) ||
            result['replayed'] is! bool) {
          throw const ApiFailure('invalid_response');
        }
        state['pending'] = pending.skip(1).toList();
        lastResult = result[brewing ? 'result_id' : 'run_id'] as String?;
        changed = true;
        if (list != null) list!['revision'] = revision;
        await _save();
      } on ApiFailure catch (failure) {
        if (!current) rethrow;
        if (failure.denied) {
          await revoke();
          rethrow;
        }
        entry['error'] = failure.code;
        entry['status'] = [404, 409, 410, 422].contains(failure.status)
            ? 'rejected'
            : 'unknown';
        await _save();
        if (entry['status'] == 'unknown') offline = true;
        break;
      }
    }
    return changed;
  }

  Future<(Json, Json)> reviewRejected() async {
    if (!writable || pending.isEmpty || pending.first['status'] == 'unknown') {
      throw const ApiFailure('outcome_unknown');
    }
    final latest = await api.household('/cooking');
    final request = clone(object(pending.first['request']));
    if (object(request['command'])['operation'] != 'timer') {
      throw const ApiFailure('cooking_review_expired');
    }
    request['expected_revision'] = latest['revision'];
    request['idempotency_key'] = newId('cook-');
    final preview = await api.household(
      '/cooking/action-preview',
      method: 'POST',
      body: request,
    );
    return (request, preview);
  }

  Future<void> submitBrewing(Json input, {bool replaceRejected = false}) async {
    if (!writable || !api.controller.online || busy) {
      throw const ApiFailure('online_required');
    }
    if (pending.isNotEmpty &&
        (!replaceRejected ||
            ![
              'rejected',
              'review_required',
            ].contains(pending.first['status']))) {
      throw const ApiFailure('cooking_pending_review');
    }
    final entry = <String, dynamic>{
      'domain': 'brewing',
      'request': clone(input),
      'status': 'pending',
      'attempted': false,
    };
    final previous = clone(state);
    state['pending'] = replaceRejected
        ? [
            entry,
            for (final later in pending.skip(1))
              {...later, 'status': 'review_required'},
          ]
        : [entry];
    try {
      await _save();
    } catch (_) {
      state = previous;
      rethrow;
    }
    _notify();
    await refresh();
  }

  Future<void> confirmReviewed(Json request) async {
    if (!writable || pending.isEmpty || pending.first['status'] == 'unknown') {
      throw const ApiFailure('outcome_unknown');
    }
    state['pending'] = [
      {'request': clone(request), 'status': 'pending', 'attempted': false},
      for (final entry in pending.skip(1))
        {...entry, 'status': 'review_required'},
    ];
    await _save();
    await refresh();
  }

  Future<void> discardRejected() async {
    if (pending.any(
      (p) =>
          p['status'] == 'unknown' ||
          (p['attempted'] == true && p['status'] != 'rejected'),
    )) {
      throw const ApiFailure('outcome_unknown');
    }
    state['pending'] = <Json>[];
    await _save();
    _notify();
    await refresh();
  }

  Future<void> setReminders(String id, bool value) async {
    if (!current || detail(id) == null) {
      throw const ApiFailure('online_required');
    }
    if (value &&
        (clock.now == null ||
            state['authorized_at'] == null ||
            clock.now! >= (state['authorized_at'] as int) + 604800)) {
      if (!api.controller.online) {
        throw const ApiFailure('cooking_clock_unverified');
      }
      await refresh();
      if (clock.now == null || state['authorized_at'] == null) {
        throw const ApiFailure('online_required');
      }
    }
    if (value) deviceStatus = await device.requestPermission();
    if (!current) return;
    object(state['enabled'])[id] = value;
    state['device_scope'] ??= newId('device-');
    await _save();
    await reconcileReminders();
    _notify();
  }

  Future<void> forget(String id) async {
    if (pending.isNotEmpty) throw const ApiFailure('cooking_pending_review');
    sessions.remove(id);
    object(state['enabled']).remove(id);
    await _save();
    await reconcileReminders();
    _notify();
  }

  Future<void> reconcileReminders() async {
    if (!current) return;
    try {
      deviceStatus = await device.status();
      final now = clock.now;
      final authorized = state['authorized_at'] as int?;
      if (!current) return;
      if (object(state['enabled']).values.every((value) => value != true) ||
          authorized == null ||
          (deviceStatus['scope'] != null &&
              deviceStatus['scope'] != state['device_scope'])) {
        await device.clear();
        if (object(state['enabled']).values.every((value) => value != true) ||
            authorized == null) {
          return;
        }
      }
      if (now == null) {
        return; // Native reboot recovery retains the same absolute deadlines.
      }
      if (now >= authorized + 604800) {
        await device.clear();
        return;
      }
      final entries = <Json>[];
      for (final id in sessions.keys.where(enabled)) {
        entries.addAll(
          reminderProjection(
            detail(id)!,
            state['device_scope'] as String,
            api.controller.language,
            authorized + 604800,
          ),
        );
      }
      entries.removeWhere((e) => (e['due'] as int) < now - 86400);
      entries.sort((a, b) => (a['due'] as int).compareTo(b['due'] as int));
      limitedReminders = entries.length > 200;
      if (entries.isEmpty) {
        await device.clear();
      } else {
        deviceStatus = await device.replace(entries.take(200).toList());
      }
    } on ApiFailure catch (failure) {
      error = failure.code;
    }
  }

  Future<void> revoke() async {
    state = {
      'version': 1,
      'sessions': <String, dynamic>{},
      'pending': <Json>[],
      'enabled': <String, dynamic>{},
    };
    await device.clear();
    if (current) await api.controller.saveDraft('cooking.v1', null);
  }

  @override
  void dispose() {
    _closed = true;
    super.dispose();
  }
}
