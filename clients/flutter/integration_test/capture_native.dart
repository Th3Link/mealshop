import 'dart:ui' as ui;
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:mealshop/capture/import.dart';
import 'package:mealshop/content/api.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/connection/vault.dart';
import 'package:mealshop/shopping/controller.dart';

import '../test/capture_test.dart' as fixture;
import '../test/content_fixtures.dart';

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:mealshop/capture/device.dart';

Future<Uint8List> image({bool rotated = false, bool barcode = false}) async {
  final recorder = ui.PictureRecorder();
  // Deliberately synthetic originals: no real label/household content enters CI.
  final actual = ui.Canvas(recorder);
  actual.drawColor(const ui.Color(0xffffffff), ui.BlendMode.src);
  if (barcode) {
    // EAN-13 4006381333931. Parity LGLLGG for leading digit 4.
    const left = [
      '0001101',
      '0011001',
      '0010011',
      '0111101',
      '0100011',
      '0110001',
      '0101111',
      '0111011',
      '0110111',
      '0001011',
    ];
    const right = [
      '1110010',
      '1100110',
      '1101100',
      '1000010',
      '1011100',
      '1001110',
      '1010000',
      '1000100',
      '1001000',
      '1110100',
    ];
    const parity = 'LGLLGG', code = '4006381333931';
    var bars = '101';
    for (var i = 0; i < 6; i++) {
      var bits = left[int.parse(code[i + 1])];
      if (parity[i] == 'G') {
        bits = bits.split('').reversed.map((b) => b == '1' ? '0' : '1').join();
      }
      bars += bits;
    }
    bars += '01010';
    for (var i = 7; i < 13; i++) {
      bars += right[int.parse(code[i])];
    }
    bars += '101';
    for (var i = 0; i < bars.length; i++) {
      if (bars[i] == '1') {
        actual.drawRect(
          ui.Rect.fromLTWH(90 + i * 6, 80, 6, 220),
          ui.Paint()..color = const ui.Color(0xff000000),
        );
      }
    }
  } else {
    final p = ui.ParagraphBuilder(ui.ParagraphStyle(fontSize: 48))
      ..pushStyle(ui.TextStyle(color: const ui.Color(0xff000000)))
      ..addText('SYNTHETIC RECIPE\n1 cup oats\n0.250001 g salt');
    final paragraph = p.build()
      ..layout(const ui.ParagraphConstraints(width: 900));
    actual.drawParagraph(paragraph, const ui.Offset(30, 40));
    paragraph.dispose();
  }
  final picture = recorder.endRecording(),
      source = await picture.toImage(1000, 400);
  picture.dispose();
  ui.Image output = source;
  if (rotated) {
    final r = ui.PictureRecorder(), c = ui.Canvas(r);
    c.translate(400, 0);
    c.rotate(3.141592653589793 / 2);
    c.drawImage(source, ui.Offset.zero, ui.Paint());
    final p = r.endRecording();
    output = await p.toImage(400, 1000);
    p.dispose();
  }
  final bytes = (await output.toByteData(format: ui.ImageByteFormat.png))!
      .buffer
      .asUint8List();
  if (output != source) output.dispose();
  source.dispose();
  return bytes;
}

void main() {
  testWidgets(
    'native protected import envelope restores and retries the identical lost response',
    (tester) async {
      final f = ContentFixture();
      await f.start();
      final bodies = <String>[];
      f.respond = (request) async {
        if (request.url.path.endsWith('/imports')) {
          bodies.add(request.body);
          if (bodies.length == 1) throw const ApiFailure('request_failed');
          return http.Response(jsonEncode({'id': 'recipe-00000042'}), 201);
        }
        return null;
      };
      final session = ImportSession(
        HouseholdApi(f.controller),
        fixture.preview(),
        0,
      );
      await expectLater(
        session.submit(object(recipeRecord()['recipe'])),
        throwsException,
      );
      final vault = DeviceVault(namespace: 'mealshop.phone.integration-test');
      await vault.clear();
      await vault.write(f.vault.value!);
      final restored = ShoppingController(
        f.controller.api,
        DeviceVault(namespace: 'mealshop.phone.integration-test'),
      );
      await restored.initialize();
      final recovered = ImportSession.restore(HouseholdApi(restored))!;
      expect(recovered.frozen, isTrue);
      expect(
        await recovered.submit(object(recipeRecord()['recipe'])),
        'recipe-00000042',
      );
      expect(bodies[1], bodies[0]);
      await restored.signOut();
      expect(await vault.read(), isNull);
      restored.dispose();
      f.controller.dispose();
    },
  );
  testWidgets(
    'bundled native OCR keeps unknown cups and exact decimals without a provider',
    (tester) async {
      final result = await const CaptureDevice().recognize(await image());
      expect((result['text'] as String).toLowerCase(), contains('cup'));
      expect(result['text'], contains('0.250001'));
      expect(result['score'], inInclusiveRange(0, 100));
      final rotated = await const CaptureDevice().recognize(
        await image(rotated: true),
        rotation: 270,
      );
      expect(rotated['text'], contains('0.250001'));
    },
  );
  testWidgets(
    'native still-image GTIN scan and invalid image fallback are real',
    (tester) async {
      final result = await const CaptureDevice().recognize(
        await image(barcode: true),
        barcode: true,
      );
      expect(result['barcodes'], contains('4006381333931'));
      await expectLater(
        const CaptureDevice().recognize(Uint8List.fromList([1, 2, 3])),
        throwsException,
      );
    },
  );
}
