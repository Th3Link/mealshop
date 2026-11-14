import 'package:material_ui/material_ui.dart';

import '../content/api.dart';
import '../connection/nextcloud.dart';
import '../planning/data.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_common.dart';
import 'meal_editor.dart';
import 'week_proposal.dart';
import 'attendance.dart';
import 'stock.dart';
import 'plan_shopping.dart';
import 'recipes.dart';
import 'prepared_editor.dart';

class PlanScreen extends StatefulWidget {
  const PlanScreen(
    this.api, {
    super.key,
    this.today = false,
    this.refresh = 0,
    required this.shopping,
  });
  final HouseholdApi api;
  final bool today;
  final int refresh;
  final VoidCallback shopping;
  @override
  State<PlanScreen> createState() => _PlanScreenState();
}

class _PlanScreenState extends State<PlanScreen> {
  DateTime selectedDay = civilDay(isoDay(DateTime.now()));
  DateTime get day =>
      widget.today ? civilDay(isoDay(DateTime.now())) : selectedDay;
  set day(DateTime value) => selectedDay = value;
  @override
  Widget build(BuildContext context) => Resource<PlanningData>(
    key: ValueKey(widget.refresh),
    sensitive: true,
    pollInterval: null,
    load: () => PlanningData.load(widget.api),
    builder: (context, data, reload) => ListView(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 32),
      children: [
        if (!widget.today) ...[
          Row(
            children: [
              IconButton(
                tooltip: context.t('previous_week'),
                icon: const Icon(Icons.chevron_left),
                onPressed: () => setState(() => day = addDays(day, -7)),
              ),
              Expanded(
                child: DayButton(
                  'week_of',
                  isoDay(weekStart(day)),
                  (v) => setState(() => day = civilDay(v)),
                ),
              ),
              IconButton(
                tooltip: context.t('next_week'),
                icon: const Icon(Icons.chevron_right),
                onPressed: () => setState(() => day = addDays(day, 7)),
              ),
            ],
          ),
          LayoutBuilder(
            builder: (context, constraints) {
              final large = MediaQuery.textScalerOf(context).scale(14) > 20;
              final width = ((constraints.maxWidth - 24) / 7).clamp(
                large ? 64.0 : 48.0,
                120.0,
              );
              return Wrap(
                spacing: 4,
                runSpacing: 4,
                children: [
                  for (var i = 0; i < 7; i++)
                    Builder(
                      builder: (context) {
                        final date = addDays(weekStart(day), i),
                            selected = isoDay(date) == isoDay(day);
                        return SizedBox(
                          width: width,
                          child: ChoiceChip(
                            showCheckmark: false,
                            labelPadding: const EdgeInsets.symmetric(
                              horizontal: 2,
                            ),
                            label: SizedBox(
                              width: double.infinity,
                              child: Column(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  Text(context.t('weekday_$i')),
                                  Text(
                                    '${date.day}',
                                    style: TextStyle(
                                      fontWeight: selected
                                          ? FontWeight.bold
                                          : FontWeight.normal,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                            selected: selected,
                            tooltip: MaterialLocalizations.of(context)
                                .formatFullDate(date),
                            onSelected: (_) => setState(() => day = date),
                          ),
                        );
                      },
                    ),
                ],
              );
            },
          ),
        ],
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 16),
          child: Text(
            MaterialLocalizations.of(context).formatFullDate(day),
            style: Theme.of(context).textTheme.titleLarge,
          ),
        ),
        if (!widget.api.writable) Note(context.t('household_read_only')),
        ..._preparations(context, data, reload),
        for (final meal in mealKeys) _meal(context, data, meal, reload),
        const SizedBox(height: 16),
        if (!widget.today && widget.api.writable)
          FilledButton.icon(
            icon: const Icon(Icons.auto_awesome_outlined),
            label: Text(context.t('propose_week')),
            onPressed: () async {
              await openPage<void>(
                context,
                WeekProposalScreen(widget.api, data, isoDay(weekStart(day))),
              );
              await reload();
            },
          ),
        const SizedBox(height: 8),
        OutlinedButton.icon(
          icon: const Icon(Icons.shopping_basket_outlined),
          label: Text(context.t('shopping')),
          onPressed: widget.shopping,
        ),
        if (widget.api.writable && data.meals.isNotEmpty)
          OutlinedButton.icon(
            icon: const Icon(Icons.playlist_add_check),
            label: Text(context.t('update_plan_shopping')),
            onPressed: () async {
              await openPage<void>(
                context,
                PlanShoppingScreen(
                  widget.api,
                  data,
                  start: isoDay(weekStart(day)),
                ),
              );
              await reload();
            },
          ),
        OutlinedButton.icon(
          icon: const Icon(Icons.kitchen_outlined),
          label: Text(context.t('stock')),
          onPressed: () async {
            await openPage<void>(context, StockScreen(widget.api));
            await reload();
          },
        ),
        if (!widget.today && widget.api.writable)
          TextButton.icon(
            icon: const Icon(Icons.people_outline),
            label: Text(context.t('attendance_schedule')),
            onPressed: () async {
              await openPage<void>(context, AttendanceScreen(widget.api, data));
              await reload();
            },
          ),
      ],
    ),
  );
  Widget _meal(
    BuildContext context,
    PlanningData data,
    String meal,
    Future<void> Function() reload,
  ) {
    final plans = data.meals
        .where(
          (p) =>
              eatingSlots(p)
                  .any((s) => s['date'] == isoDay(day) && s['meal'] == meal),
        )
        .toList();
    final batches = data.prepared
        .where((r) => object(r['batch'])['reversed'] != true)
        .where(
          (r) => objects(object(r['batch'])['allocations']).any(
            (a) =>
                a['date'] == isoDay(day) &&
                a['meal'] == meal &&
                a['state'] != 'released',
          ),
        )
        .toList();
    if (plans.isEmpty && batches.isEmpty) {
      return Column(
        children: [
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(context.t(meal)),
            subtitle: Text(context.t('no_meal_planned')),
            trailing: widget.api.writable ? const Icon(Icons.add) : null,
            onTap: !widget.api.writable
                ? null
                : () async {
                    await openPage<void>(
                      context,
                      MealEditor(
                        widget.api,
                        data,
                        date: isoDay(day),
                        meal: meal,
                      ),
                    );
                    await reload();
                  },
          ),
          const Divider(),
        ],
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 16, bottom: 4),
          child: Text(
            context.t(meal),
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        for (final p in plans)
          ListTile(
            contentPadding: EdgeInsets.zero,
            leading: const Icon(Icons.restaurant_outlined),
            title: Text(data.title(object(p['recipe']))),
            subtitle: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '${eatingSlots(p).firstWhere((s) => s['date'] == isoDay(day) && s['meal'] == meal)['servings']} ${context.t('servings')}\n${(p['attendees'] as List? ?? []).map(data.person).join(', ')}',
                ),
                if (objects(
                  data.plan['review_required_slots'],
                ).any((s) => s['date'] == p['date'] && s['meal'] == p['meal']))
                  Note(context.t('review_required'), warning: true),
                for (final row in batches.where((r) => linked(r, p)))
                  Text(
                    '${context.t('physical_prepared')}: ${objects(object(row['batch'])['allocations']).where((a) => a['date'] == isoDay(day) && a['meal'] == meal && a['state'] != 'released').map((a) => '${a['portions']} ${context.t('servings')} · ${context.t(a['state'] as String)}').join(', ')}',
                  ),
              ],
            ),
            trailing: !widget.api.writable
                ? const Icon(Icons.chevron_right)
                : PopupMenuButton<String>(
                    tooltip: context.t('meal_actions'),
                    itemBuilder: (context) => [
                      PopupMenuItem(
                        value: 'edit',
                        child: Text(context.t('edit_meal')),
                      ),
                      PopupMenuItem(
                        value: 'record',
                        child: Text(context.t('record_prepared')),
                      ),
                    ],
                    onSelected: (action) async {
                      await openPage<void>(
                        context,
                        action == 'record'
                            ? PreparedEditor(widget.api, data, source: p)
                            : MealEditor(widget.api, data, plan: p),
                      );
                      await reload();
                    },
                  ),
            onTap: () async {
              if (widget.api.writable) {
                await openPage<void>(
                  context,
                  MealEditor(widget.api, data, plan: p),
                );
              } else {
                await openPage<void>(
                  context,
                  RecipeDetail(
                    widget.api,
                    object(p['recipe'])['id'] as String,
                    revision: object(p['recipe'])['revision'] as int,
                  ),
                );
              }
              await reload();
            },
          ),
        for (final r in batches.where((r) => !plans.any((p) => linked(r, p))))
          ListTile(
            contentPadding: EdgeInsets.zero,
            leading: const Icon(Icons.inventory_2_outlined),
            title: Text(object(r['batch'])['title'] as String),
            subtitle: Text(
              '${context.t('physical_prepared')} · ${objects(object(r['batch'])['allocations']).where((a) => a['date'] == isoDay(day) && a['meal'] == meal && a['state'] != 'released').map((a) => '${a['portions']} ${context.t('servings')} · ${context.t(a['state'] as String)}').join('\n')}',
            ),
            onTap: () async {
              await openPage<void>(context, StockScreen(widget.api));
              await reload();
            },
          ),
        const Divider(),
      ],
    );
  }

  bool linked(Json row, Json plan) {
    final source = object(
      object(row['batch'])['source'] ?? <String, dynamic>{},
    );
    return source['date'] == plan['date'] && source['meal'] == plan['meal'];
  }

  List<Widget> _preparations(
    BuildContext context,
    PlanningData data,
    Future<void> Function() reload,
  ) => [
    for (final p in data.meals.where(
      (p) => (p['preparation'] as Json?)?['date'] == isoDay(day),
    ))
      ListTile(
        contentPadding: EdgeInsets.zero,
        leading: const Icon(Icons.schedule),
        title: Text(
          '${object(p['preparation'])['time']} · ${data.title(object(p['recipe']))}',
        ),
        subtitle: Text(
          '${context.t('planned_preparation')} · ${p['servings']} ${context.t('servings')}',
        ),
        onTap: () async {
          await openPage<void>(
            context,
            RecipeDetail(
              widget.api,
              object(p['recipe'])['id'] as String,
              revision: object(p['recipe'])['revision'] as int,
            ),
          );
          await reload();
        },
      ),
  ];
}
