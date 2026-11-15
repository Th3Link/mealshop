import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/ui/history_editor.dart';
import 'package:mealshop/ui/learning.dart';

import 'content_fixtures.dart';
import 'cooking_test.dart' show cookedResponse;

Widget app(Widget child, {double scale = 1}) => MaterialApp(
  locale: const Locale('en'),
  supportedLocales: const [Locale('en'), Locale('de')],
  localizationsDelegates: GlobalMaterialLocalizations.delegates,
  builder: (context, child) => MediaQuery(
    data: MediaQuery.of(context).copyWith(textScaler: TextScaler.linear(scale)),
    child: child!,
  ),
  home: child,
);
Json learning() => {
  'revision': 0,
  'epoch': 1,
  'policy': {'enabled': false, 'member_ids': <String>[]},
  'history_records': 0,
  'model_version': 'rules-v1',
  'learned_cooks': 0,
  'learned_recipes': 0,
};
void main() {
  testWidgets(
    'manual history explicitly reviews a concurrent change without losing attendees or inventing ratings',
    (tester) async {
      final f = ContentFixture();
      Json? saved;
      var attempts = 0;
      f.people['members'] = [
        {'id': 'one', 'name': 'Synthetic attendee'},
      ];
      f.respond = (r) async {
        if (r.url.path.endsWith('/recipes')) {
          return cookedResponse({
            'recipes': [
              {
                'id': 'recipe-1',
                'revision': 1,
                'title': 'Synthetic recipe',
                'servings': 2,
              },
            ],
          });
        }
        if (r.url.path.endsWith('/history')) {
          if (r.method == 'POST') {
            saved = object(jsonDecode(r.body));
            attempts++;
            if (attempts == 1) {
              return cookedResponse({'code': 'history_revision_conflict'}, 409);
            }
            return cookedResponse({'revision': 1, 'id': 'history-1'});
          }
          return cookedResponse({
            'revision': attempts == 0 ? 0 : 1,
            'entries': <Json>[],
            'page': 1,
            'per_page': 20,
            'total': 0,
          });
        }
        return null;
      };
      await f.start();
      await tester.pumpWidget(app(HistoryEditor(HouseholdApi(f.controller))));
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.text('Synthetic attendee'),
        200,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Synthetic attendee'));
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.text('This was actually cooked.'),
        200,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('This was actually cooked.'));
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text('Save'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();
      expect(attempts, 1);
      await tester.scrollUntilVisible(
        find.text('Review retained entry'),
        -200,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Review retained entry'));
      await tester.pumpAndSettle();
      await tester.tap(
        find.widgetWithText(FilledButton, 'Review retained entry').last,
      );
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.text('This was actually cooked.'),
        200,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('This was actually cooked.'));
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text('Save'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();
      expect(attempts, 2);
      expect(saved!['expected_revision'], 1);
      expect(saved, isNotNull);
      expect(object(saved!['input'])['active_seconds'], isNull);
      expect(object(saved!['input'])['household_rating'], isNull);
      expect(object(saved!['input'])['ratings'], isEmpty);
      expect(object(saved!['input'])['attendees'], ['one']);
      expect(saved!['confirmed_completed'], true);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
  testWidgets(
    'readers inspect learning at 320dp and 200 percent without owner switches',
    (tester) async {
      tester.view.physicalSize = const Size(320, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final f = ContentFixture();
      f.respond = (r) async =>
          r.url.path.endsWith('/learning') ? cookedResponse(learning()) : null;
      await f.start(role: 'reader');
      await tester.pumpWidget(
        app(LearningScreen(HouseholdApi(f.controller)), scale: 2),
      );
      await tester.pumpAndSettle();
      expect(find.byType(SwitchListTile), findsNothing);
      expect(f.requests.where((r) => r.method != 'GET'), isEmpty);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      f.controller.dispose();
    },
  );
}
