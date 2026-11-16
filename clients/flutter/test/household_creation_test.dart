import 'dart:async';
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';

import 'fixtures.dart';

void main() {
  for (final switchAccount in [false, true]) {
    test(
      switchAccount
          ? 'household creation never crosses an account change while waiting'
          : 'household creation waits for background refresh and saves once',
      () async {
        final entered = Completer<void>(), release = Completer<void>();
        var blocked = false, writes = 0;
        final homes = <Json>[
          {'id': 'home-1', 'name': 'Synthetic first', 'role': 'owner'},
        ];
        final c = ShoppingController(
          Nextcloud(
            client: MockClient((request) async {
              if (request.url.path.endsWith('/api/households')) {
                if (request.method == 'POST') {
                  writes++;
                  homes.add({
                    'id': 'home-2',
                    'name': 'Synthetic second',
                    'role': 'owner',
                  });
                  return http.Response('{"id":"home-2"}', 201);
                }
                if (blocked) {
                  blocked = false;
                  entered.complete();
                  await release.future;
                }
                return http.Response(
                  jsonEncode({...membership(), 'households': homes}),
                  200,
                );
              }
              return http.Response(jsonEncode(capability(0)), 200);
            }),
          ),
          MemoryVault(),
        );
        addTearDown(c.dispose);
        await c.initialize();
        await c.signIn(
          Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
        );
        blocked = true;
        final refresh = c.refresh();
        await entered.future;
        expect(c.busy, true);
        final creation = c.createHome('Synthetic second');
        if (switchAccount) {
          final rejected = expectLater(
            creation,
            throwsA(
              isA<ApiFailure>().having((e) => e.code, 'code', 'access_removed'),
            ),
          );
          await c.signOut();
          release.complete();
          await refresh;
          await rejected;
          expect(writes, 0);
          expect(c.credential, isNull);
        } else {
          expect(writes, 0);
          release.complete();
          await refresh;
          await creation;
          expect(writes, 1);
          expect(c.homes.length, 2);
          expect(c.selected, 'home-2');
        }
      },
    );
  }
}
