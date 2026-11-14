import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import 'common.dart';
import 'content_common.dart';
import 'cooking_scope.dart';
import 'cooking_setup.dart';
import 'cooking_review.dart';
import 'messages.dart';
import 'process_common.dart';
import 'recipes.dart';
import 'stock_cooking.dart';

class CookingScreen extends StatefulWidget {
  const CookingScreen(this.api, {super.key, this.controller});
  final HouseholdApi api;
  final CookingController? controller;
  @override
  State<CookingScreen> createState() => _CookingScreenState();
}

class _CookingScreenState extends State<CookingScreen> {
  CookingController? c;
  bool owned = false;
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (c == null) {
      c = widget.controller ?? CookingScope.maybeOf(context);
      owned = c == null;
      c ??= CookingController(widget.api);
      unawaited(_load());
    }
  }

  Future<void> _load() async {
    try {
      await c!.initialize();
      await c!.refresh();
    } catch (e) {
      if (mounted) {
        c!.error = e is ApiFailure ? e.code : 'request_failed';
        setState(() {});
      }
    }
  }

  @override
  void dispose() {
    if (owned) c?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: c!,
    builder: (context, _) => Scaffold(
      appBar: AppBar(
        title: Text(context.t('cookingSessions')),
        actions: [
          IconButton(
            tooltip: context.t('refresh'),
            icon: const Icon(Icons.refresh),
            onPressed: c!.busy ? null : c!.refresh,
          ),
        ],
      ),
      floatingActionButton: c!.writable
          ? FloatingActionButton.extended(
              onPressed:
                  c!.busy ||
                      c!.pending.isNotEmpty ||
                      !widget.api.controller.online
                  ? null
                  : () async {
                      final recipe = await pickCookingRecipe(
                        context,
                        widget.api,
                      );
                      if (recipe == null || !context.mounted) return;
                      final id = await openPage<String>(
                        context,
                        CookingSetupScreen(c!, recipe),
                      );
                      if (id != null && context.mounted) {
                        await openPage<void>(
                          context,
                          CookingDetailScreen(c!, id),
                        );
                      }
                      await c!.refresh();
                    },
              icon: const Icon(Icons.add),
              label: Text(context.t('cookNew')),
            )
          : null,
      body: !c!.initialized
          ? const Center(child: CircularProgressIndicator())
          : RefreshIndicator(
              onRefresh: c!.refresh,
              child: ListView(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 112),
                children: [
                  Note(context.t('cookOverviewHelp')),
                  if (c!.offline) Note(context.t('cookOffline'), warning: true),
                  if (c!.error != null)
                    Note(context.t(c!.error!), warning: true),
                  if (c!.list != null && !c!.protocol)
                    Note(context.t('mobile_upgrade_required'), warning: true),
                  if (!c!.writable) Note(context.t('household_read_only')),
                  CookingPending(c!),
                  if (c!.list == null)
                    Note(
                      context.t(
                        c!.offline ? 'cook_empty_cache' : 'cookEmptyHelp',
                      ),
                    ),
                  if (c!.list != null && objects(c!.list!['runs']).isEmpty)
                    Padding(
                      padding: const EdgeInsets.symmetric(vertical: 32),
                      child: Column(
                        children: [
                          const Icon(Icons.restaurant_outlined, size: 48),
                          const SizedBox(height: 16),
                          Text(
                            context.t('cookEmpty'),
                            style: Theme.of(context).textTheme.titleLarge,
                          ),
                          Text(context.t('cookEmptyHelp')),
                        ],
                      ),
                    ),
                  for (final run in objects(c!.list?['runs'] ?? []))
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      leading: Icon(
                        run['status'] == 'completed'
                            ? Icons.check_circle_outline
                            : run['status'] == 'cancelled'
                            ? Icons.cancel_outlined
                            : Icons.restaurant_outlined,
                      ),
                      title: Text(run['name'] as String),
                      subtitle: Text(
                        '${context.t(run['status'] == 'active'
                            ? 'cookRunActive'
                            : run['status'] == 'completed'
                            ? 'cookCompleted'
                            : 'cookCancelled')} · ${run['running']} ${context.t('cookRunningTimers')}${run['next'] == null ? '' : '\n${object(run['next'])['name']}'}',
                      ),
                      trailing: const Icon(Icons.chevron_right),
                      onTap: () => openPage<void>(
                        context,
                        CookingDetailScreen(c!, run['id'] as String),
                      ),
                    ),
                  Note(context.t('cook_cache_help')),
                  TextButton.icon(
                    icon: const Icon(Icons.offline_pin_outlined),
                    label: Text(context.t('cook_offline_copies')),
                    onPressed: () =>
                        openPage<void>(context, CookingCacheScreen(c!)),
                  ),
                ],
              ),
            ),
    ),
  );
}

class CookingDetailScreen extends StatefulWidget {
  const CookingDetailScreen(this.c, this.id, {super.key});
  final CookingController c;
  final String id;
  @override
  State<CookingDetailScreen> createState() => _CookingDetailScreenState();
}

class _CookingDetailScreenState extends State<CookingDetailScreen>
    with WidgetsBindingObserver {
  String? failure;
  bool loading = true, acting = false;
  Timer? poll;
  Json? get data => widget.c.detail(widget.id);
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    unawaited(load());
    poll = Timer.periodic(const Duration(seconds: 15), (_) {
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
      await widget.c.load(widget.id);
      failure = null;
    } catch (e) {
      failure = e is ApiFailure ? e.code : 'request_failed';
    }
    if (mounted) setState(() => loading = false);
  }

  Future<void> act(Future<void> Function() action) async {
    setState(() => acting = true);
    await runAction(context, action);
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
    if (!mounted) return;
    await act(
      () => widget.c.enqueue({
        'operation': 'timer',
        'run_id': widget.id,
        'step_id': id,
        'action': action,
      }, expectedRevision: data!['revision'] as int),
    );
  }

  Future<void> replan(String id, String kind) async {
    if (!widget.c.api.controller.online) {
      await showFailure(context, const ApiFailure('online_required'));
      return;
    }
    final run = object(data!['run']);
    final seed = kind == 'target' && run['target'] != null
        ? clone(object(run['target']))
        : localMoment(run['timezone'] as String);
    final String? local;
    if (kind == 'timer') {
      final step = objects(run['steps'])
          .where((s) => object(s['stage'])['id'] == id)
          .firstOrNull;
      local = (step?['local_timer'] as Json?)?['deadline'] as String?;
    } else {
      local = processTime(run, id, 'start');
    }
    if (local != null && local.length >= 16) {
      seed['local'] = local.substring(0, 16);
    }
    final changed = await openPage<Json>(context, _ReplanMoment(seed, kind));
    if (changed == null || !mounted) return;
    await act(() async {
      await widget.c.refresh();
      final preview = await widget.c.api.household(
        '/cooking/preview',
        method: 'POST',
        body: {
          'expected_revision': widget.c.list!['revision'],
          'proposal': {
            'kind': 'replan',
            'run_id': widget.id,
            'change': {
              'kind': kind,
              if (kind != 'target') 'step_id': id,
              kind == 'target'
                      ? 'target'
                      : kind == 'delay'
                      ? 'start_after'
                      : 'deadline':
                  changed,
            },
          },
        },
      );
      if (mounted) {
        await openPage<String>(context, CookingReviewScreen(widget.c, preview));
      }
      await load();
    });
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: widget.c,
    builder: (context, _) => Scaffold(
      appBar: AppBar(
        title: Text(
          data == null
              ? context.t('cookingSessions')
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
                    title: failure ?? 'cook_empty_cache',
                    action: 'retry',
                    onAction: load,
                  )
          : RefreshIndicator(
              onRefresh: load,
              child: ListView(
                padding: const EdgeInsets.all(16),
                children: [
                  if (failure != null)
                    Note(
                      '${context.t('cookOffline')}\n${context.t(failure!)}',
                      warning: true,
                    ),
                  if (widget.c.clock.now == null)
                    Note(context.t('cooking_clock_unverified'), warning: true),
                  Note(context.t('cookPinnedIngredients')),
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(
                      object(object(data!['run'])['recipe'])['title'] as String,
                    ),
                    subtitle: Text(
                      'v${object(object(data!['run'])['recipe'])['revision']} · ${object(data!['run'])['servings']} ${context.t('servings')}',
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () => openPage<void>(
                      context,
                      RecipeDetail(
                        widget.c.api,
                        object(object(data!['run'])['recipe'])['id'] as String,
                        revision:
                            object(object(data!['run'])['recipe'])['revision']
                                as int,
                      ),
                    ),
                  ),
                  CookingPending(widget.c),
                  if (object(data!['run'])['status'] == 'active' &&
                      widget.c.writable)
                    OutlinedButton.icon(
                      icon: const Icon(Icons.event),
                      label: Text(context.t('cookChangeTarget')),
                      onPressed: acting ? null : () => replan('', 'target'),
                    ),
                  ProcessSteps(
                    run: object(data!['run']),
                    detail: data,
                    clock: widget.c.clock,
                    writable:
                        widget.c.writable &&
                        widget.c.clock.trusted &&
                        !widget.c.blocked,
                    busy: acting || widget.c.busy,
                    timer: timer,
                    replan: replan,
                    pending: widget.c.pending,
                  ),
                  SectionTitle(context.t('cookNotifications')),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('cook_local_reminders')),
                    subtitle: Text(context.t('cook_local_inexact')),
                    value: widget.c.enabled(widget.id),
                    onChanged: acting
                        ? null
                        : (v) => act(() => widget.c.setReminders(widget.id, v)),
                  ),
                  Note(context.t('cook_local_help')),
                  if (widget.c.deviceStatus['precision'] == 'inexact')
                    TextButton.icon(
                      icon: const Icon(Icons.alarm),
                      label: Text(context.t('cook_allow_precise')),
                      onPressed: acting
                          ? null
                          : () => act(() async {
                              await widget.c.device.requestPrecision();
                              await widget.c.reconcileReminders();
                            }),
                    ),
                  Note(context.t('cook_local_horizon')),
                  if (widget.c.deviceStatus['permission'] != 'granted')
                    Note(
                      context.t(
                        widget.c.deviceStatus['permission'] == 'denied'
                            ? 'cookPermissionDeniedShort'
                            : 'cookPermissionUnsupportedShort',
                      ),
                      warning: true,
                    ),
                  if (widget.c.limitedReminders)
                    Note(context.t('cook_local_limited'), warning: true),
                  if (data!['notification_transport'] == true &&
                      object(data!['run'])['status'] == 'active')
                    SwitchListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(context.t('cookSubscribe')),
                      subtitle: Text(context.t('cookNotificationHelp')),
                      value:
                          (object(data!['run'])['subscription']
                              as Json?)?['enabled'] ==
                          true,
                      onChanged:
                          !widget.c.writable ||
                              acting ||
                              !widget.c.api.controller.online
                          ? null
                          : (v) => act(
                              () => widget.c.enqueue({
                                'operation': 'subscribe',
                                'run_id': widget.id,
                                'enabled': v,
                                'language': widget.c.api.controller.language,
                                'permission': 'default',
                              }, expectedRevision: data!['revision'] as int),
                            ),
                    ),
                  for (final notice in objects(data!['notices']))
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(
                        context.t(
                          {
                            'prepare': 'cookEventPrepare',
                            'start': 'cookEventStart',
                            'active_ended': 'cookEventActive',
                            'timer_ended': 'cookEventEnd',
                          }[notice['event']]!,
                        ),
                      ),
                      subtitle: Text(
                        '${context.t({'queued': 'cookDeliveryQueued', 'sending': 'cookDeliverySending', 'accepted': 'cookDeliveryAccepted', 'failed': 'cookDeliveryFailed', 'unknown': 'cookDeliveryUnknown', 'missed': 'cookDeliveryMissed', 'cancelled': 'cookCancelled'}[object(notice['delivery'])['state']]!)}${notice['acknowledged_at'] == null ? '' : ' · ${context.t('cookAcknowledged')}'}',
                      ),
                      trailing: PopupMenuButton<String>(
                        tooltip: context.t('details'),
                        itemBuilder: (context) => [
                          PopupMenuItem(
                            value: 'acknowledge',
                            child: Text(context.t('cookAcknowledge')),
                          ),
                          if ([
                            'failed',
                            'unknown',
                          ].contains(object(notice['delivery'])['state']))
                            PopupMenuItem(
                              value: 'retry',
                              child: Text(context.t('retry')),
                            ),
                        ],
                        enabled:
                            widget.c.writable &&
                            !acting &&
                            widget.c.api.controller.online,
                        onSelected: (action) async {
                          final risk =
                              object(notice['delivery'])['state'] == 'unknown';
                          if (action == 'retry' &&
                              risk &&
                              !await confirm(
                                context,
                                'retry',
                                body: 'cookRetryRisk',
                              )) {
                            return;
                          }
                          if (mounted) {
                            await act(
                              () => widget.c.enqueue({
                                'operation': action,
                                'notice_id': notice['id'],
                                if (action == 'retry')
                                  'duplicate_risk_accepted': risk,
                              }, expectedRevision: data!['revision'] as int),
                            );
                          }
                        },
                      ),
                    ),
                  if (object(data!['run'])['status'] == 'completed' &&
                      widget.c.writable)
                    OutlinedButton.icon(
                      icon: const Icon(Icons.kitchen_outlined),
                      label: Text(context.t('record_cooking_stock')),
                      onPressed: acting || !widget.c.api.controller.online
                          ? null
                          : () => openPage<void>(
                              context,
                              CookingStockEditor(
                                widget.c.api,
                                object(data!['run']),
                              ),
                            ),
                    ),
                  if (widget.c.writable)
                    TextButton.icon(
                      icon: const Icon(Icons.cancel_outlined),
                      label: Text(
                        context.t(
                          object(data!['run'])['status'] == 'active'
                              ? 'cookCancelRun'
                              : 'cookRemoveRun',
                        ),
                      ),
                      onPressed: acting || !widget.c.api.controller.online
                          ? null
                          : () async {
                              final active =
                                  object(data!['run'])['status'] == 'active';
                              if (!await confirm(
                                context,
                                active ? 'cookCancelRun' : 'cookRemoveRun',
                                body: active
                                    ? 'cookCancelConfirm'
                                    : 'cookRemoveConfirm',
                              )) {
                                return;
                              }
                              if (!mounted) return;
                              await act(() async {
                                await widget.c.enqueue({
                                  'operation': active ? 'cancel' : 'remove',
                                  'run_id': widget.id,
                                }, expectedRevision: data!['revision'] as int);
                                if (!active &&
                                    widget.c.pending.isEmpty &&
                                    context.mounted) {
                                  Navigator.pop(context);
                                }
                              });
                            },
                    ),
                ],
              ),
            ),
    ),
  );
}

class _ReplanMoment extends StatefulWidget {
  const _ReplanMoment(this.value, this.kind);
  final Json value;
  final String kind;
  @override
  State<_ReplanMoment> createState() => _ReplanMomentState();
}

class CookingCacheScreen extends StatelessWidget {
  const CookingCacheScreen(this.c, {super.key});
  final CookingController c;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: c,
    builder: (context, _) => Scaffold(
      appBar: AppBar(title: Text(context.t('cook_offline_copies'))),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Note(context.t('cook_cache_help')),
          for (final entry in c.sessions.entries)
            ListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(object(object(entry.value)['run'])['name'] as String),
              trailing: IconButton(
                tooltip: context.t('cook_forget_cache'),
                icon: const Icon(Icons.delete_outline),
                onPressed: c.pending.isNotEmpty
                    ? null
                    : () async {
                        if (await confirm(
                              context,
                              'cook_forget_cache',
                              body: 'cook_cache_help',
                            ) &&
                            context.mounted) {
                          await runAction(context, () => c.forget(entry.key));
                        }
                      },
              ),
            ),
        ],
      ),
    ),
  );
}

class _ReplanMomentState extends State<_ReplanMoment> {
  late Json value = clone(widget.value);
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('cookPreview'))),
    body: ListView(
      padding: const EdgeInsets.all(16),
      children: [
        LocalMomentEditor(
          widget.kind == 'target'
              ? 'cookTarget'
              : widget.kind == 'delay'
              ? 'cookStartAfter'
              : 'cookDeadline',
          value,
          (v) => setState(() => value = v),
          zone: false,
        ),
        Text(value['timezone'] as String),
        FilledButton(
          onPressed: () => Navigator.pop(context, value),
          child: Text(context.t('cookPreview')),
        ),
      ],
    ),
  );
}
