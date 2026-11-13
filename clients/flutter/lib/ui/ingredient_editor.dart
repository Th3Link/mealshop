import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'ingredient_picker.dart';
import 'messages.dart';

class IngredientEditor extends StatefulWidget {
  const IngredientEditor(this.api, {super.key, this.id, this.initialName = ''});
  final HouseholdApi api;
  final String? id;
  final String initialName;
  @override
  State<IngredientEditor> createState() => _IngredientEditorState();
}

class _IngredientEditorState extends State<IngredientEditor> {
  String? failure;
  final form = GlobalKey<FormState>();
  Catalogue? catalogue;
  Json? original;
  late Json annotation;
  String name = '',
      description = '',
      url = '',
      date = DateTime.now().toIso8601String().substring(0, 10),
      language = 'en';
  bool dirty = false, busy = false;
  Json get organization => object(annotation['organization']);
  String get locale => widget.api.controller.language;
  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  Future<void> load() async {
    try {
      final data = Catalogue(await widget.api.household('/ingredients'));
      original = widget.id == null ? null : data.find(widget.id!);
      if (widget.id != null && original == null) {
        throw const ApiFailure('invalid_ingredient');
      }
      annotation = original == null
          ? {
              'organization': {
                'is_group': false,
                'group_id': null,
                'diet': 'unknown',
                'origin': 'unknown',
                'production': 'unknown',
                'nutrition': {'mode': 'reference'},
                'intolerance_triggers': <String>[],
              },
              'tags': <String>[],
              'components': <String>[],
              'declared_allergens': <String>[],
              'archived': false,
              'favorite': false,
            }
          : clone({
              for (final k in [
                'organization',
                'tags',
                'components',
                'declared_allergens',
                'archived',
                'favorite',
              ])
                k: original![k],
            });
      name = original == null
          ? widget.initialName
          : data.name(original!, locale);
      language = locale;
      if (mounted) setState(() => catalogue = data);
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
  Future<void> save() async {
    if (!form.currentState!.validate()) return;
    setState(() => busy = true);
    try {
      final result = clone(annotation);
      final nutrition = object(object(result['organization'])['nutrition']);
      if (nutrition['mode'] == 'own') {
        for (final reading in object(nutrition['values']).values) {
          final r = object(reading);
          final amount = r['amount'] as String?;
          r['amount'] = amount == null || amount.isEmpty
              ? null
              : exactInput(amount, locale);
          if (r['range'] != null) {
            final range = object(r['range']);
            range['lower'] = exactInput(range['lower'] as String, locale);
            range['upper'] = exactInput(range['upper'] as String, locale);
          }
          r['uncertainty'] = (r['uncertainty'] as String? ?? '').trim().isEmpty
              ? nutrition['note']
              : r['uncertainty'];
        }
        for (final declaration in object(
          nutrition['constituents'] ?? <String, dynamic>{},
        ).values) {
          final d = object(declaration);
          final value = d['amount'] as String?;
          d['amount'] = value == null || value.isEmpty
              ? null
              : exactInput(value, locale);
        }
      }
      final response = await widget.api.household(
        '/ingredients${widget.id == null ? '' : '/${widget.id}'}',
        method: widget.id == null ? 'POST' : 'PUT',
        body: {
          'expected_revision': catalogue!.revision,
          if (widget.id == null)
            'ingredient': {
              'name': name.trim(),
              'language': language,
              'form': description.trim(),
              'source_url': url.trim(),
              'recorded_on': date,
              'annotations': result,
            }
          else
            'annotations': result,
        },
      );
      dirty = false;
      if (mounted) Navigator.pop(context, widget.id ?? response['id']);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  String itemName(String id) => catalogue!.find(id) == null
      ? id
      : catalogue!.name(catalogue!.find(id)!, locale);
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: dirty,
    child: Scaffold(
      appBar: AppBar(
        title: Text(context.t(widget.id == null ? 'new_ingredient' : 'edit')),
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
                padding: const EdgeInsets.all(20),
                children: [
                  if (original == null) ...[
                    EditText(
                      'name',
                      name,
                      (v) => change(() => name = v),
                      required: true,
                    ),
                    EditText(
                      'form',
                      description,
                      (v) => change(() => description = v),
                      required: true,
                      max: 500,
                    ),
                    Choice('source_language', language, {
                      'en': 'English',
                      'de': 'Deutsch',
                    }, (v) => change(() => language = v)),
                    EditText(
                      'source_url',
                      url,
                      (v) => change(() => url = v),
                      max: 2000,
                    ),
                    EditText(
                      'recorded_on',
                      date,
                      (v) => change(() => date = v),
                      required: true,
                    ),
                  ] else ...[
                    Text(
                      name,
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                    Text(original!['form'] as String),
                    Note(
                      '${context.t('source')}: ${object(original!['source'])['name']} · ${object(original!['source'])['version']}',
                    ),
                  ],
                  SectionTitle(context.t('ingredient_group')),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('is_group')),
                    value: organization['is_group'] as bool? ?? false,
                    onChanged: (v) => change(() {
                      organization['is_group'] = v;
                      if (v) {
                        organization['group_id'] = null;
                        if (object(organization['nutrition'])['mode'] ==
                            'inherit') {
                          organization['nutrition'] = {'mode': 'reference'};
                        }
                      }
                    }),
                  ),
                  if (organization['is_group'] != true)
                    Choice(
                      'ingredient_group',
                      organization['group_id'] as String? ?? '',
                      {
                        '': context.t('no_group'),
                        for (final item in catalogue!.items.where(
                          (i) =>
                              object(
                                    i['organization'] ?? <String, dynamic>{},
                                  )['is_group'] ==
                                  true &&
                              i['id'] != widget.id,
                        ))
                          item['id'] as String: catalogue!.name(item, locale),
                      },
                      (v) => change(
                        () => organization['group_id'] = v.isEmpty ? null : v,
                      ),
                    ),
                  Choice('diet', organization['diet'] as String? ?? 'unknown', {
                    'unknown': context.t('unknown_value'),
                    for (final k in ['vegan', 'vegetarian', 'meat_fish'])
                      k: context.t(k),
                  }, (v) => change(() => organization['diet'] = v)),
                  Choice(
                    'origin',
                    organization['origin'] as String? ?? 'unknown',
                    {
                      'unknown': context.t('unknown_value'),
                      for (final k in ['plant', 'animal', 'mixed'])
                        k: context.t(k),
                    },
                    (v) => change(() => organization['origin'] = v),
                  ),
                  Choice(
                    'production',
                    organization['production'] as String? ?? 'unknown',
                    {
                      'unknown': context.t('unknown_value'),
                      'purchased': context.t('purchased_decl'),
                      'household_produced': context.t('household_produced'),
                    },
                    (v) => change(() => organization['production'] = v),
                  ),
                  SectionTitle(context.t('nutrition')),
                  Note(context.t('per_100g')),
                  Choice(
                    'nutrition_mode',
                    object(organization['nutrition'])['mode'] as String,
                    {
                      'reference': context.t('reference'),
                      'own': context.t('own'),
                      if (organization['group_id'] != null &&
                          organization['is_group'] != true)
                        'inherit': context.t('inherit'),
                    },
                    (v) => change(
                      () => organization['nutrition'] = v == 'own'
                          ? {
                              'mode': 'own',
                              'values': <String, dynamic>{},
                              'constituents': <String, dynamic>{},
                              'note': '',
                              'source_url': '',
                            }
                          : {'mode': v},
                    ),
                  ),
                  if (object(organization['nutrition'])['mode'] == 'own')
                    ...nutritionFields(),
                  SectionTitle(context.t('tags')),
                  EditText(
                    'tags',
                    (annotation['tags'] as List? ?? []).join('\n'),
                    (v) => change(() => annotation['tags'] = lines(v)),
                    lines: 3,
                    max: 2400,
                  ),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('favorite')),
                    value: annotation['favorite'] as bool? ?? false,
                    onChanged: (v) => change(() => annotation['favorite'] = v),
                  ),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('archived')),
                    value: annotation['archived'] as bool? ?? false,
                    onChanged: (v) => change(() => annotation['archived'] = v),
                  ),
                  SectionTitle(context.t('components')),
                  for (final id in List<String>.from(
                    annotation['components'] as List? ?? [],
                  ))
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(itemName(id)),
                      trailing: IconButton(
                        tooltip: context.t('delete'),
                        icon: const Icon(Icons.delete_outline),
                        onPressed: () => change(
                          () => (annotation['components'] as List).remove(id),
                        ),
                      ),
                    ),
                  OutlinedButton.icon(
                    onPressed: () async {
                      final item = await pickIngredient(context, widget.api);
                      if (item != null &&
                          item['id'] != widget.id &&
                          !(annotation['components'] as List).contains(
                            item['id'],
                          )) {
                        change(
                          () => (annotation['components'] as List).add(
                            item['id'],
                          ),
                        );
                      }
                    },
                    icon: const Icon(Icons.add),
                    label: Text(context.t('find_ingredient')),
                  ),
                  SectionTitle(context.t('allergens')),
                  Note(context.t('allergen_review'), warning: true),
                  for (final id in [
                    'milk',
                    'egg',
                    'soy',
                    'gluten_cereals',
                    'peanut',
                    'tree_nuts',
                    'sesame',
                    'lupin',
                    'celery',
                    'mustard',
                    'fish',
                    'crustacean',
                    'mollusc',
                    'sulphites',
                  ])
                    CheckboxListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(context.t('allergen_$id')),
                      value: (annotation['declared_allergens'] as List? ?? [])
                          .contains(id),
                      onChanged: (v) => change(() {
                        annotation['declared_allergens'] ??= <String>[];
                        final list = annotation['declared_allergens'] as List;
                        v == true ? list.add(id) : list.remove(id);
                      }),
                    ),
                  CheckboxListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('sorbitol')),
                    value: (organization['intolerance_triggers'] as List? ?? [])
                        .contains('sorbitol'),
                    onChanged: (v) => change(
                      () => organization['intolerance_triggers'] = v == true
                          ? ['sorbitol']
                          : <String>[],
                    ),
                  ),
                  const SizedBox(height: 20),
                  FilledButton(
                    onPressed: busy || !widget.api.writable ? null : save,
                    child: Text(context.t('save')),
                  ),
                ],
              ),
            ),
    ),
  );
  List<Widget> nutritionFields() {
    final nutrition = object(organization['nutrition']);
    final values = object(nutrition['values']);
    nutrition['constituents'] ??= <String, dynamic>{};
    final constituents = object(nutrition['constituents']);
    return [
      Note(context.t('nutrition_unknown')),
      for (final id in values.keys.toList())
        Builder(
          builder: (context) {
            final definition = catalogue!.definitions
                .where((d) => d['id'] == id)
                .firstOrNull;
            return ReadingField(
              key: ValueKey(id),
              label:
                  '${definition?['label'] ?? id} (${definition?['unit'] ?? ''})',
              value: object(values[id]),
              changed: () => change(() {}),
              remove: () => change(() => values.remove(id)),
            );
          },
        ),
      Choice(
        'nutrition',
        null,
        {
          for (final d in catalogue!.definitions.where(
            (d) =>
                d['group'] != 'constituent' &&
                !['beta_glucan', 'soluble_fibre'].contains(d['id']) &&
                !values.containsKey(d['id']),
          ))
            d['id'] as String: '${d['label']} (${d['unit']})',
        },
        (id) => change(() => values[id] = {'amount': null, 'uncertainty': ''}),
      ),
      SectionTitle(context.t('constituents')),
      for (final id in constituents.keys.toList())
        Builder(
          builder: (context) {
            final d = object(constituents[id]);
            return Column(
              key: ValueKey(id),
              children: [
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(d['name'] as String),
                  subtitle: Text(id),
                  trailing: IconButton(
                    tooltip: context.t('delete'),
                    onPressed: () => change(() => constituents.remove(id)),
                    icon: const Icon(Icons.delete_outline),
                  ),
                ),
                Choice('unit', d['unit'] as String, {
                  for (final u in ['g', 'mg', 'µg']) u: u,
                }, (v) => change(() => d['unit'] = v)),
                EditText(
                  'amount',
                  d['amount'] as String? ?? '',
                  (v) => change(() => d['amount'] = v),
                  numeric: true,
                ),
              ],
            );
          },
        ),
      OutlinedButton.icon(
        onPressed: () async {
          final known = <String, Json>{
            for (final definition in catalogue!.definitions.where(
              (d) => ['beta_glucan', 'soluble_fibre'].contains(d['id']),
            ))
              definition['id'] as String: {
                'name': definition['label'],
                'unit': definition['unit'],
                'translations': <String, dynamic>{},
                'amount': null,
              },
            for (final i in catalogue!.items)
              for (final e in object(
                i['constituents'] ?? <String, dynamic>{},
              ).entries)
                e.key: object(e.value),
          };
          final id = await showModalBottomSheet<String>(
            context: context,
            isScrollControlled: true,
            builder: (context) => SafeArea(
              child: ListView(
                shrinkWrap: true,
                children: [
                  ListTile(
                    title: Text(context.t('new_constituent')),
                    leading: const Icon(Icons.add),
                    onTap: () => Navigator.pop(context, 'new'),
                  ),
                  for (final e in known.entries)
                    ListTile(
                      title: Text(e.value['name'] as String),
                      subtitle: Text(e.value['unit'] as String),
                      onTap: () => Navigator.pop(context, e.key),
                    ),
                ],
              ),
            ),
          );
          if (id == null || !mounted) return;
          if (id == 'new') {
            final name = await askName(context, 'new_constituent');
            if (name == null) return;
            change(
              () => constituents[newId('constituent-')] = {
                'name': name,
                'translations': <String, dynamic>{},
                'unit': 'g',
                'amount': null,
              },
            );
          } else {
            change(
              () => constituents[id] = {...clone(known[id]!), 'amount': null},
            );
          }
        },
        icon: const Icon(Icons.add),
        label: Text(context.t('new_constituent')),
      ),
      EditText(
        'declaration_note',
        nutrition['note'] as String,
        (v) => change(() => nutrition['note'] = v),
        required: true,
        max: 2000,
      ),
      EditText(
        'source_url',
        nutrition['source_url'] as String,
        (v) => change(() => nutrition['source_url'] = v),
        max: 2000,
      ),
    ];
  }
}

class ReadingField extends StatelessWidget {
  const ReadingField({
    super.key,
    required this.label,
    required this.value,
    required this.changed,
    required this.remove,
  });
  final String label;
  final Json value;
  final VoidCallback changed, remove;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      ListTile(
        contentPadding: EdgeInsets.zero,
        title: Text(label),
        trailing: IconButton(
          tooltip: context.t('delete'),
          onPressed: remove,
          icon: const Icon(Icons.delete_outline),
        ),
      ),
      if (value['range'] == null)
        EditText('amount', value['amount'] as String? ?? '', (v) {
          value['amount'] = v;
          changed();
        }, numeric: true),
      SwitchListTile(
        contentPadding: EdgeInsets.zero,
        title: Text(context.t('estimated')),
        value: value['range'] != null,
        onChanged: (v) {
          if (v) {
            value['range'] = {
              'lower': value['amount'] ?? '',
              'upper': value['amount'] ?? '',
            };
            value['amount'] = null;
          } else {
            value.remove('range');
          }
          changed();
        },
      ),
      if (value['range'] != null) ...[
        EditText(
          'range_lower',
          object(value['range'])['lower'] as String,
          (v) {
            object(value['range'])['lower'] = v;
            changed();
          },
          required: true,
          numeric: true,
        ),
        EditText(
          'range_upper',
          object(value['range'])['upper'] as String,
          (v) {
            object(value['range'])['upper'] = v;
            changed();
          },
          required: true,
          numeric: true,
        ),
      ],
      EditText('uncertainty', value['uncertainty'] as String? ?? '', (v) {
        value['uncertainty'] = v;
        changed();
      }, max: 2000),
      const Divider(),
    ],
  );
}
