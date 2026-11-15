import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import '../brewing/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'brewing_common.dart';
import 'cooking_review.dart';
import 'messages.dart';

Future<Json?> reviewBrewing(
  BuildContext context,
  CookingController c,
  Json command, {
  bool replacing = false,
}) async {
  await c.initialize();
  if (c.list == null) await c.refresh();
  if (!c.writable ||
      !c.api.controller.online ||
      (!replacing && c.pending.isNotEmpty)) {
    throw const ApiFailure('cooking_pending_review');
  }
  final current = await c.api.household('/brewing');
  if (!context.mounted) return null;
  final cmd = clone(command);
  if (cmd['operation'] == 'save_recipe' && cmd['id'] != null) {
    final head = objects(current['recipes'])
        .map((r) => object(r['record']))
        .where((r) => r['id'] == cmd['id'])
        .firstOrNull;
    if (head != null && head['revision'] != cmd['expected_recipe_revision']) {
      if (!await confirm(
        context,
        'brew_review_changed',
        body: 'brew_review_changed_help',
      )) {
        return null;
      }
      cmd['expected_recipe_revision'] = head['revision'];
    }
  }
  if (cmd['operation'] == 'save_equipment' && cmd['id'] != null) {
    final equipment = objects(current['equipment'])
        .where((e) => e['id'] == cmd['id'])
        .firstOrNull;
    if (equipment != null &&
        brewEquipmentIdentity(object(equipment['details'])) !=
            brewEquipmentIdentity(cmd['expected_equipment'] as Json?)) {
      if (!context.mounted) return null;
      if (!await confirm(
        context,
        'brew_review_changed',
        body: 'brew_review_changed_help',
      )) {
        return null;
      }
      cmd['expected_equipment'] = clone(object(equipment['details']));
    }
  }
  final input = <String, dynamic>{
    'expected_revision': current['revision'],
    'idempotency_key': newId('brew-'),
    'command': cmd,
    'reviewed': false,
  };
  final preview = await c.api.household(
    '/brewing/preview',
    method: 'POST',
    body: input,
  );
  Json? previousReading;
  if (cmd['operation'] == 'add_reading' && cmd['corrects'] != null) {
    String? before;
    for (var page = 0; page < 40; page++) {
      final values = await c.api.household(
        '/brewing/batches/${cmd['run_id']}/readings${before == null ? '' : '?before=${Uri.encodeComponent(before)}'}',
      );
      previousReading = objects(values['readings'])
          .where((r) => r['id'] == cmd['corrects'])
          .firstOrNull;
      if (previousReading != null || values['next'] == null) break;
      before = values['next'] as String;
    }
  }
  if (!context.mounted) return null;
  return openPage<Json>(
    context,
    BrewingReviewScreen(
      c,
      input,
      preview,
      replacing: replacing,
      previousReading: previousReading,
    ),
  );
}

class BrewingReviewScreen extends StatefulWidget {
  const BrewingReviewScreen(
    this.c,
    this.input,
    this.preview, {
    super.key,
    this.replacing = false,
    this.previousReading,
  });
  final CookingController c;
  final Json input, preview;
  final bool replacing;
  final Json? previousReading;
  @override
  State<BrewingReviewScreen> createState() => _BrewingReviewScreenState();
}

class _BrewingReviewScreenState extends State<BrewingReviewScreen> {
  bool checked = false, busy = false, attempted = false, finished = false;
  String? error;
  @override
  Widget build(BuildContext context) {
    final command = object(widget.input['command']),
        before = object(widget.preview['before']),
        after = object(widget.preview['after']);
    final operation = command['operation'] as String;
    final batch = objects(after['batches'])
        .where(
          (b) => operation == 'create_batch'
              ? !objects(before['batches']).any(
                  (old) => object(old['run'])['id'] == object(b['run'])['id'],
                )
              : object(b['run'])['id'] == command['run_id'],
        )
        .firstOrNull;
    final oldBatch = batch == null
        ? null
        : objects(before['batches'])
              .where(
                (b) => object(b['run'])['id'] == object(batch['run'])['id'],
              )
              .firstOrNull;
    return DraftGuard(
      dirty: () =>
          attempted &&
          !finished &&
          widget.c.pending.any((p) => p['status'] == 'unknown'),
      child: Scaffold(
        appBar: AppBar(title: Text(context.t('brewReview'))),
        body: ListenableBuilder(
          listenable: widget.c,
          builder: (context, _) => ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Note(context.t('brewReviewHelp')),
              if (operation == 'timer') ...[
                Text(
                  context.t(
                    {
                      'start': 'cookStartStep',
                      'pause': 'cookPause',
                      'resume': 'cookResume',
                      'complete': 'cookCompleteStep',
                      'cancel': 'cookCancelStep',
                    }[command['action']]!,
                  ),
                ),
                if (command['action'] == 'start')
                  Note(context.t('brew_local_timer_notice')),
              ],
              if (operation == 'cancel') Text(context.t('cookCancelRun')),
              if (operation == 'archive_batch')
                Text(
                  context.t(
                    command['archived'] == true
                        ? 'brew_archive'
                        : 'brew_restore',
                  ),
                ),
              if (operation == 'subscribe')
                Text(
                  context.t(
                    command['enabled'] == true
                        ? 'cookNotify'
                        : 'cookStopNotify',
                  ),
                ),
              if (operation == 'acknowledge' || operation == 'retry')
                Text(
                  '${context.t(operation == 'retry' ? 'cookRetry' : 'cookAcknowledge')} · ${command['notice_id']}',
                ),
              if (error != null) Note(context.t(error!), warning: true),
              if (operation == 'save_recipe') ...[
                for (final old
                    in objects(before['recipes'])
                        .map((r) => object(r['record']))
                        .where((r) => r['id'] == command['id'])) ...[
                  SectionTitle(context.t('brew_before')),
                  BrewRecipeSummary(object(old['recipe'])),
                  const Divider(),
                ],
                SectionTitle(context.t('brew_after')),
                BrewRecipeSummary(object(command['recipe'])),
              ] else if (operation == 'save_equipment') ...[
                if (command['expected_equipment'] != null) ...[
                  SectionTitle(context.t('brew_before')),
                  _equipment(context, object(command['expected_equipment'])),
                ],
                SectionTitle(context.t('brew_after')),
                _equipment(context, object(command['equipment'])),
              ] else if (operation == 'add_reading') ...[
                if (widget.previousReading != null) ...[
                  SectionTitle(context.t('brew_before')),
                  BrewReadingSummary(object(widget.previousReading!['input'])),
                ],
                SectionTitle(context.t('brew_after')),
                BrewReadingSummary(
                  object(command['reading']),
                  corrects: command['corrects'] as String?,
                  reason: command['reason'] as String?,
                ),
              ] else if (batch != null) ...[
                ScheduleReview(
                  object(batch['run']),
                  previous: oldBatch == null ? null : object(oldBatch['run']),
                ),
                Note(context.t('brewManualHelp')),
                for (final equipment in objects(
                  object(batch['batch'])['equipment'],
                ))
                  _equipment(context, object(equipment['details'])),
              ] else ...[
                for (final recipe in objects(
                  after['recipes'],
                ).where((r) => object(r['record'])['id'] == command['id']))
                  BrewRecipeSummary(object(object(recipe['record'])['recipe'])),
                for (final equipment in objects(
                  after['equipment'],
                ).where((e) => e['id'] == command['id']))
                  _equipment(context, object(equipment['details'])),
                if (operation.startsWith('archive_'))
                  Text(
                    context.t(
                      command['archived'] == false
                          ? 'brew_restore'
                          : 'brew_archive',
                    ),
                  ),
              ],
              if (widget.c.pending.isNotEmpty)
                Note(context.t('brewRetryHelp'), warning: true),
              CheckboxListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(context.t('brewReviewed')),
                value: checked,
                onChanged: busy || attempted
                    ? null
                    : (v) => setState(() => checked = v == true),
              ),
              FilledButton(
                onPressed:
                    !checked ||
                        busy ||
                        !widget.c.writable ||
                        !widget.c.api.controller.online
                    ? null
                    : () async {
                        setState(() => busy = true);
                        try {
                          if (attempted) {
                            await widget.c.refresh();
                          } else {
                            attempted = true;
                            await widget.c.submitBrewing({
                              ...widget.input,
                              'reviewed': true,
                            }, replaceRejected: widget.replacing);
                          }
                          if (widget.c.pending.isEmpty && context.mounted) {
                            finished = true;
                            Navigator.pop(context, {
                              'result_id': widget.c.lastResult,
                            });
                          }
                        } catch (e) {
                          if (mounted) {
                            setState(
                              () => error = e is ApiFailure
                                  ? e.code
                                  : 'request_failed',
                            );
                          }
                        } finally {
                          if (mounted) setState(() => busy = false);
                        }
                      },
                child: Text(
                  context.t(attempted ? 'cook_recheck' : 'apply_reviewed'),
                ),
              ),
              if (widget.c.pending.isNotEmpty &&
                  widget.c.pending.first['status'] == 'rejected')
                TextButton(
                  onPressed: busy
                      ? null
                      : () async {
                          if (await confirm(
                                context,
                                'cook_discard_actions',
                                body: 'cook_discard_help',
                              ) &&
                              context.mounted) {
                            await widget.c.discardRejected();
                            if (context.mounted) {
                              finished = true;
                              Navigator.pop(context);
                            }
                          }
                        },
                  child: Text(context.t('cookEditDraft')),
                ),
              if (busy) const LinearProgressIndicator(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _equipment(BuildContext context, Json equipment) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          equipment['name'] as String,
          style: Theme.of(context).textTheme.titleMedium,
        ),
        Text(equipment['model'] as String),
        Text(
          '${context.t('brewCapacity')}: ${brewQuantityText(context, equipment['capacity'])}',
        ),
        if (equipment['notes'] != '') Text(equipment['notes'] as String),
      ],
    ),
  );
}

class BrewingPending extends StatelessWidget {
  const BrewingPending(this.c, {super.key});
  final CookingController c;
  @override
  Widget build(BuildContext context) => Column(
    children: [
      CookingPending(c),
      if (c.pending.isNotEmpty &&
          c.pending.first['domain'] == 'brewing' &&
          ['rejected', 'review_required'].contains(c.pending.first['status']))
        OutlinedButton(
          onPressed: !c.writable || c.busy
              ? null
              : () => runAction(context, () async {
                  await reviewBrewing(
                    context,
                    c,
                    object(object(c.pending.first['request'])['command']),
                    replacing: true,
                  );
                }),
          child: Text(context.t('brewKeepInputs')),
        ),
    ],
  );
}
