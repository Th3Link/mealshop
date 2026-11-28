import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:http/testing.dart';
import 'package:material_ui/material_ui.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';
import 'package:mealshop/ui/home.dart';
import 'package:mealshop/ui/messages.dart';
import 'package:mealshop/ui/more.dart';

import 'fixtures.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const engine = MethodChannel('org.mealshop/device_engine');
  final homes = <Json>[];
  final created = <String>[];
  var failCreate = false;
  setUp(() {
    homes.clear();
    created.clear();
    failCreate = false;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(engine, (call) async {
          final input = object(
            jsonDecode(
              (call.arguments as Map<Object?, Object?>)['value'] as String,
            ),
          );
          if (input['operation'] != 'request') {
            return jsonEncode({
              'generation': 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
              'storage': 'ready',
              'last_backup': null,
            });
          }
          final path = input['path'] as String;
          Object? body;
          var status = 200;
          if (path == '/api/households' && input['method'] == 'POST') {
            if (failCreate) {
              status = 500;
              body = {'error': 'request_failed'};
            } else {
              final name =
                  object(
                        jsonDecode(
                          utf8.decode(base64Decode(input['body'] as String)),
                        ),
                      )['name']
                      as String;
              created.add(name);
              final home = {
                'id': 'home-${homes.length + 1}',
                'name': name,
                'role': 'owner',
              };
              homes.add(home);
              body = home;
            }
          } else if (path == '/api/households') {
            body = {
              'user': 'device',
              'households': homes,
              'invitations': <Object?>[],
            };
          } else if (path.endsWith('/shopping/mobile')) {
            body = capability(1);
          } else {
            status = 404;
            body = {'error': 'not_found'};
          }
          return jsonEncode({
            'status': status,
            'content_type': 'application/json',
            'body': base64Encode(utf8.encode(jsonEncode(body))),
          });
        });
  });
  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(engine, null);
  });
  Future<ShoppingController> local() async {
    final c = ShoppingController(
      Nextcloud(client: MockClient((_) => throw StateError('Unexpected HTTP'))),
      MemoryVault(),
    );
    await c.initialize();
    await c.useOnDevice();
    return c;
  }

  Widget app(Widget child) => MaterialApp(
    locale: const Locale('en'),
    supportedLocales: const [Locale('en'), Locale('de')],
    localizationsDelegates: GlobalMaterialLocalizations.delegates,
    builder: (context, child) =>
        DeviceMessageScope(enabled: true, child: child!),
    home: Scaffold(body: child),
  );

  test(
    'an empty on-device dataset gets one selected internal household',
    () async {
      final c = await local();
      expect(created, [ShoppingController.deviceHomeName]);
      expect(c.homes, hasLength(1));
      expect(c.selected, 'home-1');
      expect(c.showsHomes, false);
      expect(c.writable, true);
      await c.refresh();
      await c.refresh();
      expect(created, hasLength(1));
      c.dispose();
    },
  );

  test('an existing on-device household is reused, never duplicated', () async {
    homes.add({'id': 'home-9', 'name': 'Named earlier', 'role': 'owner'});
    final c = await local();
    expect(created, isEmpty);
    expect(c.selected, 'home-9');
    expect(c.showsHomes, false);
    c.dispose();
  });

  test('several legacy on-device households stay reachable', () async {
    homes.addAll([
      {'id': 'home-1', 'name': 'First', 'role': 'owner'},
      {'id': 'home-2', 'name': 'Second', 'role': 'owner'},
    ]);
    final c = await local();
    expect(created, isEmpty);
    expect(c.selected, isNull);
    expect(c.showsHomes, true);
    c.dispose();
  });

  testWidgets('on-device More shows no household row or name', (tester) async {
    final c = (await tester.runAsync(local))!;
    addTearDown(c.dispose);
    await tester.pumpWidget(app(MoreScreen(c)));
    await tester.pump();
    expect(find.text('On this device'), findsOneWidget);
    expect(find.text('Your households'), findsNothing);
    expect(find.text(ShoppingController.deviceHomeName), findsNothing);
    expect(find.textContaining('ousehold'), findsNothing);
  });

  testWidgets('legacy datasets keep the switcher in More', (tester) async {
    homes.addAll([
      {'id': 'home-1', 'name': 'First', 'role': 'owner'},
      {'id': 'home-2', 'name': 'Second', 'role': 'owner'},
    ]);
    final c = (await tester.runAsync(local))!;
    addTearDown(c.dispose);
    await tester.pumpWidget(app(MoreScreen(c)));
    await tester.pump();
    expect(find.text('Your households'), findsOneWidget);
  });

  testWidgets('a failed preparation offers retry instead of household setup', (
    tester,
  ) async {
    failCreate = true;
    final c = (await tester.runAsync(local))!;
    addTearDown(c.dispose);
    expect(c.homes, isEmpty);
    await tester.pumpWidget(app(NoHousehold(c)));
    await tester.pump();
    expect(find.text('Local data is not ready'), findsOneWidget);
    expect(find.text('Create household'), findsNothing);
    failCreate = false;
    await tester.runAsync(() async {
      await tester.tap(find.text('Try again'));
      await c.refresh();
    });
    expect(c.selected, 'home-1');
  });
}
