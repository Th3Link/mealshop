import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/ui/activity.dart';
import 'package:mealshop/ui/ingredient_picker.dart';
import 'package:mealshop/ui/person_tour.dart';
import 'package:mealshop/ui/recipe_editor.dart';
import 'package:mealshop/ui/recipes.dart';
import 'package:mealshop/ui/common.dart';
import 'package:mealshop/ui/ingredient_editor.dart';
import 'package:mealshop/ui/sharing.dart';

import 'content_fixtures.dart';

Widget app(Widget screen, {String language = 'en'}) => MaterialApp(
  locale: Locale(language),
  supportedLocales: const [Locale('en'), Locale('de')],
  localizationsDelegates: GlobalMaterialLocalizations.delegates,
  home: screen,
);
void main() {
  final previousError = FlutterError.onError;
  FlutterError.onError = (details) {
    debugPrint(details.toString());
    previousError?.call(details);
  };
  test('exact display preserves all normalized digits and localized aliases resolve', () {
    expect(decimal('0.200000001', 'de'), '0,200000001');
    expect(decimal('200.000000', 'de'), '200');
    expect(inputDecimal('200.000001', 'de'), '200,000001');
    expect(exactInput('200,000001', 'de'), '200.000001');
    expect(() => exactInput('1.000', 'de'), throwsA(isA<ApiFailure>()));
    expect(() => exactInput('1,000', 'en'), throwsA(isA<ApiFailure>()));
    expect(Catalogue(catalogue()).search('hafer', 'de').single['id'], 'oats');
    expect(Catalogue(catalogue()).search('synthetic alias', 'en').length, 2);
  });
  testWidgets(
    'pending household invitations render without content or a selected household',
    (tester) async {
      final fixture = ContentFixture();
      await fixture.start();
      fixture.controller.selected = null;
      fixture.shared = {
        'items': [
          {
            'share': {
              'id': 'share-1',
              'source_home': 'home-7',
              'owner': 'Synthetic owner',
              'pending': true,
              'version': 1,
              'resource_kind': 'recipe',
            },
            'content': null,
          },
        ],
        'total': 1,
        'page': 1,
        'per_page': 50,
      };
      await tester.pumpWidget(
        app(SharedScreen(HouseholdApi(fixture.controller))),
      );
      await tester.pumpAndSettle();
      expect(find.text('Household invitation'), findsOneWidget);
      expect(tester.takeException(), isNull);
      expect(
        fixture.requests.any(
          (request) => request.url.path.endsWith('/shared/home-7/share-1'),
        ),
        isFalse,
      );
      await tester.pumpWidget(const SizedBox());
      fixture.controller.dispose();
    },
  );
  testWidgets(
    'confirmed save closes a dirty editor without discard and retains its result',
    (tester) async {
      bool dirty = true;
      String? result;
      await tester.pumpWidget(
        MaterialApp(
          home: StatefulBuilder(
            builder: (context, update) {
              Future<void> openEditor() async {
                final editor = Builder(
                  builder: (editorContext) => DraftGuard(
                    dirty: () => dirty,
                    child: Scaffold(
                      body: FilledButton(
                        onPressed: () {
                          dirty = false;
                          Navigator.pop(editorContext, 'saved-recipe-id');
                        },
                        child: const Text('Save editor'),
                      ),
                    ),
                  ),
                );
                final saved = await Navigator.push<String>(
                  context,
                  MaterialPageRoute(builder: (_) => editor),
                );
                if (context.mounted) update(() => result = saved);
              }

              return Scaffold(
                body: Column(
                  children: [
                    Text(result ?? 'No saved result'),
                    FilledButton(
                      onPressed: openEditor,
                      child: const Text('Open editor'),
                    ),
                  ],
                ),
              );
            },
          ),
        ),
      );
      await tester.tap(find.text('Open editor'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Save editor'));
      await tester.pumpAndSettle();
      expect(find.text('Discard this draft?'), findsNothing);
      expect(find.text('saved-recipe-id'), findsOneWidget);
      expect(find.text('Save editor'), findsNothing);
      await tester.pumpWidget(const SizedBox());
    },
  );
  testWidgets(
    'German ingredient edits preserve exact range and constituent decimals',
    (tester) async {
      final fixture = ContentFixture();
      await fixture.start(language: 'de');
      final item = objects(fixture.foods['ingredients']).first;
      object(item['organization'])['nutrition'] = {
        'mode': 'own',
        'values': {
          'protein': {
            'amount': null,
            'range': {'lower': '1.000001', 'upper': '2.500000'},
            'uncertainty': 'Synthetic declared range',
          },
        },
        'constituents': {
          'synthetic': {
            'name': 'Synthetic constituent',
            'unit': 'g',
            'amount': '0.125000',
            'translations': <String, dynamic>{},
          },
        },
        'note': 'Synthetic declaration',
        'source_url': '',
      };
      await tester.pumpWidget(
        app(
          IngredientEditor(HouseholdApi(fixture.controller), id: 'oats'),
          language: 'de',
        ),
      );
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.byKey(const ValueKey('range_lower')),
        300,
        scrollable: find
            .byWidgetPredicate(
              (w) => w is Scrollable && w.axisDirection == AxisDirection.down,
            )
            .first,
      );
      final field = find.descendant(
        of: find.byKey(const ValueKey('range_lower')),
        matching: find.byType(EditableText),
      );
      expect(tester.widget<EditableText>(field).controller.text, '1,000001');
      await tester.scrollUntilVisible(
        find.widgetWithText(FilledButton, 'Speichern'),
        500,
        scrollable: find
            .byWidgetPredicate(
              (w) => w is Scrollable && w.axisDirection == AxisDirection.down,
            )
            .first,
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Speichern'));
      await tester.pumpAndSettle();
      final saved = object(
        object(
          jsonDecode(fixture.requests.lastWhere((r) => r.method == 'PUT').body),
        )['annotations'],
      );
      final nutrition = object(object(saved['organization'])['nutrition']);
      expect(object(object(nutrition['values'])['protein'])['range'], {
        'lower': '1.000001',
        'upper': '2.500000',
      });
      expect(
        object(object(nutrition['constituents'])['synthetic'])['amount'],
        '0.125000',
      );
      await tester.pumpWidget(const SizedBox());
      fixture.controller.dispose();
    },
  );
  testWidgets(
    'ingredient search keeps focus and offers inline creation for no match',
    (tester) async {
      final f = ContentFixture();
      await f.start();
      await tester.pumpWidget(
        app(IngredientSearch(HouseholdApi(f.controller), create: true)),
      );
      await tester.pumpAndSettle();
      final input = find.byType(EditableText);
      await tester.enterText(input, 'hafer');
      await tester.pumpAndSettle();
      expect(find.text('Oats'), findsOneWidget);
      expect(tester.widget<EditableText>(input).focusNode.hasFocus, isTrue);
      await tester.enterText(input, 'not in catalogue');
      await tester.pumpAndSettle();
      expect(find.text('Create ingredient'), findsOneWidget);
      expect(find.text('Nothing found'), findsOneWidget);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'person tour resumes a minimized draft and saves the reviewed rules',
    (tester) async {
      final f = ContentFixture();
      await f.start();
      final api = HouseholdApi(f.controller);
      await tester.pumpWidget(app(PersonTour(api)));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('person_name')),
        'Synthetic person',
      );
      await tester.pump(const Duration(milliseconds: 400));
      await tester.pumpAndSettle();
      expect(
        f.controller.draft('person:new')!['person']['name'],
        'Synthetic person',
      );
      expect(
        f.controller.draft('person:new')!.containsKey('profiles'),
        isFalse,
      );
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      await tester.pumpWidget(const SizedBox());
      await tester.pumpWidget(app(PersonTour(api)));
      await tester.pumpAndSettle();
      expect(find.text('2 / 5 · Dietary rules'), findsOneWidget);
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      await tester.tap(find.widgetWithText(FilledButton, 'Save'));
      await tester.pumpAndSettle();
      expect(objects(f.people['members']).single['name'], 'Synthetic person');
      expect(
        object(objects(f.people['members']).single['rules'])['version'],
        1,
      );
      expect(f.controller.draft('person:new'), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'multipart recipe edit preserves exact input, original language and links',
    (tester) async {
      final f = ContentFixture();
      await f.start(language: 'de');
      final record = recipeRecord();
      object(record['recipe'])['parts'] = [
        {'id': 'base', 'name': 'Synthetic base'},
      ];
      final line = objects(object(record['recipe'])['lines']).single;
      line['part_id'] = 'base';
      final step = objects(object(record['recipe'])['steps']).single;
      step['part_id'] = 'base';
      await tester.pumpWidget(
        app(
          RecipeEditor(HouseholdApi(f.controller), record: record),
          language: 'de',
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('title')),
        'Synthetic updated',
      );
      await tester.scrollUntilVisible(
        find.widgetWithText(FilledButton, 'Speichern'),
        500,
        scrollable: find
            .byWidgetPredicate(
              (w) => w is Scrollable && w.axisDirection == AxisDirection.down,
            )
            .first,
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Speichern'));
      await tester.pumpAndSettle();
      final body = object(
        jsonDecode(f.requests.lastWhere((r) => r.method == 'PUT').body),
      );
      final saved = object(body['recipe']);
      expect(saved['language'], 'en');
      expect(objects(saved['lines']).single['quantity'], {
        'amount': '200.000001',
        'unit': 'g',
      });
      expect(objects(saved['steps']).single['ingredient_line_ids'], ['line-1']);
      expect(objects(saved['steps']).single['part_id'], 'base');
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  for (final language in ['en', 'de']) {
    testWidgets(
      'recipe ranges and exact amounts fit 320dp at 200% text in $language',
      (tester) async {
        await tester.binding.setSurfaceSize(const Size(320, 740));
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
        final f = ContentFixture();
        await f.start(language: language, role: 'reader');
        final semantics = tester.ensureSemantics();
        await tester.pumpWidget(
          app(
            RecipeDetail(HouseholdApi(f.controller), 'recipe-00000001'),
            language: language,
          ),
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        await tester.scrollUntilVisible(
          find.text(language == 'de' ? '200,000001 g' : '200.000001 g'),
          150,
        );
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        await tester.scrollUntilVisible(find.text('2–3'), 200);
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
        semantics.dispose();
        await tester.pumpWidget(const SizedBox());
        f.controller.dispose();
        await tester.binding.setSurfaceSize(null);
      },
    );
  }
  testWidgets(
    'activity without weight saves 45-minute running and a revoked grant clears the form',
    (tester) async {
      final f = ContentFixture();
      await f.start(role: 'reader');
      final api = HouseholdApi(f.controller);
      await tester.pumpWidget(
        app(ActivityEditor(api, '/people/person/activity', f.activity)),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Walking'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Running').last);
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('duration_minutes')),
        '45',
      );
      await tester.scrollUntilVisible(
        find.widgetWithText(FilledButton, 'Save'),
        200,
        scrollable: find
            .byWidgetPredicate(
              (w) => w is Scrollable && w.axisDirection == AxisDirection.down,
            )
            .first,
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Save'));
      await tester.pumpAndSettle();
      final body = object(
        jsonDecode(f.requests.lastWhere((r) => r.method == 'PUT').body),
      );
      expect(object(body['entry'])['duration_seconds'], 2700);
      expect(object(body['entry'])['kind'], 'running');
      expect(object(body['entry']).containsKey('weight'), isFalse);
      expect(object(body['entry']).containsKey('kcal'), isFalse);
      await tester.pumpWidget(const SizedBox());
      await tester.pumpWidget(
        app(ActivityEditor(api, '/people/person/activity', f.activity)),
      );
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('notes')),
        'Synthetic private note',
      );
      f.activityDenied = true;
      await tester.pump(const Duration(seconds: 21));
      await tester.pumpAndSettle();
      expect(find.text('Synthetic private note'), findsNothing);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
}
