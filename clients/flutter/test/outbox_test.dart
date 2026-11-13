import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';

import 'fixtures.dart';

void main() {
  test(
    'lost response survives process restart with the exact same intent',
    () async {
      final vault = MemoryVault();
      Json? committed;
      var calls = 0;
      http.Client client() => MockClient((request) async {
        if (request.url.path.endsWith('/api/households')) {
          return http.Response(jsonEncode(membership()), 200);
        }
        if (request.method == 'GET') {
          return http.Response(
            jsonEncode(capability(committed == null ? 0 : 1)),
            200,
          );
        }
        final input = object(jsonDecode(request.body));
        calls++;
        if (committed == null) {
          committed = input;
          throw const ApiFailure('connection_failed');
        }
        expect(input, committed);
        return http.Response(
          jsonEncode({
            'version': 1,
            'operation_id': input['operation_id'],
            'applied_revision': 1,
            'replayed': true,
            'shopping': shopping(1),
          }),
          200,
        );
      });
      var c = ShoppingController(Nextcloud(client: client()), vault);
      await c.initialize();
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      await c.enqueue(
        {
          'operation': 'add_manual',
          'label': 'Synthetic oats',
          'ingredient_id': null,
          'quantity': {'amount': '200.000001', 'unit': 'g'},
        },
        original: '200,000001',
        offline: true,
      );
      await c.refresh();
      expect(c.pending.single.uncertain, isTrue);
      expect(calls, 1);
      c.dispose();
      c = ShoppingController(Nextcloud(client: client()), vault);
      await c.initialize();
      expect(calls, 2);
      expect(c.pending, isEmpty);
      expect(c.shopping!.revision, 1);
      c.dispose();
    },
  );
  test(
    'stale checks stop for review and never silently rebase onto new demand',
    () async {
      var revision = 0;
      final vault = MemoryVault();
      var writes = 0;
      final c = ShoppingController(
        Nextcloud(
          client: MockClient((request) async {
            if (request.url.path.endsWith('/api/households')) {
              return http.Response(jsonEncode(membership()), 200);
            }
            if (request.method == 'GET') {
              return http.Response(jsonEncode(capability(revision)), 200);
            }
            writes++;
            expect(object(jsonDecode(request.body))['expected_revision'], 0);
            return http.Response('{"code":"shopping_revision_conflict"}', 409);
          }),
        ),
        vault,
      );
      await c.initialize();
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      revision = 1;
      await c.enqueue({
        'operation': 'check',
        'key': 'ingredient:oats:g',
        'checked': true,
      }, offline: true);
      await c.refresh();
      expect(c.pending.single.error, 'shopping_revision_conflict');
      await c.refresh();
      expect(writes, 1);
      expect(c.pending.single.expected, 0);
      c.dispose();
    },
  );
  test('failed protected persistence prevents all network writes', () async {
    final vault = MemoryVault();
    var writes = 0;
    final c = ShoppingController(
      Nextcloud(
        client: MockClient((request) async {
          if (request.url.path.endsWith('/api/households')) {
            return http.Response(jsonEncode(membership()), 200);
          }
          if (request.method == 'POST') {
            writes++;
            return http.Response('{}', 500);
          }
          return http.Response(jsonEncode(capability(0)), 200);
        }),
      ),
      vault,
    );
    await c.initialize();
    await c.signIn(
      Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
    );
    vault.fail = true;
    await expectLater(
      c.enqueue({
        'operation': 'check',
        'key': 'ingredient:oats:g',
        'checked': true,
      }, offline: true),
      throwsA(isA<ApiFailure>()),
    );
    expect(writes, 0);
    expect(c.storageBlocked, isTrue);
    c.dispose();
  });

  test(
    '5xx and malformed success responses keep the original unknown intent',
    () async {
      for (final status in [503, 200]) {
        final vault = MemoryVault();
        Json? sent;
        var calls = 0;
        final c = ShoppingController(
          Nextcloud(
            client: MockClient((request) async {
              if (request.url.path.endsWith('/api/households')) {
                return http.Response(jsonEncode(membership()), 200);
              }
              if (request.method == 'GET') {
                return http.Response(
                  jsonEncode(capability(calls == 0 ? 0 : 1)),
                  200,
                );
              }
              calls++;
              final input = object(jsonDecode(request.body));
              if (sent == null) {
                sent = input;
                return http.Response(
                  status == 503
                      ? '{"code":"shopping_storage_failed"}'
                      : 'invalid JSON',
                  status,
                );
              }
              expect(input, sent);
              return http.Response(
                jsonEncode({
                  'version': 1,
                  'operation_id': input['operation_id'],
                  'applied_revision': 1,
                  'replayed': true,
                  'shopping': shopping(1),
                }),
                200,
              );
            }),
          ),
          vault,
        );
        await c.initialize();
        await c.signIn(
          Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
        );
        await c.enqueue({
          'operation': 'add_manual',
          'label': 'Synthetic',
          'ingredient_id': null,
          'quantity': {'amount': '1', 'unit': 'piece'},
        }, offline: true);
        await c.refresh();
        expect(c.pending.single.uncertain, isTrue);
        await expectLater(
          c.discard(c.pending.single.id),
          throwsA(isA<ApiFailure>()),
        );
        await c.refresh();
        expect(c.pending, isEmpty);
        expect(calls, 2);
        c.dispose();
      }
    },
  );
  test(
    'known revocation clears protected household cache and stops replay',
    () async {
      var revoked = false;
      var writes = 0;
      final vault = MemoryVault();
      final c = ShoppingController(
        Nextcloud(
          client: MockClient((request) async {
            if (request.url.path.endsWith('/api/households')) {
              return http.Response(jsonEncode(membership()), 200);
            }
            if (revoked) {
              return http.Response('{"code":"household_access_denied"}', 403);
            }
            if (request.method == 'POST') {
              writes++;
              throw const ApiFailure('connection_failed');
            }
            return http.Response(jsonEncode(capability(0)), 200);
          }),
        ),
        vault,
      );
      await c.initialize();
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      await c.enqueue({
        'operation': 'check',
        'key': 'ingredient:oats:g',
        'checked': true,
      }, offline: true);
      await c.refresh();
      expect(c.pending.single.uncertain, isTrue);
      revoked = true;
      await c.refresh();
      expect(c.shopping, isNull);
      expect(c.pending, isEmpty);
      expect(writes, 1);
      expect(object(vault.value!['records']).containsKey('home-1'), isFalse);
      c.dispose();
    },
  );
  test(
    'first login with several households requires an explicit selection',
    () async {
      var loaded = 0;
      final c = ShoppingController(
        Nextcloud(
          client: MockClient((request) async {
            if (request.url.path.endsWith('/api/households')) {
              final value = membership();
              (value['households'] as List).add({
                'id': 'home-2',
                'name': 'Another synthetic home',
                'role': 'reader',
              });
              return http.Response(jsonEncode(value), 200);
            }
            loaded++;
            return http.Response(jsonEncode(capability(0)), 200);
          }),
        ),
        MemoryVault(),
      );
      await c.initialize();
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      expect(c.selected, isNull);
      expect(loaded, 0);
      await c.select('home-1');
      expect(loaded, 1);
      c.dispose();
    },
  );

  test('an unsent offline addition can be bought locally without changing an attempted request', () async {
    var offline = false;
    Json? received;
    final c = ShoppingController(
      Nextcloud(
        client: MockClient((request) async {
          if (offline) throw const ApiFailure('connection_failed');
          if (request.url.path.endsWith('/api/households')) {
            return http.Response(jsonEncode(membership()), 200);
          }
          if (request.method == 'GET') {
            return http.Response(jsonEncode(capability(0)), 200);
          }
          received = object(jsonDecode(request.body));
          return http.Response(
            jsonEncode({
              'version': 1,
              'operation_id': received!['operation_id'],
              'applied_revision': 1,
              'replayed': false,
              'shopping': shopping(1),
            }),
            200,
          );
        }),
      ),
      MemoryVault(),
    );
    await c.initialize();
    await c.signIn(
      Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
    );
    offline = true;
    await c.enqueue({
      'operation': 'add_manual',
      'label': 'Synthetic oats',
      'ingredient_id': 'oats',
      'quantity': {'amount': '200.000001', 'unit': 'g'},
    }, offline: true);
    await c.refresh();
    final id = c.pending.single.id;
    expect(c.pending.single.attempted, isFalse);
    await c.markPending(id, true);
    expect(c.pending.single.command['purchased'], isTrue);
    offline = false;
    await c.refresh();
    expect(object(received!['command'])['purchased'], isTrue);
    expect(received!['operation_id'], id);
    expect(c.pending, isEmpty);
    c.dispose();
  });

  test(
    'an explicit observed revision survives later foreground refresh',
    () async {
      final c = ShoppingController(
        Nextcloud(
          client: MockClient((request) async {
            if (request.url.path.endsWith('/api/households')) {
              return http.Response(jsonEncode(membership()), 200);
            }
            if (request.method == 'GET') {
              return http.Response(jsonEncode(capability(2)), 200);
            }
            expect(object(jsonDecode(request.body))['expected_revision'], 1);
            return http.Response('{"code":"shopping_revision_conflict"}', 409);
          }),
        ),
        MemoryVault(),
      );
      await c.initialize();
      await c.signIn(
        Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
      );
      await c.enqueue(
        {'operation': 'check', 'key': 'ingredient:oats:g', 'checked': true},
        offline: true,
        expectedRevision: 1,
      );
      await c.refresh();
      expect(c.pending.single.expected, 1);
      expect(c.pending.single.error, 'shopping_revision_conflict');
      c.dispose();
    },
  );
}
