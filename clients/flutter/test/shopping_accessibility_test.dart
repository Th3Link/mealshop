import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/main.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';

import 'fixtures.dart';

void main() {
  for (final role in ['owner', 'reader']) {
    testWidgets(
      'shopping $role is usable at 320dp, 200% text and with semantics',
      (tester) async {
        await tester.binding.setSurfaceSize(const Size(320, 740));
        tester.platformDispatcher.textScaleFactorTestValue = 2;
        addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
        var disconnected = false;
        final c = ShoppingController(
          Nextcloud(
            client: MockClient((request) async {
              if (disconnected) throw const ApiFailure('connection_failed');
              if (request.url.path.endsWith('/api/households')) {
                return http.Response(jsonEncode(membership(role: role)), 200);
              }
              final value = capability(0, role: role);
              value['ingredient_groups'] = {
                'oats': {
                  'id': 'grain-group',
                  'name': 'Whole grains',
                  'translations': {'de': 'Vollkorngetreide'},
                },
              };
              final list = object(value['shopping']);
              list['items'] = [
                {
                  'key': 'ingredient:oats:g',
                  'ingredient_id': 'oats',
                  'name':
                      'Wholegrain oats with a long descriptive ingredient name',
                  'translations': {
                    'de': 'Haferkorn mit einer längeren Zutatenbezeichnung',
                  },
                  'unit': 'g',
                  'remaining': '200.000001',
                  'purchased': '0.000000',
                  'shop_id': null,
                  'contributions': <Object?>[],
                },
              ];
              return http.Response(
                jsonEncode(value),
                200,
                headers: {'content-type': 'application/json; charset=utf-8'},
              );
            }),
          ),
          MemoryVault(),
        );
        await c.initialize(deviceLanguage: 'de');
        await c.signIn(
          Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
        );
        expect(c.home, isNotNull, reason: c.error);
        expect(c.shopping, isNotNull, reason: c.error);
        expect(c.storageBlocked, isFalse);
        final semantics = tester.ensureSemantics();
        await tester.pumpWidget(MealshopApp(c));
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        await tester.scrollUntilVisible(
          find.text('200,000001 g'),
          100,
          scrollable: find
              .byWidgetPredicate(
                (w) => w is Scrollable && w.axisDirection == AxisDirection.down,
              )
              .first,
        );
        await tester.pumpAndSettle();
        expect(find.text('200,000001 g'), findsOneWidget);
        expect(find.text('Vollkorngetreide'), findsOneWidget);
        expect(
          find.byType(FloatingActionButton),
          role == 'owner' ? findsOneWidget : findsNothing,
        );
        await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
        await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
        if (role == 'owner') {
          disconnected = true;
          c.online = false;
          await c.enqueue({
            'operation': 'add_manual',
            'label': 'Synthetic offline',
            'ingredient_id': null,
            'quantity': {'amount': '1', 'unit': 'g'},
          }, offline: true);
          await c.refresh();
          await tester.pumpAndSettle();
          await tester.scrollUntilVisible(
            find.byType(Checkbox),
            -100,
            scrollable: find
                .byWidgetPredicate(
                  (w) =>
                      w is Scrollable && w.axisDirection == AxisDirection.down,
                )
                .first,
          );
          await tester.pumpAndSettle();
          expect(
            tester.getSemantics(find.byType(Checkbox)).getSemanticsData().label,
            contains('Synthetic offline'),
          );
          await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
          await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
          await tester.tap(find.byType(Checkbox));
          await tester.pumpAndSettle();
          expect(c.pending.single.command['purchased'], isTrue);
        }
        c.error = 'access_removed';
        c.notifyListeners();
        await tester.pumpAndSettle();
        expect(tester.takeException(), isNull);
        semantics.dispose();
        c.dispose();
        await tester.pumpWidget(const SizedBox());
        await tester.binding.setSurfaceSize(null);
      },
    );
  }
}
