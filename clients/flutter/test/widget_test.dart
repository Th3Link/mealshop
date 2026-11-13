import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mealshop/main.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';

import 'fixtures.dart';

void main() {
  testWidgets(
    'revoked household closes drafts and requires a new explicit choice',
    (tester) async {
      var revoked = false;
      final vault = MemoryVault();
      final requests = <String>[];
      Nextcloud api() => Nextcloud(
        client: MockClient((request) async {
          requests.add(request.url.path);
          if (request.url.path.endsWith('/api/households')) {
            final value = membership();
            if (revoked) {
              value['households'] = [
                {
                  'id': 'home-2',
                  'name': 'Synthetic second home',
                  'role': 'owner',
                },
              ];
            }
            return http.Response(jsonEncode(value), 200);
          }
          return http.Response(jsonEncode(capability(0)), 200);
        }),
      );
      var c = ShoppingController(api(), vault);
      await c.initialize();
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      await tester.pumpWidget(MealshopApp(c));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Add item'));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byType(EditableText).first,
        'Synthetic home A draft',
      );
      revoked = true;
      await c.refresh();
      await tester.pumpAndSettle();
      expect(c.selected, isNull);
      expect(c.shopping, isNull);
      expect(c.error, 'access_removed');
      expect(find.text('Synthetic home A draft'), findsNothing);
      expect(requests.any((p) => p.contains('/home-2/domain/')), isFalse);
      expect(object(vault.value!['records']), isEmpty);
      c.dispose();
      await tester.pumpWidget(const SizedBox());
      c = ShoppingController(api(), vault);
      await c.initialize();
      expect(
        c.selected,
        isNull,
        reason: 'Revocation must not auto-select another home after restart',
      );
      await c.select('home-2');
      expect(c.selected, 'home-2');
      expect(c.shopping, isNotNull);
      c.dispose();
    },
  );
  for (final language in ['en', 'de']) {
    testWidgets('connection works at 320dp and 200% text in $language', (
      tester,
    ) async {
      await tester.binding.setSurfaceSize(const Size(320, 700));
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      final c = ShoppingController(Nextcloud(), MemoryVault());
      await c.initialize(deviceLanguage: language);
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(textScaler: TextScaler.linear(2)),
          child: MealshopApp(c),
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      await tester.scrollUntilVisible(find.byType(TextField), 200);
      await tester.pumpAndSettle();
      expect(find.byType(EditableText), findsOneWidget);
      await tester.enterText(
        find.byType(EditableText),
        'https://cloud.example.org',
      );
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        'https://cloud.example.org',
      );
      c.dispose();
      await tester.pumpWidget(const SizedBox());
      await tester.binding.setSurfaceSize(null);
    });
  }
}
