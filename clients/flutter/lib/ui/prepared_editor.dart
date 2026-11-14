import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_common.dart';
import 'stock_review.dart';

class PreparedEditor extends StatefulWidget {
  const PreparedEditor(
    this.api,
    this.data, {
    super.key,
    this.batch,
    this.source,
  });
  final HouseholdApi api;
  final PlanningData data;
  final Json? batch, source;
  @override
  State<PreparedEditor> createState() => _PreparedEditorState();
}

class _PreparedEditorState extends State<PreparedEditor> {
  final form = GlobalKey<FormState>();
  late Json? source = widget.source;
  late String? recipeId =
      widget.batch?['recipe_id'] as String? ??
      (source?['recipe'] as Json?)?['id'] as String? ??
      widget.data.recipes.firstOrNull?['id'] as String?;
  late int? revision =
      widget.batch?['revision'] as int? ??
      (source?['recipe'] as Json?)?['revision'] as int? ??
      widget.data.recipes.firstOrNull?['revision'] as int?;
  late String portions = '${source?['servings'] ?? 1}',
      produced = isoDay(DateTime.now());
  late Json storage = widget.batch == null
      ? {
          'location': '',
          'condition': 'other',
          'declared_on': isoDay(DateTime.now()),
          'use_by': null,
        }
      : clone(object(widget.batch!['storage']));
  bool dirty = false, busy = false, saved = false;
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(
            widget.batch == null ? 'record_prepared' : 'change_storage',
          ),
        ),
      ),
      body: Form(
        key: form,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Note(context.t('prepared_declaration_note'), warning: true),
            if (widget.batch == null) ...[
              Choice(
                'production_source',
                source == null
                    ? 'none'
                    : '${source!['date']}/${source!['meal']}',
                {
                  'none': context.t('unlinked_production'),
                  for (final p in widget.data.meals)
                    '${p['date']}/${p['meal']}':
                        '${p['date']} · ${context.t(p['meal'] as String)} · ${widget.data.title(object(p['recipe']))}',
                },
                (v) => setState(() {
                  source = widget.data.meals
                      .where((p) => '${p['date']}/${p['meal']}' == v)
                      .firstOrNull;
                  if (source != null) {
                    recipeId = object(source!['recipe'])['id'] as String;
                    revision = object(source!['recipe'])['revision'] as int;
                  }
                  dirty = true;
                }),
              ),
              if (source == null)
                Choice(
                  'recipe',
                  recipeId,
                  {
                    for (final r in widget.data.recipes)
                      r['id'] as String: r['title'] as String,
                  },
                  (id) => setState(() {
                    recipeId = id;
                    revision =
                        widget.data.recipes.firstWhere(
                              (r) => r['id'] == id,
                            )['revision']
                            as int;
                    dirty = true;
                  }),
                )
              else
                Text(widget.data.title(object(source!['recipe']))),
              EditText(
                'actual_yield',
                portions,
                (v) {
                  portions = v;
                  dirty = true;
                },
                required: true,
                integer: true,
              ),
              DayButton(
                'produced_on',
                produced,
                (v) => setState(() {
                  produced = v;
                  dirty = true;
                }),
              ),
            ],
            StorageFields(storage, () => setState(() => dirty = true)),
            FilledButton(
              onPressed: busy || !widget.api.writable
                  ? null
                  : () async {
                      if (!form.currentState!.validate()) return;
                      setState(() => busy = true);
                      try {
                        final command = widget.batch == null
                            ? <String, dynamic>{
                                'operation': 'record_prepared',
                                'input': {
                                  'recipe_id': recipeId,
                                  'revision': revision,
                                  'portions': int.parse(portions),
                                  'produced_on': produced,
                                  'storage': storage,
                                  'source': source == null
                                      ? null
                                      : {
                                          'date': source!['date'],
                                          'meal': source!['meal'],
                                        },
                                },
                              }
                            : <String, dynamic>{
                                'operation': 'store_prepared',
                                'batch_id': widget.batch!['id'],
                                'storage': storage,
                              };
                        if (await reviewStock(
                              context,
                              widget.api,
                              widget.data,
                              command,
                            ) &&
                            context.mounted) {
                          saved = true;
                          Navigator.pop(context);
                        }
                      } catch (e) {
                        if (context.mounted) await showFailure(context, e);
                      } finally {
                        if (mounted) setState(() => busy = false);
                      }
                    },
              child: Text(context.t('review_stock')),
            ),
          ],
        ),
      ),
    ),
  );
}

class StorageFields extends StatelessWidget {
  const StorageFields(this.storage, this.changed, {super.key});
  final Json storage;
  final VoidCallback changed;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      EditText(
        'storage_location',
        storage['location'] as String,
        (v) {
          storage['location'] = v;
          changed();
        },
        required: true,
        max: 100,
      ),
      Choice(
        'storage_condition',
        storage['condition'] as String,
        {
          for (final c in ['ambient', 'chilled', 'frozen', 'other'])
            c: context.t('storage_$c'),
        },
        (v) {
          storage['condition'] = v;
          changed();
        },
      ),
      DayButton('storage_declared_on', storage['declared_on'] as String, (v) {
        storage['declared_on'] = v;
        changed();
      }),
      if (storage['use_by'] == null)
        TextButton.icon(
          icon: const Icon(Icons.add),
          label: Text(context.t('declare_use_by')),
          onPressed: () {
            storage['use_by'] = storage['declared_on'];
            changed();
          },
        )
      else ...[
        DayButton('declared_use_by', storage['use_by'] as String, (v) {
          storage['use_by'] = v;
          changed();
        }),
        TextButton(
          onPressed: () {
            storage['use_by'] = null;
            changed();
          },
          child: Text(context.t('remove_use_by')),
        ),
      ],
    ],
  );
}

class PreparedAllocationEditor extends StatefulWidget {
  const PreparedAllocationEditor(this.api, this.data, this.batch, {super.key});
  final HouseholdApi api;
  final PlanningData data;
  final Json batch;
  @override
  State<PreparedAllocationEditor> createState() =>
      _PreparedAllocationEditorState();
}

class _PreparedAllocationEditorState extends State<PreparedAllocationEditor> {
  final form = GlobalKey<FormState>();
  String date = isoDay(DateTime.now()), meal = 'dinner', portions = '1';
  late List<String> people = widget.data.attendees(date, meal);
  bool dirty = false, busy = false, saved = false;
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('allocate_portions'))),
      body: Form(
        key: form,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(
              widget.batch['title'] as String,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            DayButton(
              'date',
              date,
              (v) => setState(() {
                date = v;
                dirty = true;
              }),
            ),
            Choice(
              'meal',
              meal,
              {for (final m in mealKeys) m: context.t(m)},
              (v) => setState(() {
                meal = v;
                dirty = true;
              }),
            ),
            EditText(
              'servings',
              portions,
              (v) {
                portions = v;
                dirty = true;
              },
              required: true,
              integer: true,
            ),
            AttendeeChoices(
              widget.data,
              people,
              (v) => setState(() {
                people = v;
                dirty = true;
              }),
            ),
            Note(context.t('physical_allocation_note')),
            FilledButton(
              onPressed: busy
                  ? null
                  : () async {
                      if (!form.currentState!.validate()) return;
                      setState(() => busy = true);
                      try {
                        if (await reviewStock(
                              context,
                              widget.api,
                              widget.data,
                              {
                                'operation': 'allocate_prepared',
                                'batch_id': widget.batch['id'],
                                'date': date,
                                'meal': meal,
                                'portions': int.parse(portions),
                                'member_ids': people,
                              },
                            ) &&
                            context.mounted) {
                          saved = true;
                          Navigator.pop(context);
                        }
                      } catch (e) {
                        if (context.mounted) await showFailure(context, e);
                      } finally {
                        if (mounted) setState(() => busy = false);
                      }
                    },
              child: Text(context.t('review_stock')),
            ),
          ],
        ),
      ),
    ),
  );
}
