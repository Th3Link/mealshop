import 'dart:async';

import 'package:material_ui/material_ui.dart';
import 'package:url_launcher/url_launcher.dart';

import '../connection/nextcloud.dart';
import '../shopping/controller.dart';
import 'messages.dart';

class ConnectScreen extends StatefulWidget {
  const ConnectScreen(this.controller, {super.key});
  final ShoppingController controller;
  @override
  State<ConnectScreen> createState() => _ConnectScreenState();
}

class _ConnectScreenState extends State<ConnectScreen> {
  final address = TextEditingController();
  LoginFlow? flow;
  String? error;
  bool busy = false;
  int generation = 0;
  @override
  void dispose() {
    generation++;
    address.dispose();
    super.dispose();
  }

  Future<void> openBrowser() async {
    if (flow != null &&
        !await launchUrl(flow!.login, mode: LaunchMode.externalApplication)) {
      throw const ApiFailure('request_failed');
    }
  }

  Future<void> start() async {
    final attempt = ++generation;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final next = await widget.controller.api.start(Server(address.text));
      if (!mounted || attempt != generation) return;
      setState(() => flow = next);
      await openBrowser();
      final deadline = DateTime.now().add(const Duration(minutes: 10));
      while (mounted &&
          attempt == generation &&
          DateTime.now().isBefore(deadline)) {
        await Future<void>.delayed(const Duration(seconds: 2));
        if (!mounted || attempt != generation) return;
        final credential = await widget.controller.api.poll(next);
        if (credential != null) {
          if (!mounted || attempt != generation) return;
          await widget.controller.signIn(credential);
          return;
        }
      }
      if (mounted && attempt == generation) {
        setState(() => error = 'login_timeout');
      }
    } on ApiFailure catch (e) {
      if (mounted && attempt == generation) setState(() => error = e.code);
    } catch (_) {
      if (mounted && attempt == generation) {
        setState(() => error = 'request_failed');
      }
    } finally {
      if (mounted && attempt == generation) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: const Text('Mealshop'),
      actions: [
        PopupMenuButton<String>(
          tooltip: context.t('language'),
          onSelected: widget.controller.setLanguage,
          itemBuilder: (_) => const [
            PopupMenuItem(value: 'de', child: Text('Deutsch')),
            PopupMenuItem(value: 'en', child: Text('English')),
          ],
          icon: const Icon(Icons.language_outlined),
        ),
      ],
    ),
    body: SafeArea(
      child: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 480),
          child: ListView(
            shrinkWrap: true,
            padding: const EdgeInsets.all(24),
            children: [
              Align(
                alignment: Alignment.centerLeft,
                child: Icon(
                  Icons.shopping_basket_outlined,
                  size: 48,
                  color: Theme.of(context).colorScheme.primary,
                ),
              ),
              const SizedBox(height: 32),
              Text(
                context.t('connect'),
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              const SizedBox(height: 12),
              Text(
                context.t('connect_intro'),
                style: Theme.of(context).textTheme.bodyLarge,
              ),
              const SizedBox(height: 32),
              TextField(
                controller: address,
                enabled: !busy,
                keyboardType: TextInputType.url,
                autocorrect: false,
                onSubmitted: (_) {
                  if (!busy) unawaited(start());
                },
                decoration: InputDecoration(
                  labelText: context.t('server'),
                  hintText: context.t('server_hint'),
                ),
              ),
              const SizedBox(height: 20),
              if (error != null)
                Padding(
                  padding: const EdgeInsets.only(bottom: 16),
                  child: Text(
                    context.t(error!),
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                ),
              if (flow != null && busy) ...[
                const LinearProgressIndicator(),
                const SizedBox(height: 16),
                Text(context.t('waiting')),
                TextButton(
                  onPressed: () async {
                    try {
                      await openBrowser();
                    } catch (_) {
                      setState(() => error = 'request_failed');
                    }
                  },
                  child: Text(context.t('open_browser')),
                ),
              ] else
                FilledButton.icon(
                  onPressed: busy ? null : start,
                  icon: busy
                      ? const SizedBox.square(
                          dimension: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.open_in_browser),
                  label: Text(context.t('browser')),
                ),
              if (busy)
                TextButton(
                  onPressed: () {
                    generation++;
                    setState(() {
                      busy = false;
                      flow = null;
                    });
                  },
                  child: Text(context.t('cancel')),
                ),
            ],
          ),
        ),
      ),
    ),
  );
}
