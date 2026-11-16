import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/cooking/controller.dart';
import 'package:mealshop/brewing/data.dart';
import 'package:mealshop/ui/brewing_reading.dart';
import 'package:mealshop/ui/brewing_detail.dart';
import 'package:mealshop/ui/messages.dart';

import 'content_fixtures.dart';
import 'cooking_test.dart'
    show FakeReminders, cookingList, cookingDetail, cookedResponse;

Json brewCatalogue() => {
  'schema_version': 1,
  'retrieved_on': '2026-11-14',
  'sources': [
    {
      'id': 'synthetic-reference',
      'publisher': 'Synthetic publisher',
      'title': 'Synthetic method reference',
      'url': 'https://example.org/synthetic-method',
      'document_version': '1',
      'retrieved_on': '2026-11-14',
    },
  ],
  'phases': {
    'must': {
      'kind': 'must',
      'name': 'Synthetic must',
      'translations': {'de': 'Synthetischer Ansatz'},
      'instruction': 'Synthetic original must instruction',
      'instruction_translations': {'de': 'Synthetische Anleitung zum Ansatz'},
    },
    'ferment': {
      'kind': 'ferment',
      'name': 'Synthetic fermentation',
      'translations': {'de': 'Synthetische Gärung'},
      'instruction': 'Synthetic original fermentation instruction',
      'instruction_translations': <String, dynamic>{},
    },
  },
  'templates': [
    {
      'id': 'synthetic-mead',
      'version': 1,
      'beverage': 'mead',
      'name': 'Synthetic mead',
      'translations': {'de': 'Synthetischer Met'},
      'process': 'traditional_mead',
      'sources': ['synthetic-reference'],
      'phases': ['must', 'ferment'],
    },
  ],
};
Json brewRecipe() {
  final catalogue = brewCatalogue();
  return seedBrew(
      catalogue,
      objects(catalogue['templates']).single,
      'en',
      'Synthetic household phase',
    )
    ..['title'] = 'Synthetic mead recipe'
    ..['volume'] = {'amount': '5.000001', 'unit': 'l'};
}

Json brewRun() {
  final recipe = brewRecipe(), run = clone(object(cookingDetail(0)['run']));
  run.remove('recipe');
  run.remove('servings');
  run['kind'] = 'mead';
  run['name'] = 'Synthetic independent batch';
  run['brew_recipe'] = {
    'id': 'brew-recipe-1',
    'revision': 1,
    'title': recipe['title'],
    'language': 'en',
    'beverage': 'mead',
    'volume': recipe['volume'],
  };
  run['steps'] = [
    for (final p in objects(recipe['phases']))
      {
        'stage': {
          ...clone(object(p['stage'])),
          'id': '${object(p['stage'])['id']}-1',
        },
        'timer': {'state': 'pending'},
        'cycle': 1,
        'lag_seconds': 0,
        'generation': 1,
        'ready': object(p['stage'])['id'] == 'phase-1',
        'available_at': object(p['stage'])['id'] == 'phase-1'
            ? 1800000000
            : null,
      },
  ];
  run['schedule'] = [
    for (final step in objects(run['steps']))
      {
        'id': object(step['stage'])['id'],
        'start': 1800000010,
        'active_until': 1800000010,
        'finish': 1800000010,
        'earliest': 1800000010,
        'latest': 1800000010,
      },
  ];
  return run;
}

Json brewReading() => {
  'id': 'brew-reading-1',
  'run_id': 'cook-1',
  'input': {
    'phase_id': null,
    'measurement': {'value': '1.053000', 'unit': 'sg'},
    'measured_at': {
      'local': '2027-01-15T08:00',
      'timezone': 'UTC',
      'fold': 'reject',
    },
    'source': {
      'kind': 'instrument',
      'device_id': 'synthetic-hydrometer',
      'name': 'Synthetic instrument',
      'acquisition': 'reported',
    },
    'calibration': {'kind': 'unknown'},
    'notes': 'Synthetic original reading',
  },
  'measured_instant': 1799999900,
  'actor': 'alice',
  'recorded_at': 1800000000,
  'corrects': null,
  'reason': null,
};
Json brewDetail(int revision) => {
  'revision': revision,
  'server_now': 1800000000,
  'notification_transport': false,
  'scheduler': {'last_check_at': 1800000000, 'error': null, 'writable': true},
  'run': brewRun(),
  'batch': {
    'draft': {
      'name': 'Synthetic independent batch',
      'recipe_id': 'brew-recipe-1',
      'recipe_revision': 1,
      'brew_day': {
        'local': '2027-01-15T08:00',
        'timezone': 'UTC',
        'fold': 'reject',
      },
      'yeast_preparation': {'choice': 'skip'},
      'equipment': <String, dynamic>{},
    },
    'equipment': <Json>[],
    'actor': 'alice',
    'recorded_at': 1800000000,
    'archived': false,
  },
  'recipe': {
    'id': 'brew-recipe-1',
    'revision': 1,
    'recipe': brewRecipe(),
    'actor': 'alice',
    'recorded_at': 1800000000,
  },
  'notices': <Json>[],
  'measurements': {
    'readings': [brewReading()],
    'next': null,
    'total': 1,
  },
};
Json brewState(int revision) => {
  'revision': revision,
  'server_now': 1800000000,
  'notification_transport': false,
  'scheduler': {'last_check_at': 1800000000, 'error': null, 'writable': true},
  'recipes': [
    {'record': brewDetail(revision)['recipe'], 'archived': false},
  ],
  'equipment': <Json>[],
  'batches': [
    {
      'run': brewRun(),
      'batch': brewDetail(revision)['batch'],
      'reading_count': 1,
    },
  ],
};
Widget brewingApp(Widget child, {String language = 'de'}) => MaterialApp(
  locale: Locale(language),
  supportedLocales: const [Locale('en'), Locale('de')],
  localizationsDelegates: GlobalMaterialLocalizations.delegates,
  home: child,
);

void main() {
  test('source-linked fermentation seed retains unknowns and never inherits grain assumptions', () {
    final catalogue = brewCatalogue(),
        recipe = seedBrew(
          catalogue,
          objects(catalogue['templates']).single,
          'de',
          'Synthetic phase source',
        );
    expect(recipe['method'], {'process': 'traditional_mead'});
    expect(recipe['beverage'], 'mead');
    expect(object(recipe['volume'])['amount'], '');
    expect(
      objects(recipe['phases']).every(
        (p) =>
            p['timing_known'] == false &&
            p['temperature'] == null &&
            p['volume'] == null,
      ),
      true,
    );
    expect(
      object(objects(recipe['phases']).first['stage'])['instruction'],
      'Synthetische Anleitung zum Ansatz',
    );
    expect(
      object(objects(recipe['phases']).last['stage'])['instruction'],
      'Synthetic original fermentation instruction',
    );
    expect(
      objects(object(recipe['template'])['sources']).single['url'],
      'https://example.org/synthetic-method',
    );
  });
  test('a lost reviewed brewing response survives restart without duplicating a batch', () async {
    final f = ContentFixture(), d = FakeReminders();
    var revision = 0, lost = true;
    Json? original;
    final writes = <Json>[];
    f.respond = (r) async {
      if (r.url.path.endsWith('/cooking')) {
        return cookedResponse(cookingList(revision));
      }
      if (r.url.path.endsWith('/brewing')) {
        if (r.method == 'POST') {
          final input = object(jsonDecode(r.body));
          writes.add(input);
          if (original == null) {
            original = input;
            revision++;
          }
          expect(input, original);
          if (lost) {
            lost = false;
            return cookedResponse({'code': 'connection_failed'}, 503);
          }
          return cookedResponse({
            'operation_id': 'brew-operation-1',
            'result_id': 'cook-2',
            'replayed': true,
            'brewing': brewState(revision),
          });
        }
        return cookedResponse(brewState(revision));
      }
      return null;
    };
    await f.start();
    final c = CookingController(HouseholdApi(f.controller), device: d);
    await c.initialize();
    await c.refresh();
    final input = <String, dynamic>{
      'expected_revision': 0,
      'idempotency_key': 'synthetic-brew-frozen-0001',
      'reviewed': true,
      'command': {
        'operation': 'create_batch',
        'draft': object(brewDetail(0)['batch'])['draft'],
      },
    };
    await c.submitBrewing(input);
    expect(c.pending.single['status'], 'unknown');
    c.dispose();
    final restored = CookingController(HouseholdApi(f.controller), device: d);
    await restored.initialize();
    expect(writes, [input, input]);
    expect(revision, 1);
    expect(restored.pending, isEmpty);
    expect(restored.lastResult, 'cook-2');
    restored.dispose();
    f.controller.dispose();
  });
  testWidgets(
    'German reading correction keeps exact units and original evidence until review',
    (tester) async {
      final f = ContentFixture(), d = FakeReminders();
      Json? previewed;
      f.respond = (r) async {
        if (r.url.path.endsWith('/cooking')) {
          return cookedResponse(cookingList(0));
        }
        if (r.url.path.endsWith('/brewing')) {
          return cookedResponse(brewState(0));
        }
        if (r.url.path.endsWith('/brewing/preview')) {
          previewed = object(jsonDecode(r.body));
          return cookedResponse({
            'expected_revision': 0,
            'before': brewState(0),
            'after': brewState(1),
          });
        }
        if (r.url.path.endsWith('/readings')) {
          return cookedResponse({
            'readings': [brewReading()],
            'next': null,
            'total': 1,
          });
        }
        return null;
      };
      await f.start(language: 'de');
      final c = CookingController(HouseholdApi(f.controller), device: d);
      await c.initialize();
      await c.refresh();
      await tester.pumpWidget(
        brewingApp(BrewingReadingEditor(c, brewRun(), original: brewReading())),
      );
      await tester.pumpAndSettle();
      final value = find.byKey(const ValueKey('brewReadingValue')),
          reason = find.byKey(const ValueKey('brewCorrectionReason'));
      await tester.scrollUntilVisible(
        value,
        250,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.enterText(value, '1,054001');
      await tester.scrollUntilVisible(
        reason,
        300,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.enterText(reason, 'Synthetic transcription correction');
      FocusManager.instance.primaryFocus?.unfocus();
      await tester.pumpAndSettle();
      final review = find.widgetWithText(
        FilledButton,
        message('brewReview', 'de'),
      );
      await tester.scrollUntilVisible(
        review,
        300,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.tap(review);
      await tester.pumpAndSettle();
      final command = object(previewed!['command']),
          reading = object(command['reading']);
      expect(reading['measurement'], {'value': '1.054001', 'unit': 'sg'});
      expect(command['corrects'], 'brew-reading-1');
      expect(object(reading['source'])['acquisition'], 'transcribed');
      expect(reading['calibration'], {'kind': 'unknown'});
      expect(
        f.requests.where(
          (r) => r.url.path.endsWith('/brewing') && r.method == 'POST',
        ),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
      f.controller.dispose();
    },
  );
  testWidgets(
    'a reader can inspect batch phases and measurements without operating equipment',
    (tester) async {
      final f = ContentFixture(), d = FakeReminders();
      f.respond = (r) async {
        if (r.url.path.endsWith('/cooking')) {
          return cookedResponse(cookingList(0));
        }
        if (r.url.path.endsWith('/brewing/batches/cook-1')) {
          return cookedResponse(brewDetail(0));
        }
        return null;
      };
      await f.start(role: 'reader');
      final c = CookingController(HouseholdApi(f.controller), device: d);
      await c.initialize();
      await c.refresh();
      await c.load('cook-1', brewing: true);
      await tester.pumpWidget(
        brewingApp(BrewingDetailScreen(c, 'cook-1'), language: 'en'),
      );
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.text('Synthetic must'),
        250,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      expect(find.text('Synthetic original must instruction'), findsOneWidget);
      expect(
        tester
            .widgetList<FilledButton>(find.byType(FilledButton))
            .every((b) => b.onPressed == null),
        true,
      );
      expect(f.requests.where((r) => r.method == 'POST'), isEmpty);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
      f.controller.dispose();
    },
  );
}
