import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_common.dart';
import 'recipes.dart';

class WeekProposalScreen extends StatefulWidget {
  const WeekProposalScreen(this.api, this.data, this.start, {super.key});
  final HouseholdApi api;
  final PlanningData data;
  final String start;
  @override
  State<WeekProposalScreen> createState() => _WeekProposalScreenState();
}

class _WeekProposalScreenState extends State<WeekProposalScreen> {
  final form = GlobalKey<FormState>();
  late String start = widget.start;
  late final selected = widget.data.recipes.length <= 100
      ? widget.data.recipes.map((r) => r['id'] as String).toSet()
      : <String>{};
  final meals = <String>{'dinner'};
  String active = '1800',
      elapsed = '3600',
      advance = '0',
      portions = '1',
      query = '';
  bool quick = true, reuse = true, busy = false;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('propose_week'))),
    body: Form(
      key: form,
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Note(context.t('week_proposal_note')),
          DayButton('week_of', start, (v) => setState(() => start = v)),
          for (final meal in mealKeys)
            CheckboxListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(context.t(meal)),
              value: meals.contains(meal),
              onChanged: busy
                  ? null
                  : (v) => setState(
                      () => v == true ? meals.add(meal) : meals.remove(meal),
                    ),
            ),
          EditText(
            'servings_per_person',
            portions,
            (v) => portions = v,
            required: true,
            integer: true,
          ),
          EditText(
            'active_seconds',
            active,
            (v) => active = v,
            required: true,
            integer: true,
          ),
          EditText(
            'elapsed_seconds',
            elapsed,
            (v) => elapsed = v,
            required: true,
            integer: true,
          ),
          EditText(
            'advance_seconds',
            advance,
            (v) => advance = v,
            required: true,
            integer: true,
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(context.t('prefer_quick')),
            value: quick,
            onChanged: busy ? null : (v) => setState(() => quick = v),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(context.t('prefer_reuse')),
            value: reuse,
            onChanged: busy ? null : (v) => setState(() => reuse = v),
          ),
          Text(
            '${context.t('candidate_recipes')} · ${selected.length}/100',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          if (widget.data.recipes.isEmpty) Note(context.t('no_recipes')),
          TextField(
            decoration: InputDecoration(
              labelText: context.t('browse'),
              prefixIcon: const Icon(Icons.search),
            ),
            onChanged: (v) => setState(() => query = v.toLowerCase()),
          ),
          for (final r in widget.data.recipes.where(
            (r) => (r['title'] as String).toLowerCase().contains(query),
          ))
            CheckboxListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(r['title'] as String),
              subtitle: Text('${context.t('revision')} ${r['revision']}'),
              value: selected.contains(r['id']),
              onChanged: busy
                  ? null
                  : (v) => setState(
                      () => v == true
                          ? selected.add(r['id'] as String)
                          : selected.remove(r['id']),
                    ),
            ),
          FilledButton(
            onPressed:
                busy ||
                    selected.isEmpty ||
                    selected.length > 100 ||
                    meals.isEmpty
                ? null
                : () async {
                    if (!form.currentState!.validate()) return;
                    setState(() => busy = true);
                    try {
                      final result = await widget.api.household(
                        '/mealplan/week/preview',
                        method: 'POST',
                        body: {
                          'start_date': start,
                          'meals': mealKeys.where(meals.contains).toList(),
                          'candidates': [
                            for (final r in widget.data.recipes.where(
                              (r) => selected.contains(r['id']),
                            ))
                              {'id': r['id'], 'revision': r['revision']},
                          ],
                          'active_seconds': int.parse(active),
                          'elapsed_seconds': int.parse(elapsed),
                          'advance_seconds': int.parse(advance),
                          'servings_per_person': int.parse(portions),
                          'prefer_quick': quick,
                          'prefer_reuse': reuse,
                        },
                      );
                      if (!context.mounted) return;
                      final applied = await openPage<bool>(
                        context,
                        WeekReview(widget.api, widget.data, result),
                      );
                      if (applied == true && context.mounted) {
                        Navigator.pop(context);
                      }
                    } catch (e) {
                      if (context.mounted) await showFailure(context, e);
                    } finally {
                      if (mounted) setState(() => busy = false);
                    }
                  },
            child: Text(context.t('generate_proposal')),
          ),
          if (busy) const LinearProgressIndicator(),
        ],
      ),
    ),
  );
}

class WeekReview extends StatefulWidget {
  const WeekReview(this.api, this.data, this.result, {super.key});
  final HouseholdApi api;
  final PlanningData data;
  final Json result;
  @override
  State<WeekReview> createState() => _WeekReviewState();
}

class _WeekReviewState extends State<WeekReview> {
  final checked = <int>{};
  bool busy = false;
  @override
  Widget build(BuildContext context) {
    final proposal = object(widget.result['proposal']);
    final slots = objects(proposal['slots']);
    final count = slots.where((s) => s['plan'] != null).length;
    return Scaffold(
      appBar: AppBar(title: Text(context.t('review_week'))),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(
            '$count / ${slots.length} ${context.t('proposed_meals')}',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          Note(context.t('week_ranking_note')),
          Text(
            '${context.t('expires')} ${MaterialLocalizations.of(context).formatTimeOfDay(TimeOfDay.fromDateTime(DateTime.fromMillisecondsSinceEpoch((proposal['expires_at'] as int) * 1000).toLocal()))}',
          ),
          for (var i = 0; i < slots.length; i++)
            Builder(
              builder: (context) {
                final s = slots[i];
                final p = s['plan'] as Json?;
                final evidence = object(s['evidence']);
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Padding(
                      padding: const EdgeInsets.only(top: 16),
                      child: Text(
                        '${s['date']} · ${context.t(s['meal'] as String)}',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                    ),
                    if (p == null) ...[
                      Note(
                        context.t(s['reason'] as String),
                        warning: s['reason'] == 'no_feasible_meal',
                      ),
                      for (final code in object(
                        evidence['rejected_by_constraint'] ??
                            <String, dynamic>{},
                      ).keys)
                        Text(context.t(code)),
                    ] else ...[
                      CheckboxListTile(
                        contentPadding: EdgeInsets.zero,
                        title: Text(widget.data.title(object(p['recipe']))),
                        subtitle: Text(
                          '${p['servings']} ${context.t('servings')} · ${(p['attendees'] as List).map(widget.data.person).join(', ')}',
                        ),
                        value: checked.contains(i),
                        onChanged: busy
                            ? null
                            : (v) => setState(
                                () => v == true
                                    ? checked.add(i)
                                    : checked.remove(i),
                              ),
                      ),
                      Text(
                        '${context.t('prior_week_uses')}: ${evidence['prior_week_uses']} · ${context.t('reused_ingredients')}: ${evidence['reused_ingredients']}',
                      ),
                      Text(
                        '${context.t('active_seconds')}: ${evidence['active_seconds']}',
                      ),
                      TextButton.icon(
                        icon: const Icon(Icons.menu_book_outlined),
                        label: Text(context.t('inspect_recipe')),
                        onPressed: () => openPage<void>(
                          context,
                          RecipeDetail(
                            widget.api,
                            object(p['recipe'])['id'] as String,
                            revision: object(p['recipe'])['revision'] as int,
                          ),
                        ),
                      ),
                    ],
                    const Divider(),
                  ],
                );
              },
            ),
          Note(context.t('dietary_review_note'), warning: true),
          ShoppingDelta(objects(widget.result['shopping_delta']), widget.data),
          FilledButton(
            onPressed:
                busy ||
                    count == 0 ||
                    checked.length != count ||
                    !widget.api.writable
                ? null
                : () async {
                    setState(() => busy = true);
                    try {
                      await widget.api.household(
                        '/mealplan/week/apply',
                        method: 'POST',
                        body: {
                          'proposal_id': proposal['id'],
                          'reviewed': true,
                          'confirmed_slots': [
                            for (var i = 0; i < slots.length; i++)
                              if (checked.contains(i))
                                {
                                  'date': slots[i]['date'],
                                  'meal': slots[i]['meal'],
                                },
                          ],
                        },
                      );
                      if (context.mounted) Navigator.pop(context, true);
                    } catch (e) {
                      if (context.mounted) await showFailure(context, e);
                    } finally {
                      if (mounted) setState(() => busy = false);
                    }
                  },
            child: Text(context.t('apply_week')),
          ),
        ],
      ),
    );
  }
}
