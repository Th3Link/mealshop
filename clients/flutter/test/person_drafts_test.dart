import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';

import 'fixtures.dart';

void main() {
  test('person drafts are scoped, survive restart, expire and are cleared on revoked membership', () async {
    final vault = MemoryVault();
    var revoked = false;
    Nextcloud api() => Nextcloud(
      client: MockClient((r) async {
        if (r.url.path.endsWith('/api/households')) {
          final m = membership();
          m['households'] = revoked
              ? <Json>[]
              : [
                  ...m['households'] as List,
                  {
                    'id': 'home-2',
                    'name': 'Synthetic second household',
                    'role': 'owner',
                  },
                ];
          return http.Response(jsonEncode(m), 200);
        }
        return http.Response(jsonEncode(capability(0)), 200);
      }),
    );
    var c = ShoppingController(api(), vault);
    await c.initialize();
    await c.signIn(
      Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
    );
    await c.select('home-1');
    await c.saveDraft('person:new', {
      'person': {'id': 'person-a', 'name': 'Synthetic A'},
      'profile_revision': 0,
      'step': 1,
    });
    await c.select('home-2');
    expect(c.draft('person:new'), isNull);
    await c.saveDraft('person:new', {
      'person': {'id': 'person-b'},
      'profile_revision': 0,
      'step': 0,
    });
    c.dispose();
    c = ShoppingController(api(), vault);
    await c.initialize();
    expect(c.draft('person:new')!['person']['id'], 'person-b');
    await c.select('home-1');
    expect(c.draft('person:new')!['person']['id'], 'person-a');
    c.dispose();
    final records = object(vault.value!['drafts']);
    object(records['home-1:person:new'])['saved_at'] = 0;
    c = ShoppingController(api(), vault);
    await c.initialize();
    expect(c.draft('person:new'), isNull);
    expect(
      object(vault.value!['drafts']).containsKey('home-2:person:new'),
      isTrue,
    );
    revoked = true;
    await c.refresh();
    expect(object(vault.value!['drafts']), isEmpty);
    c.dispose();
  });
  test('draft capacity errors leave the last durable draft and shopping session usable', () async {
    final c = ShoppingController(
      Nextcloud(
        client: MockClient(
          (r) async => http.Response(
            jsonEncode(
              r.url.path.endsWith('/api/households')
                  ? membership()
                  : capability(0),
            ),
            200,
          ),
        ),
      ),
      MemoryVault(),
    );
    await c.initialize();
    await c.signIn(
      Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
    );
    await c.saveDraft('person:new', {'name': 'Synthetic draft'});
    await expectLater(
      c.saveDraft('person:new', {'name': 'x' * 262145}),
      throwsA(isA<ApiFailure>()),
    );
    expect(c.draft('person:new')!['name'], 'Synthetic draft');
    expect(c.storageBlocked, isFalse);
    c.dispose();
  });
}
