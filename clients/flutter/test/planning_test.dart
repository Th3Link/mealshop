import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/planning/data.dart';
import 'package:mealshop/shopping/controller.dart';
import 'package:mealshop/ui/plan.dart';
import 'package:mealshop/ui/meal_editor.dart';
import 'package:mealshop/ui/week_proposal.dart';
import 'package:mealshop/ui/stock_review.dart';
import 'package:mealshop/ui/stock.dart';
import 'package:mealshop/ui/prepared_editor.dart';
import 'package:mealshop/ui/attendance.dart';

import 'content_fixtures.dart';

Json emptyPlan() => {
  'revision': 0,
  'profile_revision': 1,
  'meals': <Json>[],
  'recipe_revisions': <Json>[],
  'review_required_slots': <Json>[],
  'attendance': {'recurring': <Json>[], 'overrides': <Json>[]},
};
Json emptyStock() => {
  'revision': 0,
  'lots': <Json>[],
  'prepared': <Json>[],
  'offers': <Json>[],
  'allocations': <Json>[],
  'purchases': <Json>[],
  'movements': <Json>[],
  'prepared_history': <Json>[],
};
Json meal(String date, {String name = 'dinner'}) => {
  'date': date,
  'meal': name,
  'recipe': {'id': 'recipe-00000001', 'revision': 1},
  'attendees': ['one'],
  'calendar': null,
  'servings': 2,
  'active_seconds': 1800,
  'elapsed_seconds': 3600,
  'advance_seconds': 0,
};
http.Response response(Json value, [int status = 200]) => http.Response(
  jsonEncode(value),
  status,
  headers: {'content-type': 'application/json'},
);
Future<ContentFixture> fixture({
  Future<http.Response?> Function(http.Request)? respond,
  String role = 'owner',
}) async {
  final f = ContentFixture();
  f.people['members'] = <Json>[
    {
      'id': 'one',
      'name': 'Synthetic person',
      'allergies': <String>[],
      'allergen_ids': <String>[],
      'excluded_ingredients': <String>[],
      'disliked_ingredients': <String>[],
    },
  ];
  f.respond = (r) async {
    final custom = await respond?.call(r);
    if (custom != null) return custom;
    if (r.url.path.endsWith('/mealplan')) return response(emptyPlan());
    if (r.url.path.endsWith('/inventory')) return response(emptyStock());
    if (r.url.path.endsWith('/recipes')) {
      return response({
        'recipes': [
          {
            'id': 'recipe-00000001',
            'revision': 1,
            'title': 'Synthetic meal',
            'servings': 2,
          },
        ],
      });
    }
    return null;
  };
  await f.start(role: role);
  return f;
}

Widget app(Widget child, {String language = 'en', double scale = 1}) =>
    MaterialApp(
      locale: Locale(language),
      supportedLocales: const [Locale('en'), Locale('de')],
      localizationsDelegates: GlobalMaterialLocalizations.delegates,
      builder: (context, child) => MediaQuery(
        data: MediaQuery.of(context)
            .copyWith(textScaler: TextScaler.linear(scale)),
        child: child!,
      ),
      home: Scaffold(body: child),
    );
void main() {
  testWidgets(
    'deleted-person attendance references require explicit removal and retain empty overrides',
    (tester) async {
      Json? saved;
      final f = await fixture(
        respond: (r) async {
          if (r.url.path.endsWith('/mealplan/attendance')) {
            saved = object(jsonDecode(r.body));
            return response({'revision': 1});
          }
          return null;
        },
      );
      final api = HouseholdApi(f.controller),
          data = await PlanningData.load(HouseholdApi(f.controller));
      data.plan['attendance'] = {
        'recurring': <Json>[
          {
            'member_id': 'deleted',
            'weekdays': [0],
            'meals': ['dinner'],
          },
        ],
        'overrides': <Json>[
          {
            'date': '2026-11-09',
            'meal': 'dinner',
            'member_ids': ['deleted'],
          },
        ],
      };
      await tester.pumpWidget(app(AttendanceScreen(api, data)));
      await tester.pumpAndSettle();
      expect(saved, isNull);
      await tester.tap(find.text('Remove obsolete attendance entries'));
      await tester.pumpAndSettle();
      final save = find.text('Save');
      await tester.scrollUntilVisible(
        save,
        400,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(save);
      await tester.pumpAndSettle();
      expect(
        objects(object(saved!['attendance'])['recurring'])
            .any((r) => r['member_id'] == 'deleted'),
        false,
      );
      expect(
        objects(object(saved!['attendance'])['overrides']).single['member_ids'],
        isEmpty,
      );
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'physical prepared detail shows six four two without duplicate production at large text',
    (tester) async {
      final stock = emptyStock();
      stock['prepared'] = <Json>[
        {
          'held': 4,
          'used': 0,
          'available': 2,
          'batch': {
            'id': 'stock-op-1',
            'title': 'Synthetic meal',
            'recipe_id': 'recipe-00000001',
            'revision': 1,
            'original_portions': 6,
            'produced_on': '2026-11-14',
            'source': null,
            'reversed': false,
            'storage': {
              'location': 'Synthetic freezer',
              'condition': 'frozen',
              'declared_on': '2026-11-14',
              'use_by': null,
            },
            'allocations': <Json>[],
          },
        },
      ];
      final f = await fixture(
        role: 'reader',
        respond: (r) async =>
            r.url.path.endsWith('/inventory') ? response(stock) : null,
      );
      await tester.binding.setSurfaceSize(const Size(320, 800));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        app(PreparedDetail(HouseholdApi(f.controller), 'stock-op-1'), scale: 2),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      expect(
        find.textContaining('Actual portions produced: 6'),
        findsOneWidget,
      );
      expect(
        find.textContaining('Allocated'),
        findsNothing,
      ); // no invented per-meal hold rows
      expect(find.text('Allocate portions'), findsNothing);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'recording observed prepared output previews the declared yield without ingredient movement',
    (tester) async {
      Json? input;
      final f = await fixture(
        respond: (r) async {
          if (r.url.path.endsWith('/inventory/preview')) {
            input = object(jsonDecode(r.body));
            return response({
              'changes': <Json>[],
              'shopping': <Json>[],
              'prepared_before': <Json>[],
              'prepared_after': <Json>[],
              'movements': <Json>[],
              'offers': <Json>[],
            });
          }
          return null;
        },
      );
      final api = HouseholdApi(f.controller),
          data = await PlanningData.load(HouseholdApi(f.controller));
      await tester.pumpWidget(app(PreparedEditor(api, data)));
      await tester.pumpAndSettle();
      final yieldField = find.byKey(const ValueKey('actual_yield'));
      await tester.scrollUntilVisible(
        yieldField,
        200,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.enterText(yieldField, '6');
      final location = find.byKey(const ValueKey('storage_location'));
      await tester.scrollUntilVisible(
        location,
        200,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.enterText(location, 'Synthetic freezer');
      final button = find.text('Review stock changes');
      await tester.scrollUntilVisible(
        button,
        300,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(button);
      await tester.pumpAndSettle();
      expect(object(object(input!['command'])['input'])['portions'], 6);
      expect(input!['reviewed'], false);
      expect(
        f.requests.where(
          (r) => r.method == 'POST' && r.url.path.endsWith('/inventory'),
        ),
        isEmpty,
      );
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'stock ingredient balances use exact localized decimals without padded zeros',
    (tester) async {
      final stock = emptyStock();
      stock['lots'] = <Json>[
        {
          'id': 'lot-1',
          'binding': {'name': 'Synthetic oats'},
          'available': '100.000001',
          'remaining': '100.000001',
          'reserved': '0.000000',
          'unit': 'g',
          'metadata': {'location': 'Synthetic pantry', 'expiry': null},
        },
      ];
      final f = await fixture(
        respond: (r) async =>
            r.url.path.endsWith('/inventory') ? response(stock) : null,
      );
      await tester.pumpWidget(
        app(StockScreen(HouseholdApi(f.controller)), language: 'de'),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Fertige Portionen'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Zutaten & Produkte').last);
      await tester.pumpAndSettle();
      expect(find.textContaining('100,000001'), findsOneWidget);
      expect(find.textContaining('0,000000'), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  test(
    'civil week navigation preserves leap dates and DST calendar boundaries',
    () {
      expect(isoDay(addDays(civilDay('2026-03-28'), 2)), '2026-03-30');
      expect(isoDay(addDays(civilDay('2026-10-24'), 2)), '2026-10-26');
      expect(isoDay(addDays(civilDay('2028-02-28'), 1)), '2028-02-29');
      expect(isoDay(weekStart(civilDay('2026-11-14'))), '2026-11-09');
      expect(() => civilDay('2026-02-29'), throwsA(isA<ApiFailure>()));
    },
  );
  test(
    'selected destination survives protected restore and resets with sign out',
    () async {
      final f = await fixture();
      await f.controller.setDestination('plan');
      final restored = ShoppingController(f.controller.api, f.vault);
      await restored.initialize(deviceLanguage: 'en');
      expect(restored.destination, 'plan');
      await restored.setDestination('invented');
      expect(restored.destination, 'plan');
      await restored.signOut();
      expect(restored.destination, 'shopping');
      restored.dispose();
      f.controller.dispose();
    },
  );
  testWidgets(
    'day and week remain usable at 320dp and 200 percent with reader actions',
    (tester) async {
      final f = await fixture(role: 'reader');
      await tester.binding.setSurfaceSize(const Size(320, 800));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        app(
          PlanScreen(HouseholdApi(f.controller), shopping: () {}),
          language: 'de',
          scale: 2,
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await tester.scrollUntilVisible(
        find.text('Frühstück'),
        300,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.text('Frühstück'), findsOneWidget);
      expect(find.text('Woche vorschlagen'), findsNothing);
      await tester.drag(find.byType(ListView).first, const Offset(0, -600));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      expect(
        f.requests.where(
          (r) => r.method == 'POST' && r.url.path.contains('/mealplan'),
        ),
        isEmpty,
      );
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'meal review preserves a conflict draft and sends the reviewed attendees exactly',
    (tester) async {
      const date = '2026-11-09';
      Json? submitted;
      final f = await fixture(
        respond: (r) async {
          if (r.url.path.endsWith('/mealplan/preview')) {
            submitted = object(jsonDecode(r.body));
            return response({
              'plan': submitted,
              'plan_revision': 0,
              'profile_revision': 1,
              'feasible': true,
              'conflicts': <String>[],
              'nutrition': nutrition(),
              'review': {'status': 'review_required'},
              'shopping_delta': <Json>[],
            });
          }
          if (r.url.path.endsWith('/mealplan/apply')) {
            return response({'code': 'revision_conflict'}, 409);
          }
          return null;
        },
      );
      final api = HouseholdApi(f.controller),
          data = await PlanningData.load(HouseholdApi(f.controller));
      await tester.pumpWidget(
        app(MealEditor(api, data, date: date, meal: 'dinner')),
      );
      await tester.pumpAndSettle();
      final review = find.text('Review meal');
      await tester.scrollUntilVisible(
        review,
        400,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(review);
      await tester.pumpAndSettle();
      expect(submitted?['servings'], 1);
      expect(submitted?['attendees'], ['one']);
      final checked = find.text(
        'I reviewed the participants, recipe and quantities.',
      );
      await tester.scrollUntilVisible(
        checked,
        400,
        scrollable: find.byType(Scrollable).last,
      );
      await tester.tap(checked);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Apply reviewed change'));
      await tester.pumpAndSettle();
      expect(
        f.requests.where((r) => r.url.path.endsWith('/mealplan/apply')),
        hasLength(1),
      );
      expect(submitted?['recipe'], {'id': 'recipe-00000001', 'revision': 1});
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'whole week needs every feasible slot confirmation and keeps impossible slots explicit',
    (tester) async {
      Json? applied;
      final f = await fixture(
        respond: (r) async {
          if (r.url.path.endsWith('/week/apply')) {
            applied = object(jsonDecode(r.body));
            return response({'revision': 1});
          }
          return null;
        },
      );
      final api = HouseholdApi(f.controller),
          data = await PlanningData.load(HouseholdApi(f.controller));
      final slots = <Json>[
        for (final d in ['2026-11-09', '2026-11-10'])
          {
            'date': d,
            'meal': 'dinner',
            'plan': meal(d),
            'evidence': {
              'prior_week_uses': 0,
              'reused_ingredients': 1,
              'active_seconds': 60,
            },
          },
        {
          'date': '2026-11-11',
          'meal': 'dinner',
          'plan': null,
          'reason': 'no_feasible_meal',
          'evidence': {
            'rejected_by_constraint': {'dietary_exclusion': 1},
          },
        },
      ];
      await tester.pumpWidget(
        app(
          WeekReview(api, data, {
            'proposal': {
              'id': 'week-1',
              'expires_at':
                  DateTime.now().millisecondsSinceEpoch ~/ 1000 + 1800,
              'slots': slots,
            },
            'shopping_delta': <Json>[],
          }),
        ),
      );
      await tester.pumpAndSettle();
      expect(applied, isNull);
      for (var i = 0; i < 2; i++) {
        final box = find.byType(CheckboxListTile).at(i);
        await tester.ensureVisible(box);
        await tester.tap(box);
        await tester.pumpAndSettle();
      }
      final button = find.text('Save reviewed week');
      await tester.scrollUntilVisible(
        button,
        400,
        scrollable: find.byType(Scrollable).last,
      );
      await tester.tap(button);
      await tester.pumpAndSettle();
      expect(applied?['proposal_id'], 'week-1');
      expect(applied?['confirmed_slots'], hasLength(2));
      expect(applied!.containsKey('plans'), isFalse);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets('unknown stock result retries the exact frozen key and body', (
    tester,
  ) async {
    final bodies = <Json>[];
    final f = await fixture(
      respond: (r) async {
        if (r.url.path.endsWith('/inventory') && r.method == 'POST') {
          bodies.add(object(jsonDecode(r.body)));
          return response({'code': 'temporary_failure'}, 503);
        }
        return null;
      },
    );
    final input = <String, dynamic>{
      'expected_revision': 0,
      'idempotency_key': 'stock-immutable-0001',
      'reviewed': false,
      'command': {
        'operation': 'release_prepared',
        'batch_id': 'stock-op-1',
        'allocation_id': 'stock-op-2',
      },
    };
    await tester.pumpWidget(
      app(
        StockReview(HouseholdApi(f.controller), input, {
          'changes': <Json>[],
          'shopping': <Json>[],
          'movements': <Json>[],
          'offers': <Json>[],
          'prepared_before': <Json>[],
          'prepared_after': <Json>[],
        }),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pumpAndSettle();
    for (var i = 0; i < 2; i++) {
      final button = find.text(
        i == 0 ? 'Apply reviewed change' : 'Retry the same booking',
      );
      await tester.scrollUntilVisible(
        button,
        400,
        scrollable: find.byType(Scrollable).last,
      );
      await tester.tap(button);
      await tester.pumpAndSettle();
      await tester.pump(const Duration(seconds: 5));
      await tester.pumpAndSettle();
    }
    expect(bodies, hasLength(2));
    expect(bodies[0], bodies[1]);
    expect(bodies[0]['reviewed'], true);
    await tester.pumpWidget(const SizedBox());
    f.controller.dispose();
  });
}
