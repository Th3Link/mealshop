import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';
import 'package:mealshop/ui/shops.dart';

import 'fixtures.dart';

void main() {
  for (final state in [
    (false, false, false),
    (true, false, true),
    (true, true, false),
  ]) {
    testWidgets('shop round needs consent and no active session: $state', (
      tester,
    ) async {
      final c = ShoppingController(
        Nextcloud(
          client: MockClient((request) async {
            if (request.url.path.endsWith('/api/households')) {
              return http.Response(jsonEncode(membership()), 200);
            }
            final value = capability(0);
            final shops = object(object(value['shopping'])['shops']);
            shops['entries'] = [
              {'id': 'shop-1', 'name': 'Synthetic shop'},
            ];
            shops['learning'] = state.$1;
            if (state.$2) {
              shops['sessions'] = [
                {'id': 'session-1', 'shop_id': 'shop-1', 'completed': false},
              ];
            }
            return http.Response(jsonEncode(value), 200);
          }),
        ),
        MemoryVault(),
      );
      await c.initialize();
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      await tester.pumpWidget(MaterialApp(home: ShopsScreen(c)));
      await tester.pumpAndSettle();
      await tester.tap(find.byType(PopupMenuButton<String>));
      await tester.pumpAndSettle();
      final action = tester.widget<PopupMenuItem<String>>(
        find.widgetWithText(PopupMenuItem<String>, 'Start round'),
      );
      expect(action.enabled, state.$3);
      final remove = tester.widget<PopupMenuItem<String>>(
        find.widgetWithText(PopupMenuItem<String>, 'Remove'),
      );
      expect(remove.enabled, !state.$2);
      c.dispose();
      await tester.pumpWidget(const SizedBox());
    });
  }
}
