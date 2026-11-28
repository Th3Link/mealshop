import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/shopping/controller.dart';
import 'package:mealshop/ui/person_tour.dart';
import 'package:mealshop/ui/start_assistant.dart';

import 'content_fixtures.dart';

Widget app(Widget screen, {String language = 'en', double scale = 1}) =>
    MaterialApp(
      locale: Locale(language),
      supportedLocales: const [Locale('en'), Locale('de')],
      localizationsDelegates: GlobalMaterialLocalizations.delegates,
      builder: (context, child) => MediaQuery(
        data: MediaQuery.of(context)
            .copyWith(textScaler: TextScaler.linear(scale)),
        child: child!,
      ),
      home: screen,
    );
void main() {
  Future<void> next(WidgetTester tester) async {
    await tester.tap(find.text('Next'));
    await tester.pumpAndSettle();
  }

  testWidgets(
    'assistant adds the main user and another person with names only',
    (tester) async {
      final f = ContentFixture();
      await f.start();
      final c = f.controller;
      await tester.pumpWidget(app(StartAssistant(HouseholdApi(c))));
      await tester.pumpAndSettle();
      expect(find.text('Welcome to Mealshop'), findsOneWidget);
      await tester.tap(find.text('Start with me'));
      await tester.pumpAndSettle();
      expect(c.assistantStarted, true);
      expect(find.textContaining('Start with yourself'), findsOneWidget);
      await tester.enterText(
        find.byKey(const ValueKey('person_name')),
        'Synthetic main',
      );
      await tester.tap(find.text('Save with the name only'));
      await tester.pumpAndSettle();
      final me = objects(f.people['members']).single;
      expect(me['name'], 'Synthetic main');
      expect(c.mainPerson, me['id']);
      expect(find.text('Who else eats with you?'), findsOneWidget);
      expect(find.text('You · No dietary preset'), findsOneWidget);
      expect(find.text('Add myself'), findsNothing);
      await tester.tap(find.text('Add another person'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Save with the name only'));
      await tester.pumpAndSettle();
      expect(objects(f.people['members']).map((m) => m['name']), [
        'Synthetic main',
        'Person 2',
      ]);
      expect(c.mainPerson, me['id']);
      expect(c.assistantDone, false);
      await tester.tap(find.text('Finish'));
      await tester.pumpAndSettle();
      expect(c.assistantDone, true);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  testWidgets('skipping saves nobody and does not ask again', (tester) async {
    final f = ContentFixture();
    await f.start();
    await tester.pumpWidget(app(StartAssistant(HouseholdApi(f.controller))));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Skip for now'));
    await tester.pumpAndSettle();
    expect(objects(f.people['members']), isEmpty);
    expect(f.controller.assistantDone, true);
    expect(f.controller.mainPerson, isNull);
    await tester.pumpWidget(const SizedBox());
    f.controller.dispose();
  });

  testWidgets('an interrupted assistant resumes at the people overview', (
    tester,
  ) async {
    final f = ContentFixture();
    await f.start();
    await f.controller.setAssistant(started: true);
    final restarted = ShoppingController(f.controller.api, f.vault);
    await restarted.initialize();
    expect(restarted.assistantStarted, true);
    expect(restarted.assistantDone, false);
    await tester.pumpWidget(app(StartAssistant(HouseholdApi(restarted))));
    await tester.pumpAndSettle();
    expect(find.text('Welcome to Mealshop'), findsNothing);
    expect(find.text('Nobody has been added yet.'), findsOneWidget);
    expect(find.text('Add myself'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    await restarted.signOut();
    expect(restarted.assistantStarted, false);
    restarted.dispose();
    f.controller.dispose();
  });

  testWidgets(
    'diet cards, soft dislikes and the device-only marker reach the summary',
    (tester) async {
      final f = ContentFixture();
      await f.start();
      final c = f.controller;
      await tester.pumpWidget(app(PersonTour(HouseholdApi(c), main: true)));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const ValueKey('person_name')),
        'Synthetic vegan',
      );
      await next(tester);
      expect(find.text('2 / 6 · Dietary rules'), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('preset_vegan')));
      await tester.pumpAndSettle();
      await next(tester);
      expect(find.text('3 / 6 · Allergies & intolerances'), findsOneWidget);
      await next(tester);
      expect(find.text('4 / 6 · Dislikes & exclusions'), findsOneWidget);
      expect(find.textContaining('Dislikes only lower'), findsOneWidget);
      await next(tester);
      await next(tester);
      expect(find.text('6 / 6 · Review & save'), findsOneWidget);
      await tester.scrollUntilVisible(find.text('This is me'), 200);
      expect(
        tester.widget<SwitchListTile>(find.byType(SwitchListTile).last).value,
        true,
      );
      await tester.tap(find.text('This is me'));
      await tester.pumpAndSettle();
      await tester.tap(find.widgetWithText(FilledButton, 'Save'));
      await tester.pumpAndSettle();
      final saved = objects(f.people['members']).single;
      expect(object(saved['rules'])['preset'], 'vegan');
      expect(saved.containsKey('me'), false);
      expect(c.mainPerson, isNull);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  for (final language in ['en', 'de']) {
    testWidgets('assistant and diet step fit 320dp at 200% text in $language', (
      tester,
    ) async {
      await tester.binding.setSurfaceSize(const Size(320, 740));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      final f = ContentFixture();
      await f.start(language: language);
      final api = HouseholdApi(f.controller);
      await tester.pumpWidget(
        app(StartAssistant(api), language: language, scale: 2),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await f.controller.setAssistant(started: true);
      await tester.pumpWidget(const SizedBox());
      await tester.pumpWidget(
        app(StartAssistant(api), language: language, scale: 2),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      await tester.pumpWidget(
        app(PersonTour(api), language: language, scale: 2),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await tester.tap(find.text(language == 'de' ? 'Weiter' : 'Next'));
      await tester.pumpAndSettle();
      expect(find.byType(OptionCard), findsWidgets);
      await tester.scrollUntilVisible(
        find.byKey(const ValueKey('preset_vegan')),
        200,
        scrollable: find.byType(Scrollable).first,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    });
  }
}
