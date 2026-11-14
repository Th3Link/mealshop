import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_common.dart';

class AttendanceScreen extends StatefulWidget {
  const AttendanceScreen(this.api, this.data, {super.key});
  final HouseholdApi api;
  final PlanningData data;
  @override
  State<AttendanceScreen> createState() => _AttendanceScreenState();
}

class _AttendanceScreenState extends State<AttendanceScreen> {
  late final schedule = clone(
    object(
      widget.data.plan['attendance'] ??
          {'recurring': <Json>[], 'overrides': <Json>[]},
    ),
  );
  bool dirty = false, busy = false, saved = false;
  bool get hasObsoleteReferences {
    final known = widget.data.people.map((p) => p['id']).toSet();
    return objects(schedule['recurring'])
            .any((r) => !known.contains(r['member_id'])) ||
        objects(schedule['overrides']).any(
          (o) => (o['member_ids'] as List).any((id) => !known.contains(id)),
        );
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('attendance_schedule'))),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Note(context.t('attendance_note')),
          if (hasObsoleteReferences) ...[
            Note(context.t('obsolete_attendance_note'), warning: true),
            OutlinedButton(
              onPressed: busy
                  ? null
                  : () => setState(() {
                      final known = widget.data.people
                          .map((p) => p['id'])
                          .toSet();
                      schedule['recurring'] = objects(schedule['recurring'])
                          .where((r) => known.contains(r['member_id']))
                          .toList();
                      for (final row in objects(schedule['overrides'])) {
                        row['member_ids'] = (row['member_ids'] as List)
                            .where(known.contains)
                            .toList();
                      }
                      dirty = true;
                    }),
              child: Text(context.t('remove_obsolete_attendance')),
            ),
          ],
          for (final p in widget.data.people)
            Builder(
              builder: (context) {
                final rules = objects(schedule['recurring']);
                var r = rules
                    .where((r) => r['member_id'] == p['id'])
                    .firstOrNull;
                if (r == null) {
                  r = {
                    'member_id': p['id'],
                    'weekdays': [0, 1, 2, 3, 4, 5, 6],
                    'meals': List<String>.from(mealKeys),
                  };
                  (schedule['recurring'] as List).add(r);
                }
                final rule = r;
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      p['name'] as String,
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    Wrap(
                      spacing: 8,
                      children: [
                        for (var i = 0; i < 7; i++)
                          FilterChip(
                            label: Padding(
                              padding: const EdgeInsets.symmetric(vertical: 8),
                              child: Text(
                                MaterialLocalizations.of(context)
                                    .narrowWeekdays[(i + 1) % 7],
                              ),
                            ),
                            selected: (rule['weekdays'] as List).contains(i),
                            onSelected: busy
                                ? null
                                : (v) => setState(() {
                                    final days = rule['weekdays'] as List;
                                    v ? days.add(i) : days.remove(i);
                                    dirty = true;
                                  }),
                          ),
                      ],
                    ),
                    Wrap(
                      spacing: 8,
                      children: [
                        for (final meal in mealKeys)
                          FilterChip(
                            label: Padding(
                              padding: const EdgeInsets.symmetric(vertical: 8),
                              child: Text(context.t(meal)),
                            ),
                            selected: (rule['meals'] as List).contains(meal),
                            onSelected: busy
                                ? null
                                : (v) => setState(() {
                                    final meals = rule['meals'] as List;
                                    v ? meals.add(meal) : meals.remove(meal);
                                    dirty = true;
                                  }),
                          ),
                      ],
                    ),
                    const Divider(),
                  ],
                );
              },
            ),
          Text(
            context.t('date_overrides'),
            style: Theme.of(context).textTheme.titleMedium,
          ),
          for (final o in objects(schedule['overrides']))
            Column(
              key: ObjectKey(o),
              children: [
                DayButton(
                  'date',
                  o['date'] as String,
                  (v) => setState(() {
                    o['date'] = v;
                    dirty = true;
                  }),
                ),
                Choice(
                  'meal',
                  o['meal'] as String,
                  {for (final m in mealKeys) m: context.t(m)},
                  (v) => setState(() {
                    o['meal'] = v;
                    dirty = true;
                  }),
                ),
                AttendeeChoices(
                  widget.data,
                  (o['member_ids'] as List).cast<String>(),
                  (v) => setState(() {
                    o['member_ids'] = v;
                    dirty = true;
                  }),
                ),
                IconButton(
                  tooltip: context.t('delete'),
                  icon: const Icon(Icons.delete_outline),
                  onPressed: busy
                      ? null
                      : () => setState(() {
                          (schedule['overrides'] as List).remove(o);
                          dirty = true;
                        }),
                ),
              ],
            ),
          OutlinedButton.icon(
            icon: const Icon(Icons.add),
            label: Text(context.t('add_date_override')),
            onPressed: busy
                ? null
                : () => setState(() {
                    (schedule['overrides'] as List).add({
                      'date': isoDay(DateTime.now()),
                      'meal': 'dinner',
                      'member_ids': widget.data.people
                          .map((p) => p['id'])
                          .toList(),
                    });
                    dirty = true;
                  }),
          ),
          FilledButton(
            onPressed: busy || !widget.api.writable
                ? null
                : () async {
                    setState(() => busy = true);
                    try {
                      await widget.api.household(
                        '/mealplan/attendance',
                        method: 'POST',
                        body: {
                          'expected_revision': widget.data.plan['revision'],
                          'attendance': schedule,
                        },
                      );
                      saved = true;
                      if (context.mounted) Navigator.pop(context);
                    } catch (e) {
                      if (context.mounted) await showFailure(context, e);
                    } finally {
                      if (mounted) setState(() => busy = false);
                    }
                  },
            child: Text(context.t('save')),
          ),
        ],
      ),
    ),
  );
}
