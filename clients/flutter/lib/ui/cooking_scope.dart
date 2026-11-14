import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import '../cooking/device.dart';
import '../shopping/controller.dart';

class CookingScope extends InheritedNotifier<CookingController> {
  const CookingScope({
    super.key,
    required super.notifier,
    required super.child,
  });
  static CookingController? maybeOf(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<CookingScope>()?.notifier;
}

/// Closing a scope cancels its local projections before another household can activate them.
class CookingHost extends StatefulWidget {
  const CookingHost(this.account, {super.key, required this.child});
  final ShoppingController account;
  final Widget child;
  @override
  State<CookingHost> createState() => _CookingHostState();
}

class _CookingHostState extends State<CookingHost> with WidgetsBindingObserver {
  final device = AndroidReminders();
  Credential? credential;
  String? household;
  CookingController? controller;
  bool wasOnline = false, wasWritable = false, activated = false;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _scope();
  }

  @override
  void didUpdateWidget(CookingHost oldWidget) {
    super.didUpdateWidget(oldWidget);
    _scope();
  }

  void _scope() {
    final account = widget.account;
    if (!identical(credential, account.credential) ||
        household != account.selected ||
        (account.storageBlocked && controller != null)) {
      controller?.dispose();
      credential = account.credential;
      household = account.selected;
      final ready = activated ? device.clear() : Future<void>.value();
      controller =
          credential != null &&
              household != null &&
              account.home != null &&
              !account.storageBlocked
          ? CookingController(
              HouseholdApi(account),
              device: device,
              scopeReady: ready,
            )
          : null;
      activated = activated || controller != null;
      final current = controller;
      if (current != null) unawaited(_initialize(current));
    } else if (controller != null) {
      if (wasWritable && account.home?.writable != true) {
        unawaited(controller!.revoke());
      }
      if (!wasOnline && account.online && controller!.list != null) {
        unawaited(controller!.refresh());
      }
    }
    wasOnline = account.online;
    wasWritable = account.home?.writable == true;
  }

  Future<void> _initialize(CookingController current) async {
    try {
      await current.initialize();
    } catch (error) {
      if (current.current) {
        current.error = error is ApiFailure ? error.code : 'connection_failed';
        current.initialized = true;
        if (mounted) setState(() {});
      }
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) return;
    final current = controller;
    if (current == null) return;
    unawaited(() async {
      await widget.account.refresh();
      if (current.current && current.list != null) await current.refresh();
    }());
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    controller?.dispose();
    unawaited(device.clear());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) =>
      CookingScope(notifier: controller, child: widget.child);
}
