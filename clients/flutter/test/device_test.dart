import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';
import 'package:mealshop/ui/backups.dart';
import 'package:mealshop/ui/messages.dart';

import 'fixtures.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const engine = MethodChannel('org.mealshop/device_engine');
  const documents = MethodChannel('org.mealshop/device_backups');
  final commands = <Json>[];
  var completed = false;
  String? generation;
  setUp(() {
    commands.clear();
    completed = false;
    generation = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(engine, (call) async {
          final input = object(
            jsonDecode(
              (call.arguments as Map<Object?, Object?>)['value'] as String,
            ),
          );
          commands.add(input);
          return jsonEncode(switch (input['operation']) {
            'status' => {
              'generation': generation,
              'storage': 'ready',
              'last_backup': completed
                  ? {'created_at': 0, 'households': 0, 'files': 0}
                  : null,
            },
            'backup' => {'id': '00000000000000000000000000000001'},
            'exported' => () {
              completed = true;
              return {'saved': true};
            }(),
            'request' => {
              'status': 200,
              'content_type': 'application/json',
              'body': base64Encode(
                utf8.encode(
                  jsonEncode({
                    'user': 'device',
                    'households': <Object?>[],
                    'invitations': <Object?>[],
                  }),
                ),
              ),
            },
            _ => <String, dynamic>{},
          });
        });
  });
  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(engine, null);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(documents, null);
  });
  ShoppingController controller() => ShoppingController(
    Nextcloud(client: MockClient((_) => throw StateError('Unexpected HTTP'))),
    MemoryVault(),
  );
  Widget app(Widget child, {String language = 'en'}) => MaterialApp(
    locale: Locale(language),
    supportedLocales: const [Locale('en'), Locale('de')],
    localizationsDelegates: GlobalMaterialLocalizations.delegates,
    builder: (context, child) =>
        DeviceMessageScope(enabled: true, child: child!),
    home: child,
  );

  test(
    'device credential has no server hostname, password or HTTP identity',
    () {
      final local = Credential.parse({'kind': 'device'});
      expect(local.isDevice, true);
      expect(local.server.root.host, isEmpty);
      expect(local.server.root.scheme, 'mealshop-device');
      expect(local.toJson(), {'kind': 'device'});
      expect(() => Server('mealshop-device:/'), throwsA(isA<ApiFailure>()));
      expect(
        () => Credential.parse({'kind': 'unknown'}),
        throwsA(isA<ApiFailure>()),
      );
    },
  );

  test(
    'binary local requests preserve exact bytes and never fall back to HTTP',
    () async {
      final api = Nextcloud(
        client: MockClient((_) => throw StateError('Unexpected HTTP')),
      );
      await api.upload(
        Credential.device(),
        '/api/households/home-1/domain/api/household/images',
        Uint8List.fromList([0, 255, 10]),
        'image/png',
      );
      expect(commands.single['method'], 'POST');
      expect(commands.single['content_type'], 'image/png');
      expect(base64Decode(commands.single['body'] as String), [0, 255, 10]);
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(
            engine,
            (_) async =>
                throw PlatformException(code: 'device_recovery_required'),
          );
      await expectLater(
        api.api(Credential.device(), '/api/households'),
        throwsA(
          isA<ApiFailure>().having(
            (e) => e.code,
            'code',
            'device_recovery_required',
          ),
        ),
      );
      api.close();
    },
  );

  test(
    'leaving device mode clears its session, not canonical local records',
    () async {
      final c = controller();
      await c.initialize();
      await c.useOnDevice();
      expect(c.isDevice, true);
      expect(c.online, true);
      expect((await c.vault.read())!['credential'], {
        'kind': 'device',
        'generation': generation,
      });
      await c.signOut();
      expect(await c.vault.read(), isNull);
      expect(commands.where((v) => v['operation'] == 'shutdown'), hasLength(1));
      expect(
        commands.any(
          (v) => v['operation'] == 'request' && v['method'] == 'DELETE',
        ),
        false,
      );
      c.dispose();
    },
  );

  test(
    'a committed generation change discards stale cached drafts before refresh',
    () async {
      final vault = MemoryVault();
      final c = ShoppingController(
        Nextcloud(
          client: MockClient((_) => throw StateError('Unexpected HTTP')),
        ),
        vault,
      );
      await c.initialize();
      await c.useOnDevice();
      c.selected = 'home-1';
      await c.saveDraft('recipe:new', {'title': 'Synthetic obsolete draft'});
      expect(object(vault.value!['drafts']), isNotEmpty);
      c.dispose();
      generation = 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
      final restarted = ShoppingController(
        Nextcloud(
          client: MockClient((_) => throw StateError('Unexpected HTTP')),
        ),
        vault,
      );
      await restarted.initialize();
      expect(restarted.credential!.generation, generation);
      expect(object(vault.value!['drafts']), isEmpty);
      expect(commands.last['generation'], generation);
      restarted.dispose();
    },
  );

  testWidgets(
    'backup passwords validate before native encryption; cancellation is not success',
    (tester) async {
      final c = controller();
      await c.initialize();
      await c.useOnDevice();
      addTearDown(c.dispose);
      await tester.pumpWidget(app(BackupsScreen(c)));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Create encrypted backup'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextFormField).first, 'short');
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      expect(
        find.text('Use 12–256 characters for your passphrase.'),
        findsOneWidget,
      );
      expect(commands.any((v) => v['operation'] == 'backup'), false);
      await tester.enterText(
        find.byType(TextFormField).first,
        'synthetic passphrase',
      );
      await tester.enterText(
        find.byType(TextFormField).last,
        'different passphrase',
      );
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      expect(find.text('Passphrases do not match.'), findsOneWidget);
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(documents, (_) async => null);
      await tester.enterText(
        find.byType(TextFormField).last,
        'synthetic passphrase',
      );
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      expect(commands.any((v) => v['operation'] == 'discard_export'), true);
      expect(completed, false);
      expect(find.text('No completed backup yet.'), findsOneWidget);
    },
  );

  testWidgets(
    'a document write failure remains visible and is never marked successful',
    (tester) async {
      final c = controller();
      await c.initialize();
      await c.useOnDevice();
      addTearDown(c.dispose);
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(
            documents,
            (_) async =>
                throw PlatformException(code: 'device_document_failed'),
          );
      await tester.pumpWidget(app(BackupsScreen(c)));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Create encrypted backup'));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byType(TextFormField).first,
        'synthetic passphrase',
      );
      await tester.enterText(
        find.byType(TextFormField).last,
        'synthetic passphrase',
      );
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      expect(
        find.text(message('device_document_failed', 'en')),
        findsOneWidget,
      );
      expect(completed, false);
    },
  );

  testWidgets(
    'German backup view and password dialog fit 320dp with 200 percent text',
    (tester) async {
      tester.view.physicalSize = const Size(320, 900);
      tester.view.devicePixelRatio = 1;
      tester.platformDispatcher.textScaleFactorTestValue = 2;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
      final c = controller();
      await c.initialize();
      await c.useOnDevice();
      addTearDown(c.dispose);
      await tester.pumpWidget(app(BackupsScreen(c), language: 'de'));
      await tester.pumpAndSettle();
      final button = find.text('Verschlüsseltes Backup erstellen');
      await tester.scrollUntilVisible(
        button,
        200,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(button);
      await tester.pumpAndSettle();
      await tester.tap(button);
      await tester.pumpAndSettle();
      expect(find.text('Backup schützen'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
}
