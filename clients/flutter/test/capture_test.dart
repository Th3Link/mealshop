import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:mealshop/capture/import.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/ui/recipe_editor.dart';
import 'package:mealshop/ui/label_capture.dart';
import 'package:mealshop/ui/capture_classify.dart';

import 'content_fixtures.dart';
import 'people_library_test.dart' show app;

Json preview() => {
  'preview_id': 'import-${'a' * 64}',
  'expires_at': 1,
  'requested_url': 'https://example.org/original',
  'final_url': null,
  'method': 'manual',
  'retrieved_at': null,
  'candidates': [
    {
      ...blankCandidate(),
      'title': 'Synthetic source',
      'language': 'de',
      'ingredients': ['1 cup oats', '0,250001 g salt'],
      'steps': [
        {'text': 'Mix', 'section': null},
      ],
    },
  ],
};
void main() {
  test('metric suggestions keep exact precision and leave cups ranges and separators unresolved', () {
    expect(metricSuggestion('0,250001 g Salz', 'de'), {
      'amount': '0.250001',
      'unit': 'g',
    });
    expect(metricSuggestion('1 cup oats', 'en'), isNull);
    expect(metricSuggestion('1-2 g salt', 'en'), isNull);
    expect(metricSuggestion('1,000 g salt', 'en'), isNull);
    expect(metricSuggestion('1.000 g salt', 'de'), isNull);
    expect(metricSuggestion('50 ml water', 'en'), isNull);
  });
  test('lost import response survives process recreation with identical frozen key and body', () async {
    final f = ContentFixture();
    await f.start();
    final bodies = <String>[];
    f.respond = (request) async {
      if (request.url.path.endsWith('/imports')) {
        bodies.add(request.body);
        if (bodies.length == 1) throw const ApiFailure('request_failed');
        return http.Response(jsonEncode({'id': 'recipe-00000042'}), 201);
      }
      return null;
    };
    final api = HouseholdApi(f.controller),
        session = ImportSession(HouseholdApi(f.controller), preview(), 0);
    final recipe = object(recipeRecord()['recipe']);
    await expectLater(session.submit(recipe), throwsA(isA<ApiFailure>()));
    final recovered = ImportSession.restore(api)!;
    expect(recovered.frozen, isTrue);
    expect(
      await recovered.submit({
        ...recipe,
        'title': 'Must not change the dispatched request',
      }),
      'recipe-00000042',
    );
    expect(bodies[1], bodies[0]);
    expect(f.controller.draft(ImportSession.key), isNull);
    f.controller.dispose();
  });
  test('current role and household guard retained imports and unsupported draft formats', () async {
    final f = ContentFixture();
    await f.start();
    final api = HouseholdApi(f.controller),
        session = ImportSession(HouseholdApi(f.controller), preview(), 0);
    await session.retain(session.seed());
    f.controller.selected = 'home-other';
    await expectLater(
      session.submit(object(recipeRecord()['recipe'])),
      throwsA(isA<ApiFailure>()),
    );
    expect(api.current, isFalse);
    f.controller.selected = 'home-1';
    await f.controller.saveDraft(ImportSession.key, {'version': 99});
    expect(
      () => ImportSession.restore(HouseholdApi(f.controller)),
      throwsA(isA<ApiFailure>()),
    );
    f.controller.dispose();
  });
  testWidgets(
    'import review shows unresolved source lines and cannot save an incomplete mapping',
    (tester) async {
      final f = ContentFixture();
      await f.start();
      final session = ImportSession(HouseholdApi(f.controller), preview(), 0);
      await tester.pumpWidget(
        app(RecipeEditor(HouseholdApi(f.controller), importSession: session)),
      );
      await tester.pumpAndSettle();
      expect(find.text('1 cup oats'), findsWidgets);
      expect(find.text('Choose matching ingredient'), findsWidgets);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'label review at narrow width and 200 percent scale keeps original unknown basis visible',
    (tester) async {
      final f = ContentFixture();
      await f.start(language: 'de');
      tester.view.physicalSize = const Size(320, 1000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(
        app(
          MediaQuery(
            data: const MediaQueryData(textScaler: TextScaler.linear(2)),
            child: LabelCapture(HouseholdApi(f.controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Declared label values'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'readers can inspect label declarations without an editable or save control',
    (tester) async {
      final f = ContentFixture();
      await f.start(role: 'reader');
      final value = {
        'basis': 'per 100 ml',
        'photo': {
          'image_id': 'image-00000001.png',
          'text': 'Synthetic salt label',
          'engine': 'manual_transcription',
          'score': null,
        },
        'values': [
          {
            'name': 'Salt',
            'value': '0.250001',
            'unit': 'g',
            'qualifier': 'less_than',
            'definition': null,
          },
        ],
        'reviewed': true,
      };
      await tester.pumpWidget(
        app(LabelCapture(HouseholdApi(f.controller), value: value)),
      );
      await tester.pumpAndSettle();
      expect(find.text('per 100 ml'), findsOneWidget);
      expect(find.textContaining('0.250001'), findsOneWidget);
      expect(find.byType(TextFormField), findsNothing);
      expect(find.text('Use reviewed label'), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'OCR classification preserves original cup text and requires explicit line assignments',
    (tester) async {
      await tester.pumpWidget(
        app(const CaptureClassify('Synthetic title\n1 cup oats\nMix gently')),
      );
      await tester.pumpAndSettle();
      expect(find.text('Original only'), findsNWidgets(3));
      await tester.tap(find.byType(PopupMenuButton<String>).at(1));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ingredients').last);
      await tester.pumpAndSettle();
      expect(find.text('1 cup oats'), findsOneWidget);
      expect(find.text('Ingredients'), findsOneWidget);
      expect(find.text('Original only'), findsNWidgets(2));
      await tester.pumpWidget(const SizedBox());
    },
  );
}
