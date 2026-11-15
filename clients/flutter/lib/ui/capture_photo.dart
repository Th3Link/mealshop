import 'dart:typed_data';

import 'package:image_picker/image_picker.dart';
import 'package:material_ui/material_ui.dart';

import '../capture/device.dart';
import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';

/// A local still image. Upload happens only after a second, explicit confirmation.
class CapturePhoto extends StatefulWidget {
  const CapturePhoto(
    this.api, {
    super.key,
    this.barcode = false,
    this.device = const CaptureDevice(),
  });
  final HouseholdApi api;
  final bool barcode;
  final CaptureDevice device;
  @override
  State<CapturePhoto> createState() => _CapturePhotoState();
}

class _CapturePhotoState extends State<CapturePhoto> {
  Uint8List? bytes;
  Json? result;
  int rotation = 0;
  bool busy = false;
  String? failure;
  final manual = TextEditingController();
  @override
  void dispose() {
    manual.dispose();
    bytes?.fillRange(0, bytes!.length, 0);
    super.dispose();
  }

  Future<void> choose(ImageSource source) async {
    setState(() => busy = true);
    XFile? picked;
    try {
      picked = await ImagePicker().pickImage(
        source: source,
        requestFullMetadata: false,
        maxWidth: 2500,
        maxHeight: 2500,
        imageQuality: 95,
      );
      if (picked == null || !mounted || !widget.api.current) return;
      if (await picked.length() > 5242880) {
        throw const ApiFailure('image_invalid');
      }
      final value = await picked.readAsBytes();
      if (!mounted || !widget.api.current) return;
      setState(() {
        bytes = value;
        rotation = 0;
        result = null;
        failure = null;
        manual.clear();
      });
      await recognize();
    } catch (e) {
      if (mounted) setState(() => failure = 'capture_failed');
    } finally {
      if (picked != null) await widget.device.discardPicked(picked.path);
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> recognize() async {
    if (bytes == null) return;
    setState(() {
      busy = true;
      failure = null;
    });
    try {
      final value = await widget.device.recognize(
        bytes!,
        rotation: rotation,
        barcode: widget.barcode,
      );
      if (mounted && widget.api.current) setState(() => result = value);
    } catch (_) {
      if (mounted) {
        setState(() {
          failure = 'capture_failed';
          result = null;
        });
      }
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> accept() async {
    if (!await confirm(
          context,
          'capture_upload',
          body: 'capture_upload_body',
        ) ||
        !mounted) {
      return;
    }
    setState(() => busy = true);
    try {
      final data = bytes!;
      final mime = data.length > 2 && data[0] == 255 && data[1] == 216
          ? 'image/jpeg'
          : data.length > 8 && data[0] == 137 && data[1] == 80
          ? 'image/png'
          : data.length > 12 &&
                String.fromCharCodes(data.sublist(8, 12)) == 'WEBP'
          ? 'image/webp'
          : '';
      if (mime.isEmpty) throw const ApiFailure('image_invalid');
      final id = await widget.api.upload(data, mime);
      if (mounted) {
        Navigator.pop(context, <String, dynamic>{
          'image_id': id,
          if (rotation != 0) 'rotation': rotation,
          'text': result?['text'] ?? manual.text,
          'engine': result?['engine'] ?? 'manual_transcription',
          'score': result?['score'],
        });
      }
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(
        context.t(widget.barcode ? 'capture_barcode' : 'capture_photo'),
      ),
    ),
    body: ListView(
      padding: const EdgeInsets.all(20),
      children: [
        Note(context.t('capture_local')),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            OutlinedButton.icon(
              onPressed: busy ? null : () => choose(ImageSource.camera),
              icon: const Icon(Icons.camera_alt_outlined),
              label: Text(context.t('capture_camera')),
            ),
            OutlinedButton.icon(
              onPressed: busy ? null : () => choose(ImageSource.gallery),
              icon: const Icon(Icons.photo_library_outlined),
              label: Text(context.t('capture_gallery')),
            ),
          ],
        ),
        if (busy) const LinearProgressIndicator(),
        if (failure != null) Note(context.t(failure!)),
        if (bytes != null) ...[
          const SizedBox(height: 16),
          Semantics(
            label: context.t('capture_original'),
            image: true,
            child: RotatedBox(
              quarterTurns: rotation ~/ 90,
              child: Image.memory(bytes!, height: 240, fit: BoxFit.contain),
            ),
          ),
          Wrap(
            spacing: 8,
            children: [
              TextButton.icon(
                onPressed: busy
                    ? null
                    : () async {
                        setState(() => rotation = (rotation + 90) % 360);
                        await recognize();
                      },
                icon: const Icon(Icons.rotate_right),
                label: Text(context.t('capture_rotate')),
              ),
              TextButton(
                onPressed: busy ? null : recognize,
                child: Text(context.t('retry')),
              ),
            ],
          ),
          if (widget.barcode) ...[
            if ((result?['barcodes'] as List? ?? []).isEmpty)
              Note(context.t('capture_no_barcode')),
            for (final code in result?['barcodes'] as List? ?? [])
              ListTile(
                title: Text(code as String),
                trailing: const Icon(Icons.check),
                onTap: busy ? null : () => Navigator.pop(context, code),
              ),
          ] else ...[
            SectionTitle(context.t('capture_original')),
            if (result != null) ...[
              SelectableText(result!['text'] as String),
              if (result!['score'] != null)
                Note('${context.t('capture_score')}: ${result!['score']}/100'),
            ] else
              TextField(
                controller: manual,
                minLines: 4,
                maxLines: 12,
                maxLength: 32768,
                decoration: InputDecoration(
                  labelText: context.t('capture_transcribe'),
                ),
              ),
            Note(context.t('capture_unknown')),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: busy || !widget.api.writable ? null : accept,
              child: Text(context.t('capture_upload')),
            ),
          ],
        ],
      ],
    ),
  );
}
