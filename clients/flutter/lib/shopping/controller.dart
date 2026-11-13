import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:flutter/foundation.dart';

import '../connection/nextcloud.dart';
import '../connection/vault.dart';
import 'models.dart';

class ShoppingController extends ChangeNotifier {
  ShoppingController(this.api, this.vault);
  final Nextcloud api;
  final Vault vault;
  Credential? credential;
  List<Home> homes = [], invitations = [];
  String? selected, error;
  String language = 'en';
  bool initialized = false,
      busy = false,
      online = false,
      storageBlocked = false;
  int _epoch = 0, _offset = 0;
  bool _selectionRequired = false;
  Json _records = {};
  Timer? _timer;
  Future<void>? _refreshing;
  Future<void> _storageTail = Future<void>.value();
  Home? get home => homes.where((h) => h.id == selected).firstOrNull;
  Json? get _record => selected == null ? null : _records[selected] as Json?;
  Shopping? get shopping => _record?['shopping'] == null
      ? null
      : Shopping(object(_record!['shopping']));
  DateTime? get syncedAt => _record?['synced_at'] == null
      ? null
      : DateTime.fromMillisecondsSinceEpoch(
          (_record!['synced_at'] as int) * 1000,
        );
  List<Pending> get pending => (_record?['pending'] as List? ?? [])
      .map((v) => Pending.parse(object(v)))
      .toList();
  Map<String, IngredientGroup> get ingredientGroups =>
      object(_record?['ingredient_groups'] ?? <String, dynamic>{})
          .map((id, value) => MapEntry(id, IngredientGroup(object(value))));
  bool get writable =>
      home?.writable == true && shopping != null && !storageBlocked;
  String get _domain => '/api/households/$selected/domain/api/household';

  Future<void> initialize({String deviceLanguage = 'en'}) async {
    language = deviceLanguage == 'de' ? 'de' : 'en';
    try {
      final value = await vault.read();
      if (value != null) {
        credential = value['credential'] == null
            ? null
            : Credential.parse(object(value['credential']));
        language = value['language'] as String;
        homes = (value['homes'] as List).map((v) => Home(object(v))).toList();
        selected = value['selected'] as String?;
        _selectionRequired = value['selection_required'] as bool? ?? false;
        _records = object(value['records']);
        _offset = value['clock_offset'] as int;
        if (_expireRecords()) await _save();
      }
    } catch (_) {
      storageBlocked = true;
      error = 'storage_failed';
    }
    initialized = true;
    notifyListeners();
    if (!storageBlocked && credential != null) await refresh();
    _timer = Timer.periodic(const Duration(seconds: 20), (_) {
      if (credential != null && !busy && !storageBlocked) unawaited(refresh());
    });
  }

  Json _state() => {
    'version': 1,
    'credential': credential?.toJson(),
    'language': language,
    'homes': homes.map((h) => h.toJson()).toList(),
    'selected': selected,
    'selection_required': _selectionRequired,
    'records': _records,
    'clock_offset': _offset,
  };
  Future<void> _save() async {
    final epoch = _epoch;
    final snapshot = object(jsonDecode(jsonEncode(_state())));
    final write = _storageTail.then((_) => vault.write(snapshot));
    _storageTail = write.catchError((Object _) {});
    try {
      await write;
    } catch (_) {
      if (epoch == _epoch) {
        storageBlocked = true;
        error = 'storage_failed';
        notifyListeners();
      }
      throw const ApiFailure('storage_failed');
    }
  }

  Future<void> signIn(Credential value) async {
    await signOut();
    credential = value;
    await _save();
    await refresh();
  }

  Future<void> signOut() async {
    _epoch++;
    _refreshing = null;
    await _storageTail;
    await vault.clear();
    credential = null;
    homes = [];
    invitations = [];
    selected = null;
    _selectionRequired = false;
    _records = {};
    error = null;
    busy = false;
    online = false;
    storageBlocked = false;
    notifyListeners();
  }

  Future<void> setLanguage(String value) async {
    language = value;
    await _save();
    notifyListeners();
  }

  Future<void> select(String id) async {
    if (!homes.any((h) => h.id == id)) return;
    _epoch++;
    _refreshing = null;
    busy = false;
    selected = id;
    _selectionRequired = false;
    error = null;
    await _save();
    notifyListeners();
    await refresh();
  }

  bool _expireRecords() {
    var changed = false;
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    _records.removeWhere((_, value) {
      final synced = object(value)['synced_at'] as int;
      final expired = now - synced > 604800 || now < synced - 300;
      changed = changed || expired;
      return expired;
    });
    for (final record in _records.values.map(object)) {
      final previous = record['pending'] as List;
      final retained = previous
          .where(
            (value) =>
                now + _offset - (object(value)['issued_at'] as int) <= 604800,
          )
          .toList();
      changed = changed || retained.length != previous.length;
      record['pending'] = retained;
    }
    return changed;
  }

  Future<void> refresh() {
    if (_refreshing != null) return _refreshing!;
    final run = _refreshOnce();
    _refreshing = run;
    return run.whenComplete(() {
      if (identical(_refreshing, run)) _refreshing = null;
    });
  }

  Future<void> _refreshOnce() async {
    final who = credential;
    if (who == null || busy || storageBlocked) return;
    final epoch = _epoch;
    if (_expireRecords()) {
      await _save();
      notifyListeners();
    }
    if (epoch != _epoch) return;
    busy = true;
    notifyListeners();
    try {
      final membership = await api.api(who, '/api/households');
      if (epoch != _epoch) return;
      if (membership['user'] != who.user) {
        throw const ApiFailure('invalid_response');
      }
      homes = (membership['households'] as List)
          .map((v) => Home(object(v)))
          .toList();
      invitations = (membership['invitations'] as List)
          .map((v) => Home(object(v)))
          .toList();
      final allowed = homes.map((h) => h.id).toSet();
      final removed = selected != null && !allowed.contains(selected);
      _records.removeWhere((key, _) => !allowed.contains(key));
      if (!allowed.contains(selected)) {
        if (selected != null) {
          _selectionRequired = true;
          error = 'access_removed';
        }
        selected = !_selectionRequired && homes.length == 1
            ? homes.single.id
            : null;
      }
      if (selected != null) {
        final cap = await api.api(who, '$_domain/shopping/mobile');
        if (epoch != _epoch) return;
        if (cap['version'] != 1 || cap['max_operation_age_seconds'] != 604800) {
          throw const ApiFailure('mobile_upgrade_required');
        }
        final role = cap['role'] as String;
        homes = homes
            .map(
              (h) => h.id == selected ? Home({...h.toJson(), 'role': role}) : h,
            )
            .toList();
        final list = Shopping(object(cap['shopping']));
        _offset =
            (cap['server_now'] as int) -
            DateTime.now().millisecondsSinceEpoch ~/ 1000;
        final record =
            _records[selected] as Json? ??
            <String, dynamic>{'pending': <Object?>[]};
        record['shopping'] = list.json;
        record['ingredient_groups'] = object(
          cap['ingredient_groups'] ?? <String, dynamic>{},
        );
        record['synced_at'] = DateTime.now().millisecondsSinceEpoch ~/ 1000;
        _records[selected!] = record;
      }
      online = true;
      error = removed ? 'access_removed' : null;
      await _save();
      if (home?.writable == true) await _flush(epoch);
    } on ApiFailure catch (e) {
      if (epoch != _epoch) return;
      online = false;
      error = e.code;
      if (e.denied) {
        if (e.status == 401) {
          _records = {};
          error = 'login_expired';
        } else if (selected != null) {
          _records.remove(selected);
          error = 'access_removed';
        }
        await _save();
      }
    } catch (_) {
      if (epoch == _epoch) {
        online = false;
        error = 'connection_failed';
      }
    } finally {
      if (epoch == _epoch) {
        busy = false;
        notifyListeners();
      }
    }
  }

  Future<void> _flush(int epoch) async {
    while (pending.isNotEmpty && home?.writable == true) {
      final queue = pending;
      final entry = queue.first;
      if (entry.error != null && entry.error != 'connection_failed') break;
      if (DateTime.now().millisecondsSinceEpoch ~/ 1000 +
              _offset -
              entry.issuedAt >
          604800) {
        entry.error = 'shopping_operation_expired';
        _setQueue(queue);
        await _save();
        break;
      }
      entry.expected ??= shopping!.revision;
      entry.attempted = true;
      _replaceEntry(entry);
      await _save(); // Persist the immutable request BEFORE it can reach the network.
      if (epoch != _epoch) return;
      try {
        final result = await api.api(
          credential!,
          '$_domain/shopping/mobile',
          method: 'POST',
          body: {
            'operation_id': entry.id,
            'issued_at': entry.issuedAt,
            'expected_revision': entry.expected,
            'command': entry.command,
          },
        );
        if (epoch != _epoch) return;
        if (result['version'] != 1 ||
            result['operation_id'] != entry.id ||
            result['applied_revision'] != entry.expected! + 1 ||
            result['replayed'] is! bool) {
          throw const ApiFailure('invalid_response');
        }
        _record!['shopping'] = Shopping(object(result['shopping'])).json;
        _record!['ingredient_groups'] = object(
          result['ingredient_groups'] ?? <String, dynamic>{},
        );
        _setQueue(pending.where((p) => p.id != entry.id).toList());
        await _save();
        notifyListeners();
      } on ApiFailure catch (e) {
        if (epoch != _epoch) return;
        if (e.denied) rethrow;
        final rejected =
            (e.status == 409 || e.status == 422) &&
            e.code != 'invalid_response';
        entry.error = rejected ? e.code : 'connection_failed';
        _replaceEntry(entry);
        await _save();
        if (!rejected) {
          online = false;
          error = 'connection_failed';
        }
        break;
      }
    }
  }

  void _replaceEntry(Pending entry) =>
      _setQueue(pending.map((p) => p.id == entry.id ? entry : p).toList());
  void _setQueue(List<Pending> entries) {
    _record!['pending'] = entries.map((p) => p.toJson()).toList();
  }

  Pending _intent(Json command, String? original, {int? expectedRevision}) {
    final random = Random.secure();
    final id = List.generate(
      24,
      (_) => random.nextInt(256).toRadixString(16).padLeft(2, '0'),
    ).join();
    return Pending(
      id,
      DateTime.now().millisecondsSinceEpoch ~/ 1000 + _offset,
      command,
      expected:
          command['operation'] == 'add_manual' && command['source_id'] == null
          ? null
          : expectedRevision ?? shopping!.revision,
      original: original,
    );
  }

  Future<void> replaceReviewed(
    String id,
    Json command, {
    String? original,
    int? expectedRevision,
  }) async {
    if (!writable || busy) throw const ApiFailure('online_required');
    final queue = pending;
    final previous = queue.firstWhere((p) => p.id == id);
    if (previous.uncertain || (previous.attempted && previous.error == null)) {
      throw const ApiFailure('outcome_unknown');
    }
    queue[queue.indexOf(previous)] = _intent(
      command,
      original,
      expectedRevision: expectedRevision,
    );
    _setQueue(queue);
    await _save();
    notifyListeners();
    unawaited(refresh());
  }

  Future<void> enqueue(
    Json command, {
    String? original,
    bool offline = false,
    int? expectedRevision,
  }) async {
    if (!writable || (busy && !offline) || (!online && !offline)) {
      throw const ApiFailure('online_required');
    }
    final queue = pending;
    if (queue.length >= 100) throw const ApiFailure('outbox_capacity');
    queue.add(_intent(command, original, expectedRevision: expectedRevision));
    _setQueue(queue);
    await _save();
    notifyListeners();
    unawaited(refresh());
  }

  Future<void> markPending(String id, bool purchased) async {
    if (!writable) throw const ApiFailure('online_required');
    final entries = pending;
    final entry = entries.firstWhere((p) => p.id == id);
    if (entry.attempted ||
        entry.command['operation'] != 'add_manual' ||
        entry.command['source_id'] != null) {
      throw const ApiFailure('outcome_unknown');
    }
    entry.command['purchased'] = purchased;
    _setQueue(entries);
    await _save();
    notifyListeners();
  }

  Future<void> discard(String id) async {
    final queue = pending;
    final entry = queue.firstWhere((p) => p.id == id);
    if (entry.uncertain) throw const ApiFailure('outcome_unknown');
    queue.removeWhere((p) => p.id == id);
    _setQueue(queue);
    await _save();
    notifyListeners();
    await refresh();
  }

  Future<void> retryReviewed(String id, {int? expectedRevision}) async {
    final entry = pending.firstWhere((p) => p.id == id);
    if (!online || entry.uncertain || entry.error == null) {
      throw const ApiFailure('online_required');
    }
    await replaceReviewed(
      id,
      entry.command,
      original: entry.original,
      expectedRevision: expectedRevision,
    );
  }

  Future<void> createHome(String name) async {
    if (credential == null || busy) return;
    final result = await api.api(
      credential!,
      '/api/households',
      method: 'POST',
      body: {'name': name},
    );
    selected = result['id'] as String;
    await refresh();
  }

  Future<void> invitation(String id, bool accept) async {
    await api.api(
      credential!,
      '/api/households/$id/invitation',
      method: 'PUT',
      body: {'accept': accept},
    );
    await refresh();
  }

  Future<List<Json>> ingredients() async {
    final response = await api.api(credential!, '$_domain/ingredients');
    return (response['ingredients'] as List).map(object).toList();
  }

  Future<Json?> lookup(String barcode) async {
    if (!online || pending.isNotEmpty) {
      throw const ApiFailure('online_required');
    }
    final result = await api.api(
      credential!,
      '$_domain/shopping/lookup',
      method: 'POST',
      body: {
        'expected_revision': shopping!.revision,
        'barcode': barcode,
        'consent': true,
      },
    );
    await refresh();
    return result['candidate'] == null ? null : object(result['candidate']);
  }

  void foreground(bool active) {
    if (active) {
      unawaited(refresh());
    } else {
      _timer?.cancel();
    }
    if (active && !(_timer?.isActive ?? false)) {
      _timer = Timer.periodic(
        const Duration(seconds: 20),
        (_) => unawaited(refresh()),
      );
    }
  }

  @override
  void dispose() {
    _timer?.cancel();
    api.close();
    super.dispose();
  }
}
