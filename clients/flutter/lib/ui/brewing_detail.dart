import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import 'common.dart';
import 'content_common.dart';
import 'cooking.dart' show CookingCacheScreen;
import 'brewing_common.dart';
import 'brewing_review.dart';
import 'brewing_reading.dart';
import 'messages.dart';
import 'process_common.dart';
import 'process_reminders.dart';

class BrewingDetailScreen extends StatefulWidget {
  const BrewingDetailScreen(this.c, this.id, {super.key});
  final CookingController c;
  final String id;
  @override
  State<BrewingDetailScreen> createState() => _BrewingDetailScreenState();
}

class _BrewingDetailScreenState extends State<BrewingDetailScreen>
    with WidgetsBindingObserver {
  Json? get data => widget.c.detail(widget.id);
  bool loading = true, acting = false;
  String? error, next;
  List<Json> readings = [];
  Timer? poll;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    unawaited(load());
    poll = Timer.periodic(const Duration(seconds: 20), (_) {
      if (WidgetsBinding.instance.lifecycleState == AppLifecycleState.resumed &&
          !acting &&
          !widget.c.busy) {
        unawaited(load());
      }
    });
  }

  @override
  void dispose() {
    poll?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) unawaited(load());
  }

  Future<void> load() async {
    try {
      await widget.c.initialize();
      if (widget.c.list == null) await widget.c.refresh();
      final fresh = await widget.c.load(widget.id, brewing: true);
      if (!mounted) return;
      error = null;
      readings = objects(object(fresh['measurements'])['readings']);
      next = object(fresh['measurements'])['next'] as String?;
    } catch (e) {
      error = e is ApiFailure ? e.code : 'request_failed';
      if (data != null && readings.isEmpty) {
        readings = objects(object(data!['measurements'])['readings']);
        next = object(data!['measurements'])['next'] as String?;
      }
    }
    if (mounted) setState(() => loading = false);
  }

  Future<void> command(Json command) async {
    setState(() => acting = true);
    await runAction(context, () async {
      await reviewBrewing(context, widget.c, command);
      await load();
    });
    if (mounted) setState(() => acting = false);
  }

  Future<void> timer(String id, String action) async {
    if (action == 'complete' &&
        !await confirm(
          context,
          'cookCompleteStep',
          body: 'cook_complete_confirm',
        )) {
      return;
    }
    if (mounted) {
      await command({
        'operation': 'timer',
        'run_id': widget.id,
        'step_id': id,
        'action': action,
      });
    }
  }

  Future<void> replan(String id, String kind) async {
    final run = object(data!['run']),
        moment = localMoment(object(data!['run'])['timezone'] as String);
    String? local;
    if (kind == 'timer') {
      final step = objects(run['steps'])
          .where((s) => object(s['stage'])['id'] == id)
          .firstOrNull;
      local = (step?['local_timer'] as Json?)?['deadline'] as String?;
    }
    final seed = local ?? processTime(run, id, 'start');
    if (seed.length >= 16) moment['local'] = seed.substring(0, 16);
    final changed = await openPage<Json>(
      context,
      ProcessMomentScreen(moment, kind),
    );
    if (changed != null && mounted) {
      await command({
        'operation': 'replan',
        'run_id': widget.id,
        'change': {
          'kind': kind,
          'step_id': id,
          kind == 'delay' ? 'start_after' : 'deadline': changed,
        },
      });
    }
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: widget.c,
    builder: (context, _) => Scaffold(
      appBar: AppBar(
        title: Text(
          data == null
              ? context.t('brewBatches')
              : object(data!['run'])['name'] as String,
        ),
        actions: [
          IconButton(
            tooltip: context.t('refresh'),
            icon: const Icon(Icons.refresh),
            onPressed: acting ? null : load,
          ),
        ],
      ),
      body: data == null
          ? loading
                ? const Center(child: CircularProgressIndicator())
                : EmptyContent(
                    icon: Icons.cloud_off_outlined,
                    title: error ?? 'request_failed',
                    action: 'cook_offline_copies',
                    onAction: () =>
                        openPage<void>(context, CookingCacheScreen(widget.c)),
                  )
          : RefreshIndicator(
              onRefresh: load,
              child: ListView(
                padding: const EdgeInsets.all(16),
                children: [
                  if (error != null)
                    Note(
                      '${context.t('cookOffline')}\n${context.t(error!)}',
                      warning: true,
                    ),
                  if (widget.c.clock.now == null)
                    Note(context.t('cooking_clock_unverified'), warning: true),
                  Text(
                    '${context.t('brewPinned')}: ${object(object(data!['run'])['brew_recipe'])['title']} · v${object(object(data!['run'])['brew_recipe'])['revision']} · ${brewQuantityText(context, object(object(data!['run'])['brew_recipe'])['volume'])}',
                  ),
                  OutlinedButton.icon(
                    icon: const Icon(Icons.menu_book_outlined),
                    label: Text(context.t('brewRecipes')),
                    onPressed: () => openPage<void>(
                      context,
                      Scaffold(
                        appBar: AppBar(title: Text(context.t('brewRecipes'))),
                        body: ListView(
                          padding: const EdgeInsets.all(16),
                          children: [
                            BrewRecipeSummary(
                              object(object(data!['recipe'])['recipe']),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                  Note(context.t('brewManualHelp')),
                  for (final equipment in objects(
                    object(data!['batch'])['equipment'],
                  ))
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      leading: const Icon(Icons.kitchen_outlined),
                      title: Text(
                        object(equipment['details'])['name'] as String,
                      ),
                      subtitle: Text(
                        '${object(equipment['details'])['model']} · ${brewQuantityText(context, object(equipment['details'])['capacity'])}',
                      ),
                    ),
                  BrewingPending(widget.c),
                  ProcessSteps(
                    run: object(data!['run']),
                    detail: data,
                    clock: widget.c.clock,
                    equipmentNames: {
                      for (final e in objects(
                        object(data!['batch'])['equipment'],
                      ))
                        e['id'] as String:
                            object(e['details'])['name'] as String,
                    },
                    writable:
                        widget.c.writable &&
                        widget.c.api.controller.online &&
                        error == null &&
                        widget.c.pending.isEmpty,
                    busy: acting,
                    timer: timer,
                    replan: replan,
                  ),
                  ProcessReminders(widget.c, data!, command: command),
                  SectionTitle(context.t('brewMeasurements')),
                  if (widget.c.writable)
                    OutlinedButton.icon(
                      icon: const Icon(Icons.add),
                      label: Text(context.t('brewAddReading')),
                      onPressed:
                          acting ||
                              !widget.c.api.controller.online ||
                              widget.c.pending.isNotEmpty
                          ? null
                          : () async {
                              await openPage<String>(
                                context,
                                BrewingReadingEditor(
                                  widget.c,
                                  object(data!['run']),
                                ),
                              );
                              await load();
                            },
                    ),
                  if (readings.isEmpty) Note(context.t('brewReadingEmpty')),
                  for (final reading in readings)
                    Padding(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          BrewReadingSummary(
                            object(reading['input']),
                            reason: reading['reason'] as String?,
                            corrects: reading['corrects'] as String?,
                          ),
                          Text(
                            '${context.t('brewEnteredAt')}: ${DateTime.fromMillisecondsSinceEpoch((reading['recorded_at'] as int) * 1000, isUtc: true).toIso8601String()} · ${reading['actor']}',
                          ),
                          if (readings.any(
                            (r) => r['corrects'] == reading['id'],
                          ))
                            Note(context.t('brewCorrected'))
                          else if (widget.c.writable)
                            TextButton.icon(
                              icon: const Icon(Icons.edit_outlined),
                              label: Text(context.t('brewCorrectReading')),
                              onPressed:
                                  acting ||
                                      !widget.c.api.controller.online ||
                                      widget.c.pending.isNotEmpty
                                  ? null
                                  : () async {
                                      await openPage<String>(
                                        context,
                                        BrewingReadingEditor(
                                          widget.c,
                                          object(data!['run']),
                                          original: reading,
                                        ),
                                      );
                                      await load();
                                    },
                            ),
                          const Divider(),
                        ],
                      ),
                    ),
                  if (next != null)
                    OutlinedButton(
                      onPressed: acting || !widget.c.api.controller.online
                          ? null
                          : () async {
                              setState(() => acting = true);
                              await runAction(context, () async {
                                final page = await widget.c.api.household(
                                  '/brewing/batches/${widget.id}/readings?before=${Uri.encodeComponent(next!)}',
                                );
                                if (mounted) {
                                  setState(() {
                                    readings.addAll(objects(page['readings']));
                                    next = page['next'] as String?;
                                  });
                                }
                              });
                              if (mounted) setState(() => acting = false);
                            },
                      child: Text(context.t('brewEarlier')),
                    ),
                  if (widget.c.writable) ...[
                    if (object(data!['run'])['status'] == 'active')
                      TextButton.icon(
                        icon: const Icon(Icons.cancel_outlined),
                        label: Text(context.t('cookCancelRun')),
                        onPressed: acting || !widget.c.api.controller.online
                            ? null
                            : () async {
                                if (await confirm(
                                      context,
                                      'cookCancelRun',
                                      body: 'cookCancelConfirm',
                                    ) &&
                                    mounted) {
                                  await command({
                                    'operation': 'cancel',
                                    'run_id': widget.id,
                                  });
                                }
                              },
                      )
                    else
                      TextButton(
                        onPressed: acting || !widget.c.api.controller.online
                            ? null
                            : () => command({
                                'operation': 'archive_batch',
                                'run_id': widget.id,
                                'archived':
                                    object(data!['batch'])['archived'] != true,
                              }),
                        child: Text(
                          context.t(
                            object(data!['batch'])['archived'] == true
                                ? 'brew_restore'
                                : 'brew_archive',
                          ),
                        ),
                      ),
                  ],
                ],
              ),
            ),
    ),
  );
}
