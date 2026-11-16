import 'dart:convert';

import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:http/testing.dart';
import 'package:mealshop/connection/device.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/connection/vault.dart';
import 'package:mealshop/cooking/device.dart';
import 'package:mealshop/main.dart';
import 'package:mealshop/shopping/controller.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'real offline storage, encrypted documents, restore and crash recovery',
    (tester) async {
      final vault = DeviceVault();
      final c = ShoppingController(
        Nextcloud(
          client: MockClient(
            (_) => throw StateError('HTTP used in local mode'),
          ),
        ),
        vault,
      );
      await c.initialize();
      await c.useOnDevice();
      await c.createHome('Synthetic offline household');
      final home = c.selected!;
      final root = '/api/households/$home/domain/api';
      Future<Json> call(String path, {String method = 'GET', Json? body}) =>
          c.api.api(c.credential!, '$root$path', method: method, body: body);
      final shopping = await call(
        '/household/shopping',
        method: 'POST',
        body: {
          'expected_revision': 0,
          'command': {
            'operation': 'add_manual',
            'label': 'Synthetic durable item',
            'quantity': {'amount': '0.100001', 'unit': 'kg'},
          },
        },
      );
      expect((shopping['items'] as List).single['remaining'], '100.001000');
      final ingredient = await call(
        '/household/ingredients',
        method: 'POST',
        body: {
          'expected_revision': 0,
          'ingredient': {
            'name': 'Synthetic local ingredient',
            'language': 'en',
            'form': 'As declared',
            'recorded_on': '2026-11-16',
          },
        },
      );
      final png = base64Decode(
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aPJ0AAAAASUVORK5CYII=',
      );
      final Json image;
      try {
        image = await c.api.upload(
          c.credential!,
          '$root/household/images',
          png,
          'image/png',
        );
      } on ApiFailure catch (error) {
        fail('Synthetic image upload failed: ${error.code} (${error.status})');
      }
      final recipe = await call(
        '/recipes',
        method: 'POST',
        body: {
          'title': 'Synthetic local recipe',
          'language': 'en',
          'servings': 1,
          'preparation': 'as_recorded',
          'image_id': image['id'],
          'lines': [
            {'ingredient_id': ingredient['id'], 'grams': '1.000001'},
          ],
          'steps': <Object?>[],
        },
      );
      final device = c.api.device;
      final reminders = AndroidReminders();
      final now = (await reminders.clock()).wall ~/ 1000;
      final oldGeneration = c.credential!.generation;
      final projection = <Json>[
        {
          'key': 'verification:local:step:g1:deadline',
          'due': now + 3600,
          'expires_at': now + 7200,
          'language': 'en',
        },
      ];
      await reminders.replace(projection, generation: oldGeneration);
      expect((await reminders.status())['enabled'], true);
      const password = 'synthetic portable passphrase';
      final backup = await device.command({
        'operation': 'backup',
        'passphrase': password,
      });
      final exportId = backup['id'] as String;
      // Host actions are restricted to this test's separate AVD/package and synthetic export id.
      // ignore: avoid_print
      print('MEALSHOP_VERIFY_EXPORT $exportId');
      expect(await const BackupDocuments().export(exportId), true);
      await device.command({'operation': 'exported', 'id': exportId});
      expect((await device.status())['last_backup'], isNotNull);
      await c.createHome('Synthetic added after backup');
      // ignore: avoid_print
      print('MEALSHOP_VERIFY_IMPORT');
      final imported = (await const BackupDocuments().select())!;
      await expectLater(
        device.command({
          'operation': 'restore',
          'id': imported,
          'passphrase': 'a wrong synthetic passphrase',
        }),
        throwsA(
          isA<ApiFailure>().having(
            (e) => e.code,
            'code',
            'device_restore_password',
          ),
        ),
      );
      var preview = await device.command({
        'operation': 'restore',
        'id': imported,
        'passphrase': password,
      });
      expect((preview['manifest'] as Map)['households'], 1);
      expect(
        ((await c.api.api(c.credential!, '/api/households'))['households']
                as List)
            .length,
        2,
      );
      await device.command({
        'operation': 'discard_restore',
        'token': preview['token'],
      });
      // ignore: avoid_print
      print('MEALSHOP_VERIFY_DAMAGED');
      final damaged = (await const BackupDocuments().select())!;
      await expectLater(
        device.command({
          'operation': 'restore',
          'id': damaged,
          'passphrase': password,
        }),
        throwsA(
          isA<ApiFailure>().having(
            (e) => e.code,
            'code',
            'device_restore_invalid',
          ),
        ),
      );
      expect(
        ((await c.api.api(c.credential!, '/api/households'))['households']
                as List)
            .length,
        2,
      );
      preview = await device.command({
        'operation': 'restore',
        'id': imported,
        'passphrase': password,
      });
      await device.command({
        'operation': 'commit_restore',
        'token': preview['token'],
      });
      expect((await reminders.status())['enabled'], false);
      await reminders.replace(projection, generation: oldGeneration);
      expect((await reminders.status())['enabled'], false);
      await c.useOnDevice();
      await device.command({'operation': 'discard_import', 'id': imported});
      await device.command({'operation': 'discard_import', 'id': damaged});
      expect(
        ((await c.api.api(c.credential!, '/api/households'))['households']
                as List)
            .length,
        1,
      );
      final savedRecipe = await call('/recipes/${recipe['id']}');
      expect(
        ((savedRecipe['recipe'] as Map)['lines'] as List).single['grams'],
        '1.000001',
      );
      expect(
        await c.api.bytes(
          c.credential!,
          '$root/household/images/${image['id']}',
        ),
        orderedEquals(png),
      );
      final before = (await device.status())['process'] as int;
      // ignore: avoid_print
      print('MEALSHOP_VERIFY_CRASH $before');
      var restarted = false;
      for (var attempt = 0; attempt < 60; attempt++) {
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 500)),
        );
        try {
          if ((await device.status())['process'] != before) {
            restarted = true;
            break;
          }
        } on ApiFailure {
          /* The killed owner can take a moment to release its descriptor. */
        }
      }
      expect(restarted, true);
      expect(
        ((await call('/household/shopping'))['items'] as List)
            .single['remaining'],
        '100.001000',
      );
      await c.refresh();
      await c.select(home);
      await c.setDestination('more');
      await tester.pumpWidget(MealshopApp(c));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Backup and restore'));
      await tester.pumpAndSettle();
      expect(find.text('Last completed backup'), findsOneWidget);
      // ignore: avoid_print
      print('MEALSHOP_VERIFY_SCREEN');
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(seconds: 2)),
      );
      await tester.pumpWidget(const SizedBox.shrink());
      c.dispose();
      // Normal entry-point update must read the canonical core, never this test's Flutter cache.
      await vault.write({
        'version': 1,
        'credential': Credential.device().toJson(),
        'language': 'en',
        'destination': 'shopping',
        'homes': <Object?>[],
        'selected': home,
        'selection_required': false,
        'records': <String, dynamic>{},
        'drafts': <String, dynamic>{},
        'clock_offset': 0,
      });
    },
    timeout: const Timeout(Duration(minutes: 8)),
  );
}
