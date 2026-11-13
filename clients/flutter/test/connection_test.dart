import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';

void main() {
  test(
    'installation prefix is preserved and login targets cannot change origin',
    () {
      final server = Server('https://cloud.example.org/nextcloud/');
      expect(
        server.path('/index.php/login/v2').toString(),
        'https://cloud.example.org/nextcloud/index.php/login/v2',
      );
      for (final value in [
        'https://evil.example/login',
        'https://cloud.example.org/outside',
        'https://user:pw@cloud.example.org/nextcloud/x',
      ]) {
        expect(() => server.validate(value), throwsA(isA<ApiFailure>()));
      }
      for (final value in [
        'http://cloud.example.org',
        'https://user:pw@cloud.example.org',
        'https://cloud.example.org/a/../b',
        'https://cloud.example.org/?query=1',
      ]) {
        expect(() => Server(value), throwsA(isA<ApiFailure>()));
      }
      expect(
        () => Server('http://127.0.0.1:8080', allowLoopback: false),
        throwsA(isA<ApiFailure>()),
      );
    },
  );
  test('app credentials only enter the intended proxy request and redirects are disabled', () async {
    final credential = Credential(
      Server('https://cloud.example.org/nc'),
      'alice',
      'synthetic-device-password',
    );
    final api = Nextcloud(
      client: MockClient((request) async {
        expect(request.followRedirects, isFalse);
        expect(
          request.url.path,
          '/nc/index.php/apps/app_api/proxy/mealshop/api/households',
        );
        expect(
          request.headers['authorization'],
          'Basic ${base64Encode(utf8.encode('alice:synthetic-device-password'))}',
        );
        expect(request.headers['x-mealshop-request'], '1');
        return http.Response('{"households":[]}', 200);
      }),
    );
    await api.api(credential, '/api/households');
    api.close();
  });
  test(
    'Login Flow v2 polling is anonymous and rejects a changed canonical origin',
    () async {
      final api = Nextcloud(
        client: MockClient((request) async {
          expect(request.headers.containsKey('authorization'), isFalse);
          if (request.url.path.endsWith('/poll')) {
            return http.Response(
              jsonEncode({
                'server': 'https://evil.example',
                'loginName': 'alice',
                'appPassword': 'synthetic',
              }),
              200,
            );
          }
          return http.Response(
            jsonEncode({
              'login': 'https://cloud.example.org/nc/login',
              'poll': {
                'endpoint': 'https://cloud.example.org/nc/poll',
                'token': 'ephemeral',
              },
            }),
            200,
          );
        }),
      );
      final flow = await api.start(Server('https://cloud.example.org/nc'));
      await expectLater(api.poll(flow), throwsA(isA<ApiFailure>()));
      api.close();
    },
  );
}
