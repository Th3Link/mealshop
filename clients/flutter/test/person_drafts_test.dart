import 'dart:convert';
import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';

import 'fixtures.dart';

void main() {
  test('logout immediately closes account scope and a new login waits for the durable clear', () async {
    final vault = DelayedClearVault();
    var requests = 0;
    final controller = ShoppingController(
      Nextcloud(
        client: MockClient((request) async {
          requests++;
          return http.Response(
            jsonEncode(
              request.url.path.endsWith('/api/households')
                  ? membership()
                  : capability(0),
            ),
            200,
          );
        }),
      ),
      vault,
    );
    await controller.initialize();
    await controller.signIn(
      Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
    );
    await controller.saveDraft('person:new', {'name': 'Synthetic draft'});
    vault.barrier = Completer<void>();
    final logout = controller.signOut();
    expect(controller.credential, isNull);
    expect(controller.selected, isNull);
    expect(controller.initialized, isFalse);
    expect(controller.draft('person:new'), isNull);
    final before = requests;
    await controller.refresh();
    expect(requests, before);
    final login = controller.signIn(
      Credential(
        Server('https://cloud.example.org'),
        'alice',
        'synthetic-next',
      ),
    );
    expect(controller.credential, isNull);
    vault.barrier!.complete();
    await logout;
    await login;
    expect(controller.initialized, isTrue);
    expect(controller.credential!.password, 'synthetic-next');
    expect(controller.draft('person:new'), isNull);
    controller.dispose();
  });

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

class DelayedClearVault extends MemoryVault {
  Completer<void>? barrier;
  @override
  Future<void> clear() async {
    if (barrier != null) await barrier!.future;
    await super.clear();
  }
}
