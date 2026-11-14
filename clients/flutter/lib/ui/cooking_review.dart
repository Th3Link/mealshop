import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'process_common.dart';

class ScheduleReview extends StatelessWidget {
  const ScheduleReview(
    this.run, {
    super.key,
    this.previous,
    this.warnings = const [],
  });
  final Json run;
  final Json? previous;
  final List<Json> warnings;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        run['name'] as String,
        style: Theme.of(context).textTheme.titleLarge,
      ),
      Text(run['timezone'] as String),
      if (run['recipe'] is Json)
        Text(
          '${object(run['recipe'])['title']} · v${object(run['recipe'])['revision']} · ${run['servings']} ${context.t('servings')}',
        ),
      for (final warning in warnings)
        Note(
          '${context.t(warning['code'] as String)}${(warning['steps'] as List? ?? []).isEmpty ? '' : ' · ${(warning['steps'] as List).join(', ')}'}',
          warning: true,
        ),
      for (final step in objects(run['steps']))
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                object(step['stage'])['name'] as String,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              if (previous != null)
                Text(
                  '${context.t('cookPreviousStart')}: ${processTime(previous!, object(step['stage'])['id'] as String, 'start')}',
                ),
              Text(
                '${context.t('cookPlannedStart')}: ${processTime(run, object(step['stage'])['id'] as String, 'start')}',
              ),
              Text(
                '${context.t('cookPlannedFinish')}: ${processTime(run, object(step['stage'])['id'] as String, 'finish')}',
              ),
              if (processTime(
                    run,
                    object(step['stage'])['id'] as String,
                    'earliest',
                  ) !=
                  processTime(
                    run,
                    object(step['stage'])['id'] as String,
                    'latest',
                  ))
                Text(
                  '${context.t('cookStartWindow')}: ${processTime(run, object(step['stage'])['id'] as String, 'earliest')} – ${processTime(run, object(step['stage'])['id'] as String, 'latest')}',
                ),
              if (object(
                    step['local_timer'] ?? <String, dynamic>{},
                  )['deadline'] !=
                  null)
                Text(
                  '${context.t('cookDeadline')}: ${object(step['local_timer'])['deadline']}',
                ),
              Text(
                context.t(
                  {
                    'completed': 'cookCompleted',
                    'paused': 'cookPaused',
                    'pending': 'cook_not_started',
                    'cancelled': 'cookCancelled',
                    'running': 'cookRemaining',
                  }[object(step['timer'])['state']]!,
                ),
              ),
            ],
          ),
        ),
    ],
  );
}

class CookingReviewScreen extends StatefulWidget {
  const CookingReviewScreen(
    this.c,
    this.preview, {
    super.key,
    this.retainedRequest,
  });
  final CookingController c;
  final Json preview;
  final Json? retainedRequest;
  @override
  State<CookingReviewScreen> createState() => _CookingReviewScreenState();
}

class _CookingReviewScreenState extends State<CookingReviewScreen> {
  bool checked = false, busy = false, attempted = false, finished = false;
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () =>
        attempted &&
        !finished &&
        widget.c.pending.any((p) => p['status'] == 'unknown'),
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(
            widget.retainedRequest == null
                ? 'cookReview'
                : 'cook_action_review',
          ),
        ),
      ),
      body: ListenableBuilder(
        listenable: widget.c,
        builder: (context, _) => ListView(
          padding: const EdgeInsets.all(16),
          children: [
            if (widget.retainedRequest != null)
              Text(
                '${context.t('cook_action_time')}: ${DateTime.fromMillisecondsSinceEpoch((widget.retainedRequest!['occurred_at'] as int) * 1000, isUtc: true).toIso8601String()}',
              ),
            ScheduleReview(
              object(widget.preview['run']),
              previous: widget.preview['previous'] as Json?,
              warnings: objects(
                (widget.preview['proposal'] as Json?)?['warnings'] ?? [],
              ),
            ),
            if (widget.c.pending.isNotEmpty)
              Note(context.t('cook_pending_help'), warning: true),
            if (widget.c.error != null)
              Note(context.t(widget.c.error!), warning: true),
            CheckboxListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(context.t('cookReviewed')),
              value: checked,
              onChanged: busy || attempted
                  ? null
                  : (v) => setState(() => checked = v == true),
            ),
            FilledButton(
              onPressed: busy || !checked || !widget.c.writable
                  ? null
                  : () async {
                      setState(() => busy = true);
                      await runAction(context, () async {
                        if (attempted) {
                          await widget.c.refresh();
                        } else {
                          attempted = true;
                          if (widget.retainedRequest != null) {
                            await widget.c.confirmReviewed(
                              widget.retainedRequest!,
                            );
                          } else {
                            await widget.c.enqueue(
                              {
                                'operation': 'apply',
                                'preview_id': widget.preview['preview_id'],
                                'reviewed': true,
                              },
                              expectedRevision:
                                  widget.preview['revision'] as int,
                            );
                          }
                        }
                        if (widget.c.pending.isEmpty && context.mounted) {
                          finished = true;
                          Navigator.pop(context, widget.c.lastResult);
                        }
                      });
                      if (mounted) setState(() => busy = false);
                    },
              child: Text(
                context.t(
                  attempted
                      ? 'cook_recheck'
                      : widget.retainedRequest == null
                      ? 'cookApply'
                      : 'cook_action_confirm',
                ),
              ),
            ),
            if (widget.c.pending.isNotEmpty &&
                widget.c.pending.every((p) => p['status'] != 'unknown'))
              TextButton(
                onPressed: busy
                    ? null
                    : () async {
                        if (!await confirm(
                          context,
                          'cook_discard_actions',
                          body: 'cook_discard_help',
                        )) {
                          return;
                        }
                        if (!context.mounted) return;
                        await runAction(context, () async {
                          await widget.c.discardRejected();
                          if (context.mounted) {
                            finished = true;
                            Navigator.pop(context);
                          }
                        });
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

class CookingPending extends StatelessWidget {
  const CookingPending(this.c, {super.key});
  final CookingController c;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      if (c.pending.isNotEmpty) ...[
        SectionTitle('${c.pending.length} · ${context.t('cook_pending')}'),
        Note(context.t('cook_pending_help'), warning: true),
        for (final entry in c.pending)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Text(
              '${context.t(entry['status'] == 'unknown'
                  ? 'cook_unknown'
                  : entry['status'] == 'pending'
                  ? 'cook_timer_pending'
                  : 'cook_rejected')}${entry['error'] == null ? '' : ' · ${context.t(entry['error'] as String)}'}',
            ),
          ),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            OutlinedButton(
              onPressed: c.busy ? null : () => c.refresh(),
              child: Text(context.t('cook_recheck')),
            ),
            if (c.pending.first['domain'] != 'brewing' &&
                c.pending.first['status'] != 'unknown' &&
                c.pending.first['status'] != 'pending' &&
                object(
                      object(c.pending.first['request'])['command'],
                    )['operation'] ==
                    'timer')
              OutlinedButton(
                onPressed: c.busy || !c.writable
                    ? null
                    : () => runAction(context, () async {
                        final (request, preview) = await c.reviewRejected();
                        if (context.mounted) {
                          await openPage<String>(
                            context,
                            CookingReviewScreen(
                              c,
                              preview,
                              retainedRequest: request,
                            ),
                          );
                        }
                      }),
                child: Text(context.t('cook_pending_review')),
              ),
            if (c.pending.every((p) => p['status'] != 'unknown'))
              TextButton(
                onPressed: c.busy
                    ? null
                    : () async {
                        if (await confirm(
                              context,
                              'cook_discard_actions',
                              body: 'cook_discard_help',
                            ) &&
                            context.mounted) {
                          await runAction(context, c.discardRejected);
                        }
                      },
                child: Text(context.t('cook_discard_actions')),
              ),
          ],
        ),
        const Divider(),
      ],
    ],
  );
}
