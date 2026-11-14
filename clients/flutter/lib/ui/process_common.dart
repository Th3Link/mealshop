import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/device.dart';
import '../planning/data.dart' show isoDay;
import 'content_common.dart';
import 'common.dart';
import 'messages.dart';
import 'plan_common.dart';

String durationText(int seconds) {
  final value = seconds.abs();
  return '${value ~/ 3600}:${(value ~/ 60 % 60).toString().padLeft(2, '0')}:${(value % 60).toString().padLeft(2, '0')}';
}

String processTime(Json run, String step, String field) =>
    objects(run['local_schedule'])
            .where((t) => t['id'] == step)
            .firstOrNull?[field]
        as String? ??
    '';

Json localMoment(String timezone, [DateTime? day]) => {
  'local':
      '${isoDay(day ?? DateTime.now())}T${TimeOfDay.now().hour.toString().padLeft(2, '0')}:${TimeOfDay.now().minute.toString().padLeft(2, '0')}',
  'timezone': timezone,
  'fold': 'reject',
};

class LocalMomentEditor extends StatelessWidget {
  const LocalMomentEditor(
    this.label,
    this.value,
    this.changed, {
    super.key,
    this.zone = true,
  });
  final String label;
  final Json value;
  final ValueChanged<Json> changed;
  final bool zone;
  @override
  Widget build(BuildContext context) {
    final local = value['local'] as String,
        date = local.substring(0, 10),
        time = local.substring(11, 16);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SectionTitle(context.t(label)),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            DayButton(
              'cook_date',
              date,
              (v) => changed({...value, 'local': '${v}T$time'}),
            ),
            OutlinedButton.icon(
              icon: const Icon(Icons.schedule),
              label: Text('${context.t('cook_time')} · $time'),
              onPressed: () async {
                final selected = await showTimePicker(
                  context: context,
                  initialTime: TimeOfDay(
                    hour: int.parse(time.substring(0, 2)),
                    minute: int.parse(time.substring(3, 5)),
                  ),
                );
                if (selected != null) {
                  changed({
                    ...value,
                    'local':
                        '${date}T${selected.hour.toString().padLeft(2, '0')}:${selected.minute.toString().padLeft(2, '0')}',
                  });
                }
              },
            ),
          ],
        ),
        if (zone)
          EditText(
            'cookTimezone',
            value['timezone'] as String,
            (v) => changed({...value, 'timezone': v}),
            required: true,
            max: 100,
            hint: 'cookTimeZoneHelp',
          ),
        Choice('cookFoldChoice', value['fold'] as String, {
          'reject': context.t('cookFoldReject'),
          'earlier': context.t('cookFoldEarlier'),
          'later': context.t('cookFoldLater'),
        }, (v) => changed({...value, 'fold': v})),
      ],
    );
  }
}

class DurationField extends StatefulWidget {
  const DurationField(
    this.label,
    this.seconds,
    this.changed, {
    super.key,
    this.maximum = 604800,
  });
  final String label;
  final int seconds, maximum;
  final ValueChanged<int> changed;
  @override
  State<DurationField> createState() => _DurationFieldState();
}

class _DurationFieldState extends State<DurationField> {
  late int unit = widget.seconds > 0 && widget.seconds % 86400 == 0
      ? 86400
      : widget.seconds > 0 && widget.seconds % 3600 == 0
      ? 3600
      : widget.seconds > 0 && widget.seconds % 60 == 0
      ? 60
      : 1;
  late final text = TextEditingController(text: '${widget.seconds ~/ unit}');
  void changed() {
    final number = int.tryParse(text.text);
    if (number != null && number >= 0 && number <= widget.maximum ~/ unit) {
      widget.changed(number * unit);
    }
  }

  @override
  void dispose() {
    text.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TextFormField(
          key: ValueKey(widget.label),
          controller: text,
          keyboardType: TextInputType.number,
          decoration: InputDecoration(labelText: context.t(widget.label)),
          validator: (v) {
            final n = int.tryParse(v ?? '');
            return n == null || n < 0 || n > widget.maximum ~/ unit
                ? context.t('cookDurationError')
                : null;
          },
          onChanged: (_) => changed(),
        ),
        Wrap(
          spacing: 8,
          children: [
            for (final option in <int, String>{
              1: 'cookSeconds',
              60: 'cookMinutes',
              3600: 'cookHours',
              86400: 'cookDays',
            }.entries)
              ChoiceChip(
                label: Text(context.t(option.value)),
                selected: unit == option.key,
                onSelected: (_) {
                  setState(() => unit = option.key);
                  changed();
                },
              ),
          ],
        ),
      ],
    ),
  );
}

class ProcessSteps extends StatefulWidget {
  const ProcessSteps({
    super.key,
    required this.run,
    required this.clock,
    required this.writable,
    required this.busy,
    required this.timer,
    required this.replan,
    this.detail,
    this.pending = const [],
  });
  final Json run;
  final Json? detail;
  final CookingClock clock;
  final bool writable, busy;
  final List<Json> pending;
  final Future<void> Function(String, String) timer;
  final Future<void> Function(String, String) replan;
  @override
  State<ProcessSteps> createState() => _ProcessStepsState();
}

class _ProcessStepsState extends State<ProcessSteps> {
  Timer? tick;
  @override
  void initState() {
    super.initState();
    tick = Timer.periodic(const Duration(seconds: 1), (_) {
      if (mounted) setState(() {});
    });
  }

  @override
  void dispose() {
    tick?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final steps = objects(widget.run['steps']);
    final active = widget.run['status'] == 'active';
    final brewing = [
      'beer',
      'cider',
      'wine',
      'mead',
    ].contains(widget.run['kind']);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (final step in steps) ...[
          Builder(
            builder: (context) {
              final stage = object(step['stage']),
                  timer = object(step['timer']);
              final state = timer['state'] as String,
                  id = stage['id'] as String;
              final recorded = widget.pending
                  .where(
                    (p) =>
                        object(object(p['request'])['command'])['step_id'] ==
                        id,
                  )
                  .toList();
              final available =
                  widget.writable && !widget.busy && active && recorded.isEmpty;
              final now = widget.clock.now;
              final manual =
                  brewing &&
                  state == 'running' &&
                  timer['deadline'] == timer['started_at'];
              final remaining = state == 'running' && now != null
                  ? (timer['deadline'] as int) - now
                  : state == 'paused'
                  ? timer['remaining'] as int
                  : null;
              final text = state == 'pending'
                  ? step['ready'] == true
                        ? 'cookReady'
                        : 'cookBlocked'
                  : state == 'completed'
                  ? 'cookCompleted'
                  : state == 'cancelled'
                  ? 'cookCancelled'
                  : state == 'paused'
                  ? 'cookPaused'
                  : manual
                  ? 'brewWorking'
                  : remaining != null && remaining <= 0
                  ? 'cookElapsed'
                  : 'cookRemaining';
              final origin = object(stage['origin']),
                  scope = widget.detail == null
                      ? null
                      : objects(widget.detail!['step_ingredients'] ?? [])
                            .where((s) => s['index'] == origin['index'])
                            .firstOrNull;
              final lines = widget.detail == null
                  ? <Json>[]
                  : objects(widget.detail!['ingredients'] ?? []);
              final localTimer = object(
                step['local_timer'] ?? <String, dynamic>{},
              );
              return Padding(
                padding: const EdgeInsets.symmetric(vertical: 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      stage['name'] as String,
                      style: Theme.of(context).textTheme.titleLarge,
                    ),
                    const SizedBox(height: 8),
                    SelectableText(
                      stage['instruction'] as String,
                      style: Theme.of(context).textTheme.bodyLarge,
                    ),
                    const SizedBox(height: 12),
                    Text(
                      context.t(text),
                      style: Theme.of(context).textTheme.labelLarge,
                    ),
                    if (!manual && remaining != null)
                      Text(
                        durationText(remaining),
                        style: Theme.of(context).textTheme.headlineMedium
                            ?.copyWith(
                              fontFeatures: const [
                                FontFeature.tabularFigures(),
                              ],
                            ),
                      ),
                    if (localTimer['deadline'] != null)
                      Text(
                        '${context.t('cookDeadline')}: ${localTimer['deadline']}',
                      ),
                    if (state == 'pending')
                      Text(
                        '${context.t('cookPlannedStart')}: ${processTime(widget.run, id, 'start')}',
                      ),
                    if ((stage['dependencies'] as List).isNotEmpty)
                      Text(
                        '${context.t('cookDependencies')}: ${(stage['dependencies'] as List).map((id) => steps.where((s) => object(s['stage'])['id'] == id).firstOrNull?['stage']).map((s) => object(s)['name']).join(', ')}',
                      ),
                    if ((stage['equipment'] as List).isNotEmpty)
                      Text(
                        '${context.t('cookEquipment')}: ${(stage['equipment'] as List).join(', ')}',
                      ),
                    if ((stage['uncertainty'] as String? ?? '').isNotEmpty)
                      Note(stage['uncertainty'] as String),
                    if (scope != null && lines.isNotEmpty) ...[
                      const SizedBox(height: 12),
                      Text(
                        context.t(
                          scope['scope'] == 'unassigned'
                              ? 'cook_recipe_reference'
                              : 'cook_step_ingredients',
                        ),
                        style: Theme.of(context).textTheme.titleSmall,
                      ),
                      for (final index
                          in (scope['line_indexes'] as List).cast<int>())
                        if (index >= 0 && index < lines.length)
                          Padding(
                            padding: const EdgeInsets.symmetric(vertical: 3),
                            child: Text(
                              '${(lines[index]['translations'] as Json?)?[Localizations.localeOf(context).languageCode]?['name'] ?? lines[index]['name']} · ${exactNumber(context, lines[index]['grams'] as String)} g${lines[index]['prepared_part_name'] == null ? '' : ' · ${lines[index]['prepared_part_name']}'}',
                            ),
                          ),
                    ],
                    if (widget.detail?['recipe'] is Json && brewing) ...[
                      for (final line
                          in objects(
                            object(
                              object(widget.detail!['recipe'])['recipe'],
                            )['ingredients'],
                          ).where(
                            (line) =>
                                line['phase_id'] == null ||
                                line['phase_id'] == id ||
                                line['phase_id'] ==
                                    (id.endsWith('-1')
                                        ? id.substring(0, id.length - 2)
                                        : id),
                          ))
                        Text(
                          '${line['name']} · ${line['quantity'] == null ? context.t('unknown') : '${exactNumber(context, object(line['quantity'])['amount'] as String)} ${object(line['quantity'])['unit']}'}',
                        ),
                    ],
                    for (final entry in recorded)
                      Note(
                        context.t(
                          entry['status'] == 'unknown'
                              ? 'cook_unknown'
                              : entry['status'] == 'pending'
                              ? 'cook_timer_pending'
                              : 'cook_rejected',
                        ),
                        warning: true,
                      ),
                    if (active)
                      Wrap(
                        spacing: 8,
                        runSpacing: 8,
                        children: [
                          if (state == 'pending')
                            FilledButton.icon(
                              onPressed: available && step['ready'] == true
                                  ? () => widget.timer(id, 'start')
                                  : null,
                              icon: const Icon(Icons.play_arrow),
                              label: Text(
                                context.t(
                                  stage['start_after_work'] == true
                                      ? 'cook_work_finished'
                                      : !brewing &&
                                            (stage['active_seconds'] as int) ==
                                                0 &&
                                            (stage['passive_seconds'] as int) ==
                                                0
                                      ? 'cook_manual_complete'
                                      : 'cookStartStep',
                                ),
                              ),
                            ),
                          if (state == 'running' && !manual)
                            OutlinedButton(
                              onPressed: available
                                  ? () => widget.timer(id, 'pause')
                                  : null,
                              child: Text(context.t('cookPause')),
                            ),
                          if (state == 'paused')
                            FilledButton(
                              onPressed: available
                                  ? () => widget.timer(id, 'resume')
                                  : null,
                              child: Text(context.t('cookResume')),
                            ),
                          if (['pending', 'running', 'paused'].contains(state))
                            OutlinedButton(
                              onPressed: available
                                  ? () => widget.timer(id, 'complete')
                                  : null,
                              child: Text(context.t('cookCompleteStep')),
                            ),
                          if (state == 'pending')
                            TextButton(
                              onPressed: available
                                  ? () => widget.replan(id, 'delay')
                                  : null,
                              child: Text(context.t('cookDelayStep')),
                            ),
                          if (state == 'running' || state == 'paused')
                            TextButton(
                              onPressed: available && !manual
                                  ? () => widget.replan(id, 'timer')
                                  : null,
                              child: Text(context.t('cookChangeTimer')),
                            ),
                        ],
                      ),
                  ],
                ),
              );
            },
          ),
          const Divider(),
        ],
      ],
    );
  }
}
