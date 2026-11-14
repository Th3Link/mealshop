import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_common.dart';
import 'recipes.dart';

class MealEditor extends StatefulWidget {
  const MealEditor(
    this.api,
    this.data, {
    super.key,
    this.plan,
    this.date,
    this.meal,
  });
  final HouseholdApi api;
  final PlanningData data;
  final Json? plan;
  final String? date, meal;
  @override
  State<MealEditor> createState() => _MealEditorState();
}

class _MealEditorState extends State<MealEditor> {
  final form = GlobalKey<FormState>();
  late Json plan = widget.plan == null
      ? {
          'date': widget.date!,
          'meal': widget.meal!,
          'recipe': widget.data.recipes.isEmpty
              ? <String, dynamic>{}
              : {
                  'id': widget.data.recipes.first['id'],
                  'revision': widget.data.recipes.first['revision'],
                },
          'attendees': widget.data.attendees(widget.date!, widget.meal!),
          'calendar': null,
          'servings': widget.data
              .attendees(widget.date!, widget.meal!)
              .length
              .clamp(1, 100),
          'active_seconds': 1800,
          'elapsed_seconds': 3600,
          'advance_seconds': 0,
        }
      : clone(widget.plan!);
  bool dirty = false, busy = false, saved = false;
  final numbers = <String, String>{};
  @override
  void initState() {
    super.initState();
    for (final k in [
      'servings',
      'active_seconds',
      'elapsed_seconds',
      'advance_seconds',
    ]) {
      numbers[k] = '${plan[k]}';
    }
  }

  Future<void> review({bool remove = false}) async {
    if (!form.currentState!.validate()) return;
    setState(() => busy = true);
    try {
      if (widget.data.recipes.isEmpty) throw const ApiFailure('no_recipes');
      for (final e in numbers.entries) {
        plan[e.key] = int.parse(e.value);
      }
      final submitted = clone(plan);
      final result = await widget.api.household(
        '/mealplan/${remove ? 'remove-preview' : 'preview'}',
        method: 'POST',
        body: submitted,
      );
      if (remove && result['plan_revision'] != widget.data.plan['revision']) {
        throw const ApiFailure('revision_conflict', 409);
      }
      if (!mounted) return;
      final accepted = await openPage<bool>(
        context,
        MealReview(widget.api, widget.data, submitted, result, remove: remove),
      );
      if (accepted == true && mounted) {
        saved = true;
        Navigator.pop(context);
      }
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final recipes = <String, String>{
      for (final r in widget.data.recipes)
        r['id'] as String: r['title'] as String,
    };
    final reference = object(plan['recipe']);
    if (reference['id'] != null) {
      recipes.putIfAbsent(
        reference['id'] as String,
        () => widget.data.title(reference),
      );
    }
    return DraftGuard(
      dirty: () => !saved && dirty,
      child: Scaffold(
        appBar: AppBar(
          title: Text(
            context.t(widget.plan == null ? 'plan_meal' : 'edit_meal'),
          ),
        ),
        body: Form(
          key: form,
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Text(
                '${plan['date']} · ${context.t(plan['meal'] as String)}',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              if (recipes.isEmpty) Note(context.t('no_recipes')),
              Choice(
                'recipe',
                reference['id'] as String?,
                recipes,
                (id) => setState(() {
                  final r = widget.data.recipes.firstWhere(
                    (r) => r['id'] == id,
                  );
                  plan['recipe'] = {'id': id, 'revision': r['revision']};
                  dirty = true;
                }),
              ),
              Text('${context.t('revision')} ${reference['revision'] ?? '—'}'),
              EditText(
                'production_servings',
                numbers['servings']!,
                (v) {
                  numbers['servings'] = v;
                  dirty = true;
                },
                required: true,
                integer: true,
              ),
              AttendeeChoices(
                widget.data,
                (plan['attendees'] as List? ?? []).cast<String>(),
                (ids) => setState(() {
                  plan['attendees'] = ids;
                  dirty = true;
                }),
              ),
              Note(context.t('attendee_skip_note')),
              for (final k in [
                'active_seconds',
                'elapsed_seconds',
                'advance_seconds',
              ])
                EditText(
                  k,
                  numbers[k]!,
                  (v) {
                    numbers[k] = v;
                    dirty = true;
                  },
                  required: true,
                  integer: true,
                ),
              const SizedBox(height: 16),
              Text(
                context.t('meal_allocations'),
                style: Theme.of(context).textTheme.titleMedium,
              ),
              Note(context.t('planned_not_stock')),
              if (!plan.containsKey('allocations'))
                OutlinedButton.icon(
                  icon: const Icon(Icons.add),
                  label: Text(context.t('distribute_portions')),
                  onPressed: () => setState(() {
                    plan['allocations'] = [
                      {
                        'date': plan['date'],
                        'meal': plan['meal'],
                        'servings': int.tryParse(numbers['servings']!) ?? 1,
                      },
                    ];
                    dirty = true;
                  }),
                ),
              for (final a in objects(plan['allocations'] ?? []))
                _allocation(a),
              if (plan.containsKey('allocations')) ...[
                OutlinedButton.icon(
                  icon: const Icon(Icons.add),
                  label: Text(context.t('new_allocation')),
                  onPressed: () => setState(() {
                    (plan['allocations'] as List).add({
                      'date': plan['date'],
                      'meal': 'dinner',
                      'servings': 1,
                    });
                    dirty = true;
                  }),
                ),
                TextButton(
                  onPressed: () => setState(() {
                    plan.remove('allocations');
                    dirty = true;
                  }),
                  child: Text(context.t('single_meal')),
                ),
              ],
              const SizedBox(height: 16),
              if (plan['preparation'] == null)
                OutlinedButton.icon(
                  icon: const Icon(Icons.add),
                  label: Text(context.t('schedule_preparation')),
                  onPressed: () => setState(() {
                    plan['preparation'] = {
                      'date': plan['date'],
                      'time': '12:00',
                    };
                    dirty = true;
                  }),
                )
              else ...[
                DayButton(
                  'preparation_date',
                  object(plan['preparation'])['date'] as String,
                  (v) => setState(() {
                    object(plan['preparation'])['date'] = v;
                    dirty = true;
                  }),
                ),
                EditText(
                  'preparation_time',
                  object(plan['preparation'])['time'] as String,
                  (v) {
                    object(plan['preparation'])['time'] = v;
                    dirty = true;
                  },
                  required: true,
                ),
                TextButton(
                  onPressed: () => setState(() {
                    plan.remove('preparation');
                    dirty = true;
                  }),
                  child: Text(context.t('remove_preparation')),
                ),
              ],
              const SizedBox(height: 24),
              FilledButton(
                onPressed: busy || !widget.api.writable ? null : review,
                child: Text(context.t('review_plan')),
              ),
              if (widget.plan != null)
                TextButton(
                  onPressed: busy ? null : () => review(remove: true),
                  child: Text(context.t('remove_meal')),
                ),
              if (busy) const LinearProgressIndicator(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _allocation(Json a) => Padding(
    key: ObjectKey(a),
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: Column(
      children: [
        DayButton(
          'date',
          a['date'] as String,
          (v) => setState(() {
            a['date'] = v;
            dirty = true;
          }),
        ),
        Choice(
          'meal',
          a['meal'] as String,
          {for (final m in mealKeys) m: context.t(m)},
          (v) => setState(() {
            a['meal'] = v;
            dirty = true;
          }),
        ),
        EditText(
          'servings',
          '${a['servings']}',
          (v) {
            a['servings'] = int.tryParse(v) ?? 0;
            dirty = true;
          },
          required: true,
          integer: true,
        ),
        Align(
          alignment: Alignment.centerRight,
          child: IconButton(
            tooltip: context.t('delete'),
            icon: const Icon(Icons.delete_outline),
            onPressed: () => setState(() {
              (plan['allocations'] as List).remove(a);
              dirty = true;
            }),
          ),
        ),
        const Divider(),
      ],
    ),
  );
}

class MealReview extends StatefulWidget {
  const MealReview(
    this.api,
    this.data,
    this.plan,
    this.preview, {
    super.key,
    this.remove = false,
  });
  final HouseholdApi api;
  final PlanningData data;
  final Json plan, preview;
  final bool remove;
  @override
  State<MealReview> createState() => _MealReviewState();
}

class _MealReviewState extends State<MealReview> {
  bool checked = false, busy = false;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('review_plan'))),
    body: ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(
          widget.data.title(object(widget.plan['recipe'])),
          style: Theme.of(context).textTheme.titleLarge,
        ),
        Text(
          '${widget.plan['date']} · ${context.t(widget.plan['meal'] as String)}',
        ),
        Text(
          (widget.plan['attendees'] as List? ?? [])
              .map(widget.data.person)
              .join(', '),
        ),
        if (!widget.remove) ...[
          Note(
            context.t(
              widget.preview['feasible'] == true
                  ? 'plan_feasible'
                  : 'plan_infeasible',
            ),
            warning: widget.preview['feasible'] != true,
          ),
          for (final code in widget.preview['conflicts'] as List)
            ListTile(
              leading: const Icon(Icons.block),
              title: Text(context.t(code as String)),
            ),
          Note(context.t('dietary_review_note'), warning: true),
          NutritionView(object(widget.preview['nutrition'])),
        ],
        ShoppingDelta(objects(widget.preview['shopping_delta']), widget.data),
        CheckboxListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(context.t('reviewed_plan')),
          value: checked,
          onChanged: busy ? null : (v) => setState(() => checked = v == true),
        ),
        FilledButton(
          onPressed:
              !checked ||
                  busy ||
                  !widget.api.writable ||
                  (!widget.remove && widget.preview['feasible'] != true)
              ? null
              : () async {
                  setState(() => busy = true);
                  try {
                    await widget.api.household(
                      '/mealplan/apply',
                      method: 'POST',
                      body: {
                        'plan': widget.plan,
                        'expected_revision': widget.preview['plan_revision'],
                        'profile_revision': widget.preview['profile_revision'],
                        'reviewed': true,
                        'remove': widget.remove,
                      },
                    );
                    if (context.mounted) Navigator.pop(context, true);
                  } catch (e) {
                    if (context.mounted) await showFailure(context, e);
                  } finally {
                    if (mounted) setState(() => busy = false);
                  }
                },
          child: Text(
            context.t(widget.remove ? 'remove_meal' : 'apply_reviewed'),
          ),
        ),
      ],
    ),
  );
}
