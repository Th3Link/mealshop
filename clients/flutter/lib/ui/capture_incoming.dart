import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../capture/device.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'recipe_import.dart';

class IncomingCapture extends StatefulWidget {
  const IncomingCapture(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<IncomingCapture> createState() => _IncomingCaptureState();
}

class _IncomingCaptureState extends State<IncomingCapture>
    with WidgetsBindingObserver {
  static const device = CaptureDevice();
  String? text, error;
  Timer? timer;
  bool busy = false;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    unawaited(peek());
    timer = Timer.periodic(const Duration(seconds: 5), (_) => peek());
  }

  @override
  void dispose() {
    timer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) unawaited(peek());
  }

  Future<void> peek() async {
    if (busy) return;
    busy = true;
    try {
      final value = await device.incoming();
      if (mounted && widget.api.current && value != text) {
        setState(() => text = value);
      }
    } catch (_) {
      if (mounted) setState(() => error = 'storage_failed');
    } finally {
      busy = false;
    }
  }

  @override
  Widget build(BuildContext context) => text == null && error == null
      ? const SizedBox.shrink()
      : ListTile(
          leading: const Icon(Icons.move_to_inbox_outlined),
          title: Text(context.t(error ?? 'capture_incoming')),
          subtitle: Text(context.t('capture_destination')),
          trailing: IconButton(
            tooltip: context.t('delete'),
            icon: const Icon(Icons.close),
            onPressed: () async {
              await device.clearIncoming();
              if (mounted) setState(() => text = null);
            },
          ),
          onTap: !widget.api.writable || text == null
              ? null
              : () async {
                  try {
                    await openPage<String>(
                      context,
                      RecipeImport(widget.api, sharedText: text),
                    );
                    // Explicitly retain the native incoming item until the customer dismisses it.
                  } catch (e) {
                    if (context.mounted) await showFailure(context, e);
                  }
                },
        );
}
