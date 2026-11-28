import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'person_tour.dart';

/// First-start guide: yourself first, then the people you cook for. Every part is optional.
class StartAssistant extends StatefulWidget {
  const StartAssistant(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<StartAssistant> createState() => _StartAssistantState();
}

class _StartAssistantState extends State<StartAssistant> {
  List<Json>? members;
  String? failure;
  bool welcome = true, closing = false;
  @override
  void initState() {
    super.initState();
    welcome = !widget.api.controller.assistantStarted;
    unawaited(load());
  }

  Future<void> load() async {
    setState(() => failure = null);
    try {
      final profiles = await widget.api.household('/profiles');
      if (mounted) setState(() => members = objects(profiles['members']));
    } catch (e) {
      if (mounted) {
        setState(() => failure = e is ApiFailure ? e.code : 'request_failed');
      }
    }
  }

  Future<void> finish() async {
    if (closing) return;
    closing = true;
    try {
      await widget.api.controller.setAssistant(done: true, started: true);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    }
    if (mounted) Navigator.pop(context);
  }

  Future<void> person({bool main = false, Json? existing}) async {
    final c = widget.api.controller;
    try {
      if (!c.assistantStarted) await c.setAssistant(started: true);
    } catch (e) {
      if (mounted) await showFailure(context, e);
      return;
    }
    if (!mounted) return;
    setState(() => welcome = false);
    await openPage<void>(
      context,
      PersonTour(widget.api, main: main, person: existing),
    );
    if (mounted) await load();
  }

  bool get hasMe {
    final id = widget.api.controller.mainPerson;
    return id != null && (members ?? []).any((m) => m['id'] == id);
  }

  bool get draft => widget.api.controller.draft('person:new') != null;
  @override
  Widget build(BuildContext context) {
    final c = widget.api.controller;
    final text = Theme.of(context).textTheme;
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (popped, _) {
        if (!popped) unawaited(finish());
      },
      child: Scaffold(
        appBar: AppBar(
          automaticallyImplyLeading: false,
          title: Text(context.t('assistant')),
        ),
        body: SafeArea(
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 520),
              child: welcome
                  ? ListView(
                      padding: const EdgeInsets.all(24),
                      children: [
                        Icon(
                          Icons.waving_hand_outlined,
                          size: 48,
                          color: Theme.of(context).colorScheme.primary,
                        ),
                        const SizedBox(height: 24),
                        Text(
                          context.t('assistant_welcome'),
                          style: text.headlineMedium,
                        ),
                        const SizedBox(height: 12),
                        Text(
                          context.t('assistant_welcome_body'),
                          style: text.bodyLarge,
                        ),
                        if (c.isDevice) Note(context.t('assistant_local')),
                        const SizedBox(height: 24),
                        FilledButton.icon(
                          onPressed: widget.api.writable
                              ? () => person(main: true)
                              : null,
                          icon: const Icon(Icons.person_outline),
                          label: Text(
                            context.t(
                              draft ? 'assistant_resume' : 'assistant_begin',
                            ),
                          ),
                        ),
                        const SizedBox(height: 8),
                        TextButton(
                          onPressed: finish,
                          child: Text(context.t('skip')),
                        ),
                      ],
                    )
                  : failure != null
                  ? EmptyContent(
                      icon: Icons.cloud_off_outlined,
                      title: failure!,
                      action: 'retry',
                      onAction: load,
                    )
                  : members == null
                  ? const Center(child: CircularProgressIndicator())
                  : ListView(
                      padding: const EdgeInsets.all(24),
                      children: [
                        Text(
                          context.t('assistant_people'),
                          style: text.headlineSmall,
                        ),
                        const SizedBox(height: 8),
                        Text(context.t('assistant_people_body')),
                        const SizedBox(height: 16),
                        if (members!.isEmpty)
                          Note(context.t('assistant_nobody')),
                        for (final member in members!)
                          Card(
                            margin: const EdgeInsets.symmetric(vertical: 4),
                            child: ListTile(
                              leading: CircleAvatar(
                                child: Text(
                                  (member['name'] as String).characters.first,
                                ),
                              ),
                              title: Text(member['name'] as String),
                              subtitle: Text(
                                [
                                  if (member['id'] == c.mainPerson)
                                    context.t('you'),
                                  context.t(
                                    object(
                                              member['rules'] ??
                                                  <String, dynamic>{},
                                            )['preset']
                                            as String? ??
                                        'unrestricted',
                                  ),
                                ].join(' · '),
                              ),
                              trailing: const Icon(Icons.chevron_right),
                              onTap: widget.api.writable
                                  ? () => person(existing: member)
                                  : null,
                            ),
                          ),
                        const SizedBox(height: 16),
                        if (!hasMe)
                          OutlinedButton.icon(
                            onPressed: widget.api.writable
                                ? () => person(main: true)
                                : null,
                            icon: const Icon(Icons.person_outline),
                            label: Text(context.t('assistant_add_me')),
                          ),
                        OutlinedButton.icon(
                          onPressed: widget.api.writable
                              ? () => person()
                              : null,
                          icon: const Icon(Icons.person_add_outlined),
                          label: Text(context.t('assistant_add_other')),
                        ),
                        const SizedBox(height: 24),
                        FilledButton(
                          onPressed: finish,
                          child: Text(context.t('assistant_finish')),
                        ),
                      ],
                    ),
            ),
          ),
        ),
      ),
    );
  }
}
