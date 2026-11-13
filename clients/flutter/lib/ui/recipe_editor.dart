import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'ingredient_picker.dart';
import 'messages.dart';
import 'photo.dart';

Future<List<String>?> chooseReferences(
  BuildContext context,
  String label,
  Map<String, String> options,
  List<String> selected,
) => showModalBottomSheet<List<String>>(
  context: context,
  isScrollControlled: true,
  useSafeArea: true,
  showDragHandle: true,
  builder: (context) {
    final values = [...selected];
    return StatefulBuilder(
      builder: (context, set) => DraggableScrollableSheet(
        expand: false,
        initialChildSize: .65,
        builder: (context, scroll) => Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                context.t(label),
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ),
            Expanded(
              child: ListView(
                controller: scroll,
                children: [
                  for (final e in options.entries)
                    CheckboxListTile(
                      title: Text(e.value),
                      value: values.contains(e.key),
                      onChanged: (v) => set(() {
                        v == true ? values.add(e.key) : values.remove(e.key);
                      }),
                    ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(16),
              child: FilledButton(
                onPressed: () => Navigator.pop(context, values),
                child: Text(context.t('done')),
              ),
            ),
          ],
        ),
      ),
    );
  },
);

class RecipeEditor extends StatefulWidget {
  const RecipeEditor(this.api, {super.key, this.record});
  final HouseholdApi api;
  final Json? record;
  @override
  State<RecipeEditor> createState() => _RecipeEditorState();
}

class _RecipeEditorState extends State<RecipeEditor> {
  String? failure;
  final form = GlobalKey<FormState>();
  late Json recipe;
  Catalogue? catalogue;
  List<Json> conversions = [];
  int? expectedCount;
  bool dirty = false, busy = false;
  final Set<String> expanded = {};
  String get language => widget.api.controller.language;
  List<Json> get parts => objects(recipe['parts'] ?? []);
  List<Json> get ingredients => objects(recipe['lines']);
  List<Json> get steps => objects(recipe['steps']);
  @override
  void initState() {
    super.initState();
    recipe = widget.record == null
        ? {
            'title': '',
            'language': language,
            'author': '',
            'servings': 1,
            'parts': <Json>[],
            'lines': <Json>[],
            'steps': <Json>[],
            'preparation': 'as_recorded',
          }
        : clone(object(widget.record!['recipe']));
    for (final line in ingredients) {
      line['id'] ??= newId('line-');
      if (line['quantity'] != null) {
        final quantity = object(line['quantity']);
        quantity['amount'] = inputDecimal(
          quantity['amount'] as String,
          language,
        );
      } else {
        line['grams'] = inputDecimal(line['grams'] as String, language);
      }
      for (final extension in objects(line['extensions'] ?? [])) {
        extension['value'] = inputDecimal(
          extension['value'] as String,
          language,
        );
      }
    }
    unawaited(load());
  }

  Future<void> load() async {
    try {
      if (widget.record == null && expectedCount == null) {
        expectedCount = objects((await widget.api.recipes())['recipes']).length;
      }
      final cat = Catalogue(await widget.api.household('/ingredients'));
      final value = await widget.api.call('/api/conversions');
      if (mounted) {
        setState(() {
          catalogue = cat;
          conversions = objects(value['entries']);
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => failure = e is ApiFailure ? e.code : 'request_failed');
      }
    }
  }

  void change(VoidCallback action) => setState(() {
    action();
    dirty = true;
  });
  String name(String id) => catalogue?.find(id) == null
      ? id
      : catalogue!.name(catalogue!.find(id)!, language);
  Map<String, String> get partOptions => {
    '': context.t('whole_recipe'),
    for (final p in parts) p['id'] as String: p['name'] as String,
  };
  Future<void> addIngredient() async {
    final item = await pickIngredient(context, widget.api, create: true);
    if (item == null || !mounted) return;
    await load();
    change(
      () => (recipe['lines'] as List).add({
        'id': newId('line-'),
        'ingredient_id': item['id'],
        'quantity': {'amount': '', 'unit': 'g'},
      }),
    );
  }

  Future<void> save() async {
    if (!form.currentState!.validate()) return;
    setState(() => busy = true);
    try {
      final result = clone(recipe);
      final count = result['servings'];
      if (count is! int || count < 1) {
        throw const ApiFailure('invalid_recipe');
      }
      for (final line in objects(result['lines'])) {
        if (line['quantity'] != null) {
          final q = object(line['quantity']);
          q['amount'] = exactInput(q['amount'] as String, language);
          line.remove('grams');
        } else {
          line['grams'] = exactInput(line['grams'] as String, language);
        }
        for (final extension in objects(line['extensions'] ?? [])) {
          extension['value'] = exactInput(
            extension['value'] as String,
            language,
            signed: true,
          );
        }
      }
      final value = await widget.api.recipes(
        widget.record == null
            ? '?expected_count=$expectedCount'
            : '/${widget.record!['id']}',
        widget.record == null ? 'POST' : 'PUT',
        widget.record == null
            ? result
            : {
                'expected_revision': widget.record!['revision'],
                'recipe': result,
              },
      );
      dirty = false;
      if (mounted) Navigator.pop(context, value['id'] as String);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Widget references(
    String label,
    Json record,
    String field,
    Map<String, String> choices,
  ) => ListTile(
    contentPadding: EdgeInsets.zero,
    title: Text(context.t(label)),
    subtitle: Text(
      List<String>.from(record[field] as List? ?? [])
          .map((id) => choices[id] ?? id)
          .join(', '),
    ),
    trailing: const Icon(Icons.add),
    onTap: () async {
      final selected = await chooseReferences(
        context,
        label,
        choices,
        List<String>.from(record[field] as List? ?? []),
      );
      if (selected != null) change(() => record[field] = selected);
    },
  );
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty,
    child: Scaffold(
      appBar: AppBar(
        title: Text(context.t(widget.record == null ? 'new_recipe' : 'edit')),
      ),
      body: catalogue == null
          ? failure == null
                ? const Center(child: CircularProgressIndicator())
                : EmptyContent(
                    icon: Icons.cloud_off_outlined,
                    title: failure!,
                    action: 'retry',
                    onAction: load,
                  )
          : Form(
              key: form,
              child: ListView(
                padding: const EdgeInsets.fromLTRB(20, 8, 20, 30),
                children: [
                  EditText(
                    'title',
                    recipe['title'] as String,
                    (v) => change(() => recipe['title'] = v),
                    required: true,
                  ),
                  EditText(
                    'author',
                    recipe['author'] as String? ?? '',
                    (v) => change(() => recipe['author'] = v),
                  ),
                  Choice('source_language', recipe['language'] as String, {
                    'en': 'English',
                    'de': 'Deutsch',
                  }, (v) => change(() => recipe['language'] = v)),
                  EditText(
                    'servings',
                    '${recipe['servings']}',
                    (v) => change(() => recipe['servings'] = int.tryParse(v)),
                    required: true,
                    numeric: true,
                    integer: true,
                  ),
                  PhotoField(
                    widget.api,
                    recipe['image_id'] as String?,
                    (id) => change(() => recipe['image_id'] = id),
                  ),
                  if (object(recipe['origin'] ?? <String, dynamic>{})['kind'] ==
                      'imported')
                    Note(context.t('source_only')),
                  SectionTitle(context.t('parts')),
                  for (final part in parts)
                    Column(
                      key: ValueKey(part['id']),
                      children: [
                        EditText(
                          'name',
                          part['name'] as String,
                          (v) => change(() => part['name'] = v),
                          required: true,
                        ),
                        references('uses_parts', part, 'uses_parts', {
                          for (final p in parts.where(
                            (p) => p['id'] != part['id'],
                          ))
                            p['id'] as String: p['name'] as String,
                        }),
                        references('starts_after', part, 'starts_after', {
                          for (final p in parts.where(
                            (p) => p['id'] != part['id'],
                          ))
                            p['id'] as String: p['name'] as String,
                        }),
                        Align(
                          alignment: Alignment.centerRight,
                          child: IconButton(
                            tooltip: context.t('delete'),
                            icon: const Icon(Icons.delete_outline),
                            onPressed: () async {
                              final id = part['id'];
                              final used =
                                  ingredients.any(
                                    (l) =>
                                        l['part_id'] == id ||
                                        l['prepared_part'] == id,
                                  ) ||
                                  steps.any(
                                    (s) =>
                                        s['part_id'] == id ||
                                        (s['prepared_parts'] as List? ?? [])
                                            .contains(id),
                                  ) ||
                                  parts.any(
                                    (p) =>
                                        (p['uses_parts'] as List? ?? [])
                                            .contains(id) ||
                                        (p['starts_after'] as List? ?? [])
                                            .contains(id),
                                  );
                              if (used) {
                                await showFailure(
                                  context,
                                  const ApiFailure('part_in_use'),
                                );
                                return;
                              }
                              change(
                                () => (recipe['parts'] as List).remove(part),
                              );
                            },
                          ),
                        ),
                        const Divider(),
                      ],
                    ),
                  OutlinedButton.icon(
                    onPressed: parts.length >= 30
                        ? null
                        : () => change(() {
                            recipe['parts'] ??= <Json>[];
                            (recipe['parts'] as List).add({
                              'id': newId('part-'),
                              'name': '',
                              'uses_parts': <String>[],
                              'starts_after': <String>[],
                            });
                          }),
                    icon: const Icon(Icons.add),
                    label: Text(context.t('new_part')),
                  ),
                  SectionTitle(context.t('ingredients')),
                  for (final line in ingredients) ingredientRow(line),
                  OutlinedButton.icon(
                    onPressed: ingredients.length >= 100 ? null : addIngredient,
                    icon: const Icon(Icons.add),
                    label: Text(context.t('find_ingredient')),
                  ),
                  SectionTitle(context.t('steps')),
                  for (var i = 0; i < steps.length; i++) stepRow(steps[i], i),
                  OutlinedButton.icon(
                    onPressed: steps.length >= 100
                        ? null
                        : () => change(
                            () => (recipe['steps'] as List).add({
                              'instruction': '',
                              'active_seconds': 0,
                              'passive_seconds': 0,
                              'ingredient_line_ids': <String>[],
                              'prepared_parts': <String>[],
                            }),
                          ),
                    icon: const Icon(Icons.add),
                    label: Text(context.t('new_step')),
                  ),
                  const SizedBox(height: 24),
                  FilledButton(
                    onPressed: busy || ingredients.isEmpty ? null : save,
                    child: Text(context.t('save')),
                  ),
                ],
              ),
            ),
    ),
  );
  Widget ingredientRow(Json line) {
    final id = line['id'] as String;
    final q = line['quantity'] == null ? null : object(line['quantity']);
    final unit = q?['unit'] as String? ?? 'g';
    final options = conversions
        .where(
          (c) =>
              object(c['evidence'])['ingredient_id'] == line['ingredient_id'],
        )
        .toList();
    return Column(
      key: ValueKey(id),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(
              child: TextButton.icon(
                style: TextButton.styleFrom(
                  alignment: Alignment.centerLeft,
                  padding: const EdgeInsets.symmetric(vertical: 8),
                ),
                onPressed: () async {
                  final item = await pickIngredient(
                    context,
                    widget.api,
                    create: true,
                  );
                  if (item != null) {
                    await load();
                    change(() {
                      line['ingredient_id'] = item['id'];
                      line['quantity'] = {'amount': '', 'unit': 'g'};
                      line.remove('conversion');
                      line.remove('grams');
                    });
                  }
                },
                icon: const Icon(Icons.eco_outlined),
                label: Text(name(line['ingredient_id'] as String)),
              ),
            ),
            IconButton(
              tooltip: context.t('delete'),
              icon: const Icon(Icons.delete_outline),
              onPressed: () => change(() {
                (recipe['lines'] as List).remove(line);
                for (final step in steps) {
                  (step['ingredient_line_ids'] as List?)?.remove(id);
                }
              }),
            ),
          ],
        ),
        LayoutBuilder(
          builder: (context, box) {
            final amount = EditText(
              'quantity',
              q?['amount'] as String? ?? line['grams'] as String,
              (v) => change(() {
                line['quantity'] ??= {'amount': line['grams'], 'unit': 'g'};
                object(line['quantity'])['amount'] = v;
              }),
              numeric: true,
              required: true,
            );
            final units = Choice(
              'unit',
              unit,
              {
                for (final u in {
                  'g',
                  'kg',
                  ...options.map((o) => o['unit'] as String),
                  unit,
                })
                  u: u,
              },
              (v) => change(() {
                line['quantity'] = {
                  'amount': q?['amount'] ?? line['grams'],
                  'unit': v,
                };
                line.remove('grams');
                line.remove('conversion');
              }),
            );
            return box.maxWidth < 340
                ? Column(children: [amount, units])
                : Row(
                    children: [
                      Expanded(flex: 2, child: amount),
                      const SizedBox(width: 12),
                      Expanded(child: units),
                    ],
                  );
          },
        ),
        if (!['g', 'kg'].contains(unit)) ...[
          Choice(
            'source',
            null,
            {
              for (final option in options.where((c) => c['unit'] == unit))
                option['id'] as String: option['label'] as String,
            },
            (v) => change(
              () => line['conversion'] = clone(
                object(options.firstWhere((c) => c['id'] == v)['evidence']),
              ),
            ),
          ),
          if (line['conversion'] != null)
            Note(
              '${object(line['conversion'])['form']} · ${object(line['conversion'])['version']}\n${object(line['conversion'])['uncertainty']}',
            ),
        ],
        if (parts.isNotEmpty) ...[
          Choice(
            'part',
            line['part_id'] as String? ?? '',
            partOptions,
            (v) => change(() => line['part_id'] = v.isEmpty ? null : v),
          ),
          Choice('homemade', line['prepared_part'] as String? ?? '', {
            '': context.t('purchased_input'),
            for (final p in parts) p['id'] as String: p['name'] as String,
          }, (v) => change(() => line['prepared_part'] = v.isEmpty ? null : v)),
        ],
        if (expanded.contains(id) ||
            objects(line['extensions'] ?? []).isNotEmpty) ...[
          for (final ext in objects(line['extensions'] ?? []))
            Row(
              children: [
                Expanded(
                  child: EditText(
                    'temperature',
                    ext['value'] as String,
                    (v) => change(() {
                      ext['value'] = v;
                      ext['original_input'] = v;
                    }),
                    numeric: true,
                    signed: true,
                  ),
                ),
                IconButton(
                  tooltip: context.t('delete'),
                  onPressed: () =>
                      change(() => (line['extensions'] as List).remove(ext)),
                  icon: const Icon(Icons.delete_outline),
                ),
              ],
            ),
        ],
        if (objects(line['extensions'] ?? []).isEmpty)
          Align(
            alignment: Alignment.centerRight,
            child: IconButton(
              tooltip: context.t('extra'),
              icon: const Icon(Icons.add),
              onPressed: () => change(() {
                expanded.add(id);
                line['extensions'] = [
                  {
                    'type': 'temperature',
                    'value': '',
                    'unit': 'celsius',
                    'original_input': '',
                    'provenance': 'recipe_author',
                  },
                ];
              }),
            ),
          ),
        const Divider(),
      ],
    );
  }

  Widget stepRow(Json step, int index) => Column(
    key: ObjectKey(step),
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Row(
        children: [
          Expanded(
            child: Text(
              '${index + 1}',
              style: Theme.of(context).textTheme.titleLarge,
            ),
          ),
          IconButton(
            tooltip: context.t('move_up'),
            onPressed: index == 0
                ? null
                : () => change(() {
                    final list = recipe['steps'] as List;
                    list.removeAt(index);
                    list.insert(index - 1, step);
                  }),
            icon: const Icon(Icons.arrow_upward),
          ),
          IconButton(
            tooltip: context.t('move_down'),
            onPressed: index == steps.length - 1
                ? null
                : () => change(() {
                    final list = recipe['steps'] as List;
                    list.removeAt(index);
                    list.insert(index + 1, step);
                  }),
            icon: const Icon(Icons.arrow_downward),
          ),
          IconButton(
            tooltip: context.t('delete'),
            onPressed: () =>
                change(() => (recipe['steps'] as List).remove(step)),
            icon: const Icon(Icons.delete_outline),
          ),
        ],
      ),
      if (parts.isNotEmpty)
        Choice(
          'part',
          step['part_id'] as String? ?? '',
          partOptions,
          (v) => change(() => step['part_id'] = v.isEmpty ? null : v),
        ),
      EditText(
        'instruction',
        step['instruction'] as String,
        (v) => change(() => step['instruction'] = v),
        required: true,
        lines: 3,
        max: 4000,
      ),
      references('uses_lines', step, 'ingredient_line_ids', {
        for (final l in ingredients)
          l['id'] as String: name(l['ingredient_id'] as String),
      }),
      if (parts.isNotEmpty)
        references('uses_parts', step, 'prepared_parts', {
          for (final p in parts) p['id'] as String: p['name'] as String,
        }),
      for (final key in [
        'active_seconds',
        'passive_seconds',
        'passive_max_seconds',
      ])
        EditText(
          key,
          step[key]?.toString() ?? '',
          (v) => change(
            () => step[key] = v.isEmpty && key == 'passive_max_seconds'
                ? null
                : int.tryParse(v),
          ),
          numeric: true,
          integer: true,
        ),
      const Divider(),
    ],
  );
}
