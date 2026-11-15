import 'package:image_picker/image_picker.dart';
import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';

class PhotoField extends StatefulWidget {
  const PhotoField(this.api, this.id, this.changed, {super.key});
  final HouseholdApi api;
  final String? id;
  final ValueChanged<String?> changed;
  @override
  State<PhotoField> createState() => _PhotoFieldState();
}

class _PhotoFieldState extends State<PhotoField> {
  bool busy = false;
  Future<void> choose() async {
    setState(() => busy = true);
    try {
      final picked = await ImagePicker().pickImage(
        source: ImageSource.gallery,
        requestFullMetadata: false,
        maxWidth: 2048,
        maxHeight: 2048,
        imageQuality: 88,
      );
      if (picked == null) return;
      if (await picked.length() > 5242880) {
        throw const ApiFailure('image_invalid');
      }
      final bytes = await picked.readAsBytes();
      final mime = bytes.length > 3 && bytes[0] == 255 && bytes[1] == 216
          ? 'image/jpeg'
          : bytes.length > 8 && bytes[0] == 137 && bytes[1] == 80
          ? 'image/png'
          : bytes.length > 12 &&
                String.fromCharCodes(bytes.sublist(8, 12)) == 'WEBP'
          ? 'image/webp'
          : '';
      if (mime.isEmpty) throw const ApiFailure('image_invalid');
      final id = await widget.api.upload(bytes, mime);
      if (mounted) widget.changed(id);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      if (widget.id != null)
        ClipRRect(
          borderRadius: BorderRadius.circular(12),
          child: PrivateImage(
            widget.api,
            '${widget.api.domain}/household/images/${widget.id}',
            key: ValueKey(widget.id),
          ),
        ),
      Wrap(
        spacing: 8,
        children: [
          OutlinedButton.icon(
            onPressed: busy ? null : choose,
            icon: busy
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.photo_library_outlined),
            label: Text(context.t('image')),
          ),
          if (widget.id != null)
            TextButton(
              onPressed: busy ? null : () => widget.changed(null),
              child: Text(context.t('remove_image')),
            ),
        ],
      ),
    ],
  );
}
