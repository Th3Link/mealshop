import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/main.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/connection/vault.dart';
import 'package:mealshop/shopping/controller.dart';

import '../test/fixtures.dart';
import '../test/people_library_test.dart' as content_tests;
import '../test/planning_test.dart' as planning_tests;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  content_tests.main();
  planning_tests.main();
  testWidgets(
    'Android protected storage survives a new instance and clears only its namespace',
    (tester) async {
      final vault = DeviceVault(namespace: 'mealshop.phone.integration-test');
      await vault.clear();
      await vault.write({
        'version': 1,
        'value': 'synthetic-protected-shopping',
      });
      expect(
        (await DeviceVault(namespace: 'mealshop.phone.integration-test')
            .read())!['value'],
        'synthetic-protected-shopping',
      );
      await vault.clear();
      expect(await vault.read(), isNull);
    },
  );
  testWidgets(
    'native shopping navigation, exact German quantity and pending confirmation',
    (tester) async {
      var revision = 0;
      final c = ShoppingController(
        Nextcloud(
          client: MockClient((request) async {
            if (request.url.path.endsWith('/api/households')) {
              return http.Response(jsonEncode(membership()), 200);
            }
            if (request.method == 'GET') {
              return http.Response(jsonEncode(capability(revision)), 200);
            }
            final input = object(jsonDecode(request.body));
            expect(
              object(object(input['command'])['quantity'])['amount'],
              '200.000001',
            );
            revision++;
            return http.Response(
              jsonEncode({
                'version': 1,
                'operation_id': input['operation_id'],
                'applied_revision': revision,
                'replayed': false,
                'shopping': shopping(revision),
              }),
              200,
            );
          }),
        ),
        MemoryVault(),
      );
      await c.initialize(deviceLanguage: 'de');
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      await tester.pumpWidget(MealshopApp(c));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Hinzufügen'));
      await tester.pumpAndSettle();
      final fields = find.byType(EditableText);
      await tester.enterText(fields.at(0), 'Synthetic oats');
      await tester.enterText(fields.at(1), '200,000001');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.ensureVisible(find.text('Speichern'));
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();
      await c.refresh();
      await tester.pumpAndSettle();
      expect(c.pending, isEmpty);
      expect(revision, 1);
      expect(tester.takeException(), isNull);
      await tester.tap(find.text('Mehr'));
      await tester.pumpAndSettle();
      expect(find.text('Geschäfte & Reihenfolge'), findsOneWidget);
      await tester.tap(
        find.descendant(
          of: find.byType(NavigationBar),
          matching: find.text('Einkaufen'),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Alles beginnt mit einer Zutat'), findsOneWidget);
      c.dispose();
      await tester.pumpWidget(const SizedBox());
    },
  );
}
