import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:mealshop/ui/cooking.dart';
import 'package:mealshop/ui/process_common.dart';
import 'package:mealshop/ui/cooking_review.dart';
import 'package:http/http.dart' as http;
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/cooking/controller.dart';
import 'package:mealshop/cooking/device.dart';

import 'content_fixtures.dart';

class FakeReminders implements ReminderDevice {
  DeviceTime sample = const DeviceTime(1800000000000, 1000000, 'boot-a');
  List<Json> entries = [];
  int clears = 0;
  @override
  Future<DeviceTime> clock() async => sample;
  @override
  Future<Json> status() async => {
    'permission': 'granted',
    'precision': 'inexact',
  };
  @override
  Future<Json> requestPermission() => status();
  @override
  Future<Json> requestPrecision() => status();
  @override
  Future<Json> replace(List<Json> entries, {String? generation}) async {
    this.entries = entries;
    return status();
  }

  @override
  Future<void> clear() async {
    clears++;
    entries = [];
  }
}

Json cookingList(int revision) => {
  'revision': revision,
  'server_now': 1800000000,
  'runs': <Json>[],
  'action_protocol': 1,
  'max_operation_age_seconds': 604800,
};
Json cookingDetail(int revision) => {
  'revision': revision,
  'server_now': 1800000000,
  'ingredients': [
    {
      'ingredient_id': 'oats',
      'name': 'Synthetic oats',
      'translations': <String, dynamic>{},
      'grams': '200.000001',
      'part_id': null,
    },
  ],
  'step_ingredients': [
    {
      'index': 0,
      'scope': 'explicit',
      'line_indexes': [0],
      'prepared_parts': <String>[],
      'part_id': null,
    },
    {
      'index': 1,
      'scope': 'unassigned',
      'line_indexes': [0],
      'prepared_parts': <String>[],
      'part_id': null,
    },
  ],
  'parts': <Json>[],
  'notices': <Json>[],
  'run': {
    'id': 'cook-1',
    'name': 'Synthetic cooking session',
    'kind': 'cooking',
    'recipe': {
      'id': 'recipe-00000001',
      'revision': 1,
      'title': 'Synthetic cooking recipe',
      'language': 'en',
    },
    'servings': 2,
    'timezone': 'UTC',
    'status': 'active',
    'created_at': 1799999900,
    'steps': [
      for (var index = 0; index < 2; index++)
        {
          'stage': {
            'id': 's${index + 1}-1',
            'origin': {'kind': 'recipe_step', 'index': index},
            'name': 'Synthetic step ${index + 1}',
            'instruction': 'Synthetic original instruction',
            'dependencies': <String>[],
            'active_seconds': 0,
            'passive_seconds': 60,
            'passive_min_seconds': 60,
            'passive_max_seconds': 60,
            'lead_seconds': 5,
            'start_after_work': true,
            'equipment': <String>[],
          },
          'timer': {'state': 'pending'},
          'generation': 1,
          'cycle': 1,
          'lag_seconds': 0,
          'ready': true,
          'available_at': 1799999900,
        },
    ],
    'schedule': [
      for (var index = 0; index < 2; index++)
        {
          'id': 's${index + 1}-1',
          'start': 1800000010,
          'active_until': 1800000010,
          'finish': 1800000070,
          'earliest': 1800000010,
          'latest': 1800000010,
        },
    ],
    'local_schedule': <Json>[],
    'subscription': null,
  },
};
http.Response cookedResponse(Json value, [int status = 200]) => http.Response(
  jsonEncode(value),
  status,
  headers: {'content-type': 'application/json'},
);

void main() {
  testWidgets(
    'custom-only cooking stages keep the full pinned ingredient reference offline',
    (tester) async {
      final detail = cookingDetail(0), run = object(detail['run']);
      for (final step in objects(run['steps'])) {
        object(step['stage'])['origin'] = {
          'kind': 'custom',
          'note': 'Synthetic custom kitchen work',
        };
      }
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: ProcessSteps(
                run: run,
                detail: detail,
                clock: CookingClock(FakeReminders()),
                writable: false,
                busy: false,
                timer: (_, _) async {},
                replan: (_, _) async {},
              ),
            ),
          ),
        ),
      );
      expect(
        find.text('Recipe ingredients · pinned quantities'),
        findsOneWidget,
      );
      expect(find.textContaining('200.000001 g'), findsOneWidget);
      expect(find.text('Synthetic original instruction'), findsNWidgets(2));
      await tester.pumpWidget(const SizedBox());
    },
  );

  test('a pre-P4 server remains readable but cannot accept an unreceipted mobile cooking intent', () async {
    final f = ContentFixture();
    f.respond = (r) async => r.url.path.endsWith('/cooking')
        ? cookedResponse(cookingList(0)..remove('action_protocol'))
        : null;
    await f.start();
    final c = CookingController(
      HouseholdApi(f.controller),
      device: FakeReminders(),
    );
    await c.initialize();
    await c.refresh();
    expect(c.list, isNotNull);
    await expectLater(
      c.enqueue({
        'operation': 'timer',
        'run_id': 'cook-1',
        'step_id': 's1-1',
        'action': 'start',
      }),
      throwsA(
        isA<ApiFailure>().having(
          (e) => e.code,
          'code',
          'mobile_upgrade_required',
        ),
      ),
    );
    expect(c.pending, isEmpty);
    expect(f.requests.where((r) => r.method == 'POST'), isEmpty);
    c.dispose();
    f.controller.dispose();
  });

  test(
    'damaged recovery data never drops pending actions or re-enables writes',
    () async {
      final f = ContentFixture(), d = FakeReminders();
      await f.start();
      final saved = <String, dynamic>{
        'version': 1,
        'sessions': <String, dynamic>{},
        'enabled': <String, dynamic>{},
        'pending': [
          {'status': 'unknown', 'attempted': true, 'request': 'damaged'},
        ],
      };
      await f.controller.saveDraft('cooking.v1', saved);
      final c = CookingController(HouseholdApi(f.controller), device: d);
      await c.initialize();
      expect(c.recoveryBlocked, true);
      expect(c.writable, false);
      await c.refresh();
      expect(f.controller.draft('cooking.v1'), saved);
      expect(c.error, 'cooking_recovery_blocked');
      c.dispose();
      f.controller.dispose();
    },
  );

  test('forgetting one offline copy cancels only its local reminders and leaves server records', () async {
    final f = ContentFixture(), d = FakeReminders();
    f.respond = (r) async {
      if (r.url.path.endsWith('/cooking')) {
        return cookedResponse(cookingList(0));
      }
      if (r.url.path.contains('/cooking/cook-')) {
        final detail = cookingDetail(0);
        object(detail['run'])['id'] = r.url.pathSegments.last;
        return cookedResponse(detail);
      }
      return null;
    };
    await f.start();
    final c = CookingController(HouseholdApi(f.controller), device: d);
    await c.initialize();
    await c.refresh();
    for (final id in ['cook-1', 'cook-2']) {
      await c.load(id);
      await c.setReminders(id, true);
    }
    expect(d.entries.length, 8);
    await c.forget('cook-1');
    expect(c.detail('cook-1'), isNull);
    expect(c.detail('cook-2'), isNotNull);
    expect(d.entries.length, 4);
    expect(
      d.entries.every((e) => (e['key'] as String).contains(':cook-2:')),
      true,
    );
    await c.load('cook-1');
    expect(c.detail('cook-1'), isNotNull);
    expect(
      f.requests.where(
        (r) => r.url.path.contains('/cooking') && r.method != 'GET',
      ),
      isEmpty,
    );
    c.dispose();
    f.controller.dispose();
  });
  testWidgets(
    'a canonical cooking schedule cannot be applied before explicit review',
    (tester) async {
      final f = ContentFixture(), d = FakeReminders();
      Json? written;
      var revision = 0;
      f.respond = (r) async {
        if (r.url.path.endsWith('/cooking')) {
          if (r.method == 'POST') {
            written = object(jsonDecode(r.body));
            revision++;
            return cookedResponse({
              'revision': revision,
              'applied_revision': revision,
              'run_id': 'cook-1',
              'replayed': false,
            });
          }
          return cookedResponse(cookingList(revision));
        }
        return null;
      };
      await f.start();
      final c = CookingController(HouseholdApi(f.controller), device: d);
      await c.initialize();
      await c.refresh();
      final preview = {
        'revision': 0,
        'preview_id': 'synthetic-review',
        'run': cookingDetail(0)['run'],
        'previous': null,
        'proposal': {'warnings': <Json>[]},
      };
      String? selected;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                onPressed: () async {
                  selected = await Navigator.push<String>(
                    context,
                    MaterialPageRoute(
                      builder: (_) => CookingReviewScreen(c, preview),
                    ),
                  );
                },
                child: const Text('Open synthetic review'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open synthetic review'));
      await tester.pumpAndSettle();
      final save = find.widgetWithText(FilledButton, 'Save this schedule');
      await tester.scrollUntilVisible(
        save,
        300,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      expect(tester.widget<FilledButton>(save).onPressed, isNull);
      expect(written, isNull);
      await tester.tap(find.byType(CheckboxListTile));
      await tester.pumpAndSettle();
      await tester.ensureVisible(save);
      await tester.pumpAndSettle();
      await tester.tap(save);
      await tester.pumpAndSettle();
      expect(written?['command'], {
        'operation': 'apply',
        'preview_id': 'synthetic-review',
        'reviewed': true,
      });
      expect(selected, 'cook-1');
      expect(c.pending, isEmpty);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
      f.controller.dispose();
    },
  );
  testWidgets(
    'guided cooking stays open with exact step ingredients and large German text',
    (tester) async {
      await tester.binding.setSurfaceSize(const Size(320, 740));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      final f = ContentFixture(), d = FakeReminders();
      Json? written;
      var revision = 0;
      f.respond = (r) async {
        if (r.url.path.endsWith('/cooking')) {
          if (r.method == 'POST') {
            written = object(jsonDecode(r.body));
            revision++;
            return cookedResponse({
              'revision': revision,
              'applied_revision': revision,
              'run_id': 'cook-1',
              'replayed': false,
            });
          }
          return cookedResponse(cookingList(revision));
        }
        if (r.url.path.endsWith('/cooking/cook-1')) {
          final value = cookingDetail(revision);
          if (written != null) {
            final at = written!['occurred_at'] as int;
            objects(object(value['run'])['steps']).first['timer'] = {
              'state': 'running',
              'started_at': at,
              'active_until': at,
              'deadline': at + 60,
            };
          }
          return cookedResponse(value);
        }
        return null;
      };
      await f.start(language: 'de');
      final c = CookingController(HouseholdApi(f.controller), device: d);
      await c.initialize();
      await c.refresh();
      await c.load('cook-1');
      await tester.pumpWidget(
        MaterialApp(
          locale: const Locale('de'),
          supportedLocales: const [Locale('en'), Locale('de')],
          localizationsDelegates: GlobalMaterialLocalizations.delegates,
          builder: (context, child) => MediaQuery(
            data: MediaQuery.of(context)
                .copyWith(textScaler: const TextScaler.linear(2)),
            child: child!,
          ),
          home: CookingDetailScreen(c, 'cook-1'),
        ),
      );
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.text('Synthetic step 1'),
        250,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      expect(find.text('Synthetic original instruction'), findsNWidgets(2));
      expect(find.byType(ExpansionTile), findsNothing);
      expect(find.textContaining('200,000001 g'), findsNWidgets(2));
      final start = find
          .widgetWithText(FilledButton, 'Arbeit fertig · Wartezeit starten')
          .first;
      await tester.scrollUntilVisible(
        start,
        250,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.tap(start);
      await tester.pumpAndSettle();
      expect(written?['command'], {
        'operation': 'timer',
        'run_id': 'cook-1',
        'step_id': 's1-1',
        'action': 'start',
      });
      expect(written?['occurred_at'], isA<int>());
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
      f.controller.dispose();
    },
  );
  test('a lost cooking response restores and retries the identical frozen intent after restart', () async {
    final f = ContentFixture(), device = FakeReminders();
    var revision = 0, lost = true;
    final writes = <Json>[];
    Json? receipt;
    f.respond = (r) async {
      if (r.url.path.endsWith('/cooking')) {
        if (r.method == 'POST') {
          final body = object(jsonDecode(r.body));
          writes.add(body);
          if (receipt == null) {
            revision++;
            receipt = body;
          }
          if (lost) {
            lost = false;
            return cookedResponse({'code': 'connection_failed'}, 503);
          }
          expect(body, receipt);
          return cookedResponse({
            'revision': revision,
            'applied_revision': 1,
            'replayed': true,
            'run_id': 'cook-1',
          });
        }
        return cookedResponse(cookingList(revision));
      }
      if (r.url.path.endsWith('/cooking/cook-1')) {
        return cookedResponse(cookingDetail(revision));
      }
      return null;
    };
    await f.start();
    final first = CookingController(HouseholdApi(f.controller), device: device);
    await first.initialize();
    await first.refresh();
    await first.load('cook-1');
    await first.enqueue({
      'operation': 'timer',
      'run_id': 'cook-1',
      'step_id': 's1-1',
      'action': 'start',
    });
    expect(first.pending.single['status'], 'unknown');
    expect(writes.length, 1);
    first.dispose();
    final restored = CookingController(
      HouseholdApi(f.controller),
      device: device,
    );
    await restored.initialize();
    expect(writes.length, 2);
    expect(writes.first, writes.last);
    expect(revision, 1);
    expect(restored.pending, isEmpty);
    expect(writes.first['occurred_at'], 1800000000);
    restored.dispose();
    f.controller.dispose();
  });
  test('independent offline timer intents persist exact order without claiming server confirmation', () async {
    final f = ContentFixture(), device = FakeReminders();
    f.respond = (r) async {
      if (r.url.path.endsWith('/cooking')) {
        return cookedResponse(cookingList(0));
      }
      if (r.url.path.endsWith('/cooking/cook-1')) {
        return cookedResponse(cookingDetail(0));
      }
      return null;
    };
    await f.start();
    final c = CookingController(HouseholdApi(f.controller), device: device);
    await c.initialize();
    await c.refresh();
    await c.load('cook-1');
    f.controller.online = false;
    for (final step in ['s1-1', 's2-1']) {
      await c.enqueue({
        'operation': 'timer',
        'run_id': 'cook-1',
        'step_id': step,
        'action': 'start',
      });
    }
    expect(c.pending.map((p) => object(p['request'])['expected_revision']), [
      0,
      1,
    ]);
    expect(c.pending.every((p) => p['attempted'] == false), true);
    expect(
      object(
        (object(c.detail('cook-1')!['run'])['steps'] as List).first,
      )['timer'],
      {'state': 'pending'},
    );
    expect(objects(f.controller.draft('cooking.v1')!['pending']).length, 2);
    expect(
      f.requests.where(
        (r) => r.method == 'POST' && r.url.path.endsWith('/cooking'),
      ),
      isEmpty,
    );
    c.dispose();
    f.controller.dispose();
  });
  test('a wall-clock edit preserves elapsed server time while a new boot needs revalidation', () async {
    final d = FakeReminders(), clock = CookingClock(FakeReminders());
    final anchored = CookingClock(d);
    await anchored.observe(1800000000);
    d.sample = const DeviceTime(1700000000000, 1060000, 'boot-a');
    await anchored.observe();
    expect(anchored.now, 1800000060);
    final original = clone(anchored.anchor!);
    d.sample = const DeviceTime(1800000000000, 5000, 'boot-b');
    await anchored.observe();
    expect(anchored.trusted, false);
    expect(anchored.now, isNull);
    expect(anchored.anchor, original);
    expect(clock.now, isNull);
  });
  test('local reminder identities track generation and never include private recipe content', () {
    final detail = cookingDetail(0);
    final original = reminderProjection(
      detail,
      'opaque-scope',
      'de',
      1800604800,
    );
    expect(original.length, 4);
    expect(
      original.every(
        (e) => e.keys.toSet().containsAll([
          'key',
          'due',
          'expires_at',
          'language',
        ]),
      ),
      true,
    );
    expect(jsonEncode(original).contains('Synthetic'), false);
    final steps = objects(object(detail['run'])['steps']);
    steps.first['generation'] = 2;
    final changed = reminderProjection(
      detail,
      'opaque-scope',
      'de',
      1800604800,
    );
    expect(changed.first['key'], isNot(original.first['key']));
    steps.first['timer'] = {
      'state': 'paused',
      'remaining': 60,
      'active_remaining': 0,
      'paused_at': 1800000000,
      'started_at': 1799999990,
    };
    expect(
      reminderProjection(detail, 'opaque-scope', 'de', 1800604800).length,
      2,
    );
    steps.last['timer'] = {
      'state': 'running',
      'started_at': 1800000000,
      'active_until': 1800000000,
      'deadline': 1800000000,
    };
    expect(
      reminderProjection(detail, 'opaque-scope', 'de', 1800604800),
      isEmpty,
    );
  });
  test(
    'denied access removes cooking copies, pending intents and local reminders',
    () async {
      final f = ContentFixture(), d = FakeReminders();
      var denied = false;
      f.respond = (r) async {
        if (r.url.path.endsWith('/cooking')) {
          return denied
              ? cookedResponse({'code': 'household_access_removed'}, 403)
              : cookedResponse(cookingList(0));
        }
        if (r.url.path.endsWith('/cooking/cook-1')) {
          return cookedResponse(cookingDetail(0));
        }
        return null;
      };
      await f.start();
      final c = CookingController(HouseholdApi(f.controller), device: d);
      await c.initialize();
      await c.refresh();
      await c.load('cook-1');
      await c.setReminders('cook-1', true);
      expect(d.entries, isNotEmpty);
      denied = true;
      await c.refresh();
      expect(c.sessions, isEmpty);
      expect(c.pending, isEmpty);
      expect(d.entries, isEmpty);
      expect(f.controller.draft('cooking.v1'), isNull);
      c.dispose();
      f.controller.dispose();
    },
  );
}
