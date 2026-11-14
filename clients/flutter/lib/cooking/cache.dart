import '../connection/nextcloud.dart';

/// Reject damaged recovery envelopes instead of replacing an unconfirmed request with empty state.
bool validCookingCache(Json value) {
  bool integer(Object? v) => v is int && v >= 0;
  bool run(Object? v) {
    if (v is! Json ||
        v['id'] is! String ||
        v['name'] is! String ||
        v['kind'] is! String ||
        v['timezone'] is! String ||
        v['steps'] is! List) {
      return false;
    }
    return (v['steps'] as List).every(
      (s) =>
          s is Json &&
          s['stage'] is Json &&
          (s['stage'] as Json)['id'] is String &&
          s['timer'] is Json &&
          (s['timer'] as Json)['state'] is String,
    );
  }

  if (value['version'] != 1 ||
      value['sessions'] is! Json ||
      value['enabled'] is! Json ||
      value['pending'] is! List) {
    return false;
  }
  final pending = value['pending'] as List,
      sessions = value['sessions'] as Json;
  if (pending.length > 20 ||
      sessions.length > 5 ||
      !(value['enabled'] as Json).values.every((v) => v is bool)) {
    return false;
  }
  if (!sessions.entries.every(
    (e) =>
        e.value is Json &&
        run((e.value as Json)['run']) &&
        ((e.value as Json)['run'] as Json)['id'] == e.key,
  )) {
    return false;
  }
  if (value['authorized_at'] != null && !integer(value['authorized_at'])) {
    return false;
  }
  if (value['clock'] != null) {
    final c = value['clock'];
    if (c is! Json ||
        c['boot'] is! String ||
        !integer(c['elapsed_ms']) ||
        !integer(c['server_now'])) {
      return false;
    }
  }
  if (value['list'] != null) {
    final l = value['list'];
    if (l is! Json ||
        !integer(l['revision']) ||
        l['runs'] is! List ||
        !integer(l['server_now'])) {
      return false;
    }
  }
  for (final p in pending) {
    if (p is! Json ||
        ![
          'pending',
          'unknown',
          'rejected',
          'review_required',
        ].contains(p['status']) ||
        p['attempted'] is! bool ||
        p['request'] is! Json ||
        ![null, 'brewing'].contains(p['domain'])) {
      return false;
    }
    final r = p['request'] as Json;
    if (!integer(r['expected_revision']) ||
        r['idempotency_key'] is! String ||
        (r['idempotency_key'] as String).isEmpty ||
        r['command'] is! Json ||
        (r['command'] as Json)['operation'] is! String) {
      return false;
    }
    if (p['domain'] == 'brewing' && r['reviewed'] != true) return false;
    if (r['occurred_at'] != null && !integer(r['occurred_at'])) return false;
  }
  return true;
}
