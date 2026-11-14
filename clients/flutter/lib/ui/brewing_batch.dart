import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import '../brewing/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'brewing_common.dart';
import 'brewing_review.dart';
import 'messages.dart';
import 'process_common.dart';
import 'recipe_editor.dart' show chooseReferences;

class BrewingBatchEditor extends StatefulWidget {
  const BrewingBatchEditor(this.c, this.state, {super.key, this.recipeId});
  final CookingController c;
  final Json state;
  final String? recipeId;
  @override
  State<BrewingBatchEditor> createState() => _BrewingBatchEditorState();
}

class _BrewingBatchEditorState extends State<BrewingBatchEditor> {
  final form = GlobalKey<FormState>();
  late final Json draft = {
    'name': '',
    'recipe_id': '',
    'recipe_revision': 1,
    'brew_day': localMoment(
      widget.c.clock.sample?.timezone ?? 'UTC',
      DateTime.now().add(const Duration(days: 1)),
    ),
    'yeast_preparation': {'choice': 'skip'},
    'equipment': <String, dynamic>{},
  };
  Json? selected;
  bool dirty = false, busy = false, saved = false;
  String? error;
  List<Json> get records =>
      objects(widget.state['recipes'])
          .where((r) => r['archived'] != true)
          .map((r) => object(r['record']))
          .toList();
  @override
  void initState() {
    super.initState();
    final id = widget.recipeId ?? records.firstOrNull?['id'] as String?;
    if (id != null) choose(id);
  }

  void choose(String id) {
    selected = records.firstWhere((r) => r['id'] == id);
    final recipe = object(selected!['recipe']);
    draft['name'] = recipe['title'];
    draft['recipe_id'] = id;
    draft['recipe_revision'] = selected!['revision'];
    draft['yeast_preparation'] = {
      'choice': recipe['yeast_preparation'] == null ? 'skip' : 'recipe',
    };
    draft['equipment'] = {
      for (final phase in objects(recipe['phases']))
        object(phase['stage'])['id'] as String: <String>[],
    };
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('brewNewBatch'))),
      body: Form(
        key: form,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            if (records.isEmpty) Note(context.t('brewFirstRecipe')),
            Choice(
              'brewRecipes',
              draft['recipe_id'] as String,
              {
                for (final record in records)
                  record['id'] as String:
                      '${object(record['recipe'])['title']} · v${record['revision']}',
              },
              (v) => setState(() {
                choose(v);
                dirty = true;
              }),
            ),
            if (selected != null) ...[
              EditText('brewBatchName', draft['name'] as String, (v) {
                draft['name'] = v;
                dirty = true;
              }, required: true),
              LocalMomentEditor(
                'brewDay',
                object(draft['brew_day']),
                (v) => setState(() {
                  draft['brew_day'] = v;
                  dirty = true;
                }),
              ),
              Choice(
                'brewYeast',
                object(draft['yeast_preparation'])['choice'] as String,
                {
                  'skip': context.t('brewSkipPreparation'),
                  if (object(selected!['recipe'])['yeast_preparation'] != null)
                    'recipe': context.t('brewRecipePreparation'),
                  'custom': context.t('brewCustomPreparation'),
                },
                (v) => setState(() {
                  draft['yeast_preparation'] = v == 'custom'
                      ? {
                          'choice': 'custom',
                          'at': clone(object(draft['brew_day'])),
                          'instruction': '',
                        }
                      : {'choice': v};
                  dirty = true;
                }),
              ),
              if (object(draft['yeast_preparation'])['choice'] == 'custom') ...[
                LocalMomentEditor(
                  'brewYeast',
                  object(object(draft['yeast_preparation'])['at']),
                  (v) => setState(() {
                    object(draft['yeast_preparation'])['at'] = v;
                    dirty = true;
                  }),
                ),
                EditText(
                  'instruction',
                  object(draft['yeast_preparation'])['instruction'] as String,
                  (v) {
                    object(draft['yeast_preparation'])['instruction'] = v;
                    dirty = true;
                  },
                  required: true,
                  lines: 2,
                  max: 4000,
                ),
              ],
              Note(context.t('brewEquipmentHelp')),
              for (final phase in objects(
                object(selected!['recipe'])['phases'],
              ))
                Builder(
                  builder: (context) {
                    final stage = object(phase['stage']),
                        id = stage['id'] as String,
                        chosen = (object(draft['equipment'])[id] as List)
                            .cast<String>();
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            stage['name'] as String,
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          Text(
                            '${context.t('brewPhaseVolume')}: ${brewQuantityText(context, phase['volume'])}',
                          ),
                          if (phase['timing_known'] != true)
                            Note(context.t('brewUnknownTiming')),
                          OutlinedButton.icon(
                            icon: const Icon(Icons.kitchen_outlined),
                            label: Text(
                              chosen.isEmpty
                                  ? context.t('brewNoEquipment')
                                  : objects(widget.state['equipment'])
                                        .where((e) => chosen.contains(e['id']))
                                        .map(
                                          (e) => object(e['details'])['name'],
                                        )
                                        .join(', '),
                            ),
                            onPressed: () async {
                              final picked = await chooseReferences(
                                context,
                                'brewEquipment',
                                {
                                  for (final equipment in objects(
                                    widget.state['equipment'],
                                  ).where((e) => e['archived'] != true))
                                    equipment['id'] as String:
                                        '${object(equipment['details'])['name']} · ${brewQuantityText(context, object(equipment['details'])['capacity'])}',
                                },
                                chosen,
                              );
                              if (picked != null && mounted) {
                                setState(() {
                                  object(draft['equipment'])[id] = picked;
                                  dirty = true;
                                });
                              }
                            },
                          ),
                          for (final batch
                              in objects(widget.state['batches']).where(
                                (b) => object(b['run'])['status'] == 'active',
                              ))
                            for (final step
                                in objects(object(batch['run'])['steps']).where(
                                  (s) =>
                                      ![
                                        'completed',
                                        'cancelled',
                                      ].contains(object(s['timer'])['state']) &&
                                      (object(s['stage'])['equipment'] as List)
                                          .any(chosen.contains),
                                ))
                              Note(
                                '${context.t(['running', 'paused'].contains(object(step['timer'])['state']) ? 'brewInUse' : 'brewReservation')}: ${object(batch['run'])['name']} · ${object(step['stage'])['name']}',
                                warning: true,
                              ),
                        ],
                      ),
                    );
                  },
                ),
              BrewingPending(widget.c),
              if (error != null) Note(context.t(error!), warning: true),
              FilledButton(
                onPressed:
                    busy || !widget.c.writable || widget.c.pending.isNotEmpty
                    ? null
                    : () async {
                        if (!form.currentState!.validate()) return;
                        setState(() => busy = true);
                        try {
                          final result = await reviewBrewing(
                            context,
                            widget.c,
                            {
                              'operation': 'create_batch',
                              'draft': clone(draft),
                            },
                          );
                          if (result != null && context.mounted) {
                            saved = true;
                            Navigator.pop(context, result['result_id']);
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
                child: Text(context.t('brewReview')),
              ),
              if (busy) const LinearProgressIndicator(),
            ],
          ],
        ),
      ),
    ),
  );
}

class BrewingEquipmentEditor extends StatefulWidget {
  const BrewingEquipmentEditor(this.c, {super.key, this.record});
  final CookingController c;
  final Json? record;
  @override
  State<BrewingEquipmentEditor> createState() => _BrewingEquipmentEditorState();
}

class _BrewingEquipmentEditorState extends State<BrewingEquipmentEditor> {
  final form = GlobalKey<FormState>();
  late Json equipment = widget.record == null
      ? {
          'name': '',
          'kind': 'kettle',
          'model': '',
          'capacity': null,
          'notes': '',
        }
      : clone(object(widget.record!['details']));
  bool dirty = false, saved = false, busy = false;
  String? error;
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('brewEquipment'))),
      body: Form(
        key: form,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            EditText('name', equipment['name'] as String, (v) {
              equipment['name'] = v;
              dirty = true;
            }, required: true),
            Choice(
              'brewEquipmentKind',
              equipment['kind'] as String,
              brewOptions(context, brewEquipmentKinds),
              (v) => setState(() {
                equipment['kind'] = v;
                dirty = true;
              }),
            ),
            EditText('brewModel', equipment['model'] as String, (v) {
              equipment['model'] = v;
              dirty = true;
            }),
            if (equipment['capacity'] == null)
              TextButton.icon(
                icon: const Icon(Icons.add),
                label: Text(context.t('brewCapacity')),
                onPressed: () => setState(() {
                  equipment['capacity'] = {'amount': '', 'unit': 'l'};
                  dirty = true;
                }),
              )
            else ...[
              BrewQuantityField(
                'brewCapacity',
                object(equipment['capacity']),
                () => dirty = true,
                volume: true,
              ),
              TextButton(
                onPressed: () => setState(() {
                  equipment['capacity'] = null;
                  dirty = true;
                }),
                child: Text(context.t('remove')),
              ),
            ],
            Note(context.t('brewVolumeHelp')),
            EditText(
              'brewNotes',
              equipment['notes'] as String,
              (v) {
                equipment['notes'] = v;
                dirty = true;
              },
              lines: 3,
              max: 4000,
            ),
            BrewingPending(widget.c),
            if (error != null) Note(context.t(error!), warning: true),
            FilledButton(
              onPressed:
                  busy || !widget.c.writable || widget.c.pending.isNotEmpty
                  ? null
                  : () async {
                      if (!form.currentState!.validate()) return;
                      setState(() => busy = true);
                      try {
                        final result = await reviewBrewing(context, widget.c, {
                          'operation': 'save_equipment',
                          'id': widget.record?['id'],
                          'expected_equipment': widget.record?['details'],
                          'equipment': clone(equipment),
                        });
                        if (result != null && context.mounted) {
                          saved = true;
                          Navigator.pop(context, result['result_id']);
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
              child: Text(context.t('brewReview')),
            ),
            if (busy) const LinearProgressIndicator(),
          ],
        ),
      ),
    ),
  );
}
