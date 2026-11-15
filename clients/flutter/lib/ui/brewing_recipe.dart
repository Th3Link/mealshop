import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import '../brewing/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'brewing_common.dart';
import 'brewing_review.dart';
import 'ingredient_picker.dart';
import 'messages.dart';
import 'process_stage_editor.dart';

class BrewingRecipeEditor extends StatefulWidget {
  const BrewingRecipeEditor(this.c, {super.key, this.record});
  final CookingController c;
  final Json? record;
  @override
  State<BrewingRecipeEditor> createState() => _BrewingRecipeEditorState();
}

class _BrewingRecipeEditorState extends State<BrewingRecipeEditor> {
  final form = GlobalKey<FormState>();
  Json? catalogue, recipe;
  String? error;
  bool busy = true, dirty = false, saved = false;
  final notes = <String>{};
  @override
  void initState() {
    super.initState();
    recipe = widget.record == null
        ? null
        : clone(object(widget.record!['recipe']));
    load();
  }

  Future<void> load() async {
    try {
      catalogue = await widget.c.api.household('/brewing/templates');
    } catch (e) {
      error = e is ApiFailure ? e.code : 'request_failed';
    }
    if (mounted) setState(() => busy = false);
  }

  void changed() {
    dirty = true;
  }

  Future<void> choose(String id) async {
    if (recipe != null && !await confirm(context, 'brewTemplateReplace')) {
      return;
    }
    final template = objects(catalogue!['templates'])
        .firstWhere((t) => t['id'] == id);
    if (mounted) {
      setState(() {
        recipe = seedBrew(
          catalogue!,
          template,
          widget.c.api.controller.language,
          context.t('brewPhaseSource'),
        );
        dirty = true;
      });
    }
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(widget.record == null ? 'brewNewRecipe' : 'brewEdit'),
        ),
      ),
      body: catalogue == null
          ? busy
                ? const Center(child: CircularProgressIndicator())
                : EmptyContent(
                    icon: Icons.error_outline,
                    title: error ?? 'request_failed',
                    action: 'retry',
                    onAction: load,
                  )
          : Form(
              key: form,
              child: ListView(
                padding: const EdgeInsets.all(16),
                children: [
                  Note(context.t('brewTemplateHelp')),
                  Choice(
                    'brewTemplate',
                    recipe == null
                        ? null
                        : object(recipe!['template'])['id'] as String,
                    {
                      for (final template in objects(catalogue!['templates']))
                        template['id'] as String:
                            '${context.t(brewBeverages[template['beverage']]!)} · ${brewLocalized(template, widget.c.api.controller.language)}',
                    },
                    choose,
                  ),
                  if (error != null) Note(context.t(error!), warning: true),
                  BrewingPending(widget.c),
                  if (recipe != null) ...[
                    EditText('title', recipe!['title'] as String, (v) {
                      recipe!['title'] = v;
                      changed();
                    }, required: true),
                    Choice(
                      'brewRecipeLanguage',
                      recipe!['language'] as String,
                      {'en': 'English', 'de': 'Deutsch'},
                      (v) {
                        setState(() => recipe!['language'] = v);
                        changed();
                      },
                    ),
                    Text(
                      '${context.t('brew_original_language')}: ${recipe!['language']}',
                    ),
                    BrewQuantityField(
                      'brewVolume',
                      object(recipe!['volume']),
                      changed,
                      volume: true,
                    ),
                    if (object(recipe!['method']).containsKey('strategy'))
                      for (final method in brewMethods.entries)
                        Choice(
                          {
                            'strategy': 'brewStrategy',
                            'layout': 'brewLayout',
                            'heat': 'brewHeat',
                            'sparge': 'brewSparge',
                            'mash': 'brewMash',
                          }[method.key]!,
                          object(recipe!['method'])[method.key] as String,
                          brewOptions(
                            context,
                            method.key == 'mash'
                                ? {
                                    for (final e in method.value.entries)
                                      if (['extract', 'extract_steep'].contains(
                                            object(
                                              recipe!['method'],
                                            )['strategy'],
                                          )
                                          ? e.key == 'none'
                                          : e.key != 'none')
                                        e.key: e.value,
                                  }
                                : method.value,
                          ),
                          (v) => setState(() {
                            object(recipe!['method'])[method.key] = v;
                            if (method.key == 'strategy') {
                              if (['extract', 'extract_steep'].contains(v)) {
                                object(recipe!['method'])['mash'] = 'none';
                              } else if (object(recipe!['method'])['mash'] ==
                                  'none') {
                                object(recipe!['method'])['mash'] = 'infusion';
                              }
                            }
                            changed();
                          }),
                        )
                    else
                      Text(
                        context.t(
                          brewProcesses[object(recipe!['method'])['process']]!,
                        ),
                      ),
                    SectionTitle(context.t('ingredients')),
                    for (final line in objects(recipe!['ingredients']))
                      Column(
                        key: ValueKey(line['id']),
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          EditText('ingredient_name', line['name'] as String, (
                            v,
                          ) {
                            line['name'] = v;
                            changed();
                          }, required: true),
                          OutlinedButton.icon(
                            icon: const Icon(Icons.search),
                            label: Text(
                              context.t(
                                line['ingredient_id'] == null
                                    ? 'find_ingredient'
                                    : 'brew_linked_ingredient',
                              ),
                            ),
                            onPressed: () async {
                              final item = await pickIngredient(
                                context,
                                widget.c.api,
                              );
                              if (item != null && mounted) {
                                setState(() {
                                  line['ingredient_id'] = item['id'];
                                  if (line['name'] == '') {
                                    line['name'] =
                                        (item['translations']
                                            as Json?)?[recipe!['language']]?['name'] ??
                                        item['name'];
                                  }
                                  changed();
                                });
                              }
                            },
                          ),
                          if (line['ingredient_id'] != null)
                            TextButton(
                              onPressed: () => setState(() {
                                line['ingredient_id'] = null;
                                changed();
                              }),
                              child: Text(context.t('brew_unlink_ingredient')),
                            ),
                          if (line['quantity'] == null)
                            TextButton.icon(
                              icon: const Icon(Icons.add),
                              label: Text(context.t('brewAddQuantity')),
                              onPressed: () => setState(() {
                                line['quantity'] = {'amount': '', 'unit': 'g'};
                                changed();
                              }),
                            )
                          else ...[
                            BrewQuantityField(
                              'brewAmount',
                              object(line['quantity']),
                              changed,
                            ),
                            TextButton(
                              onPressed: () => setState(() {
                                line['quantity'] = null;
                                changed();
                              }),
                              child: Text(context.t('remove')),
                            ),
                          ],
                          Choice(
                            'brewPhase',
                            line['phase_id'] as String? ?? '',
                            {
                              '': context.t('brewNone'),
                              for (final phase in objects(recipe!['phases']))
                                object(phase['stage'])['id'] as String:
                                    object(phase['stage'])['name'] as String,
                            },
                            (v) => setState(() {
                              line['phase_id'] = v.isEmpty ? null : v;
                              changed();
                            }),
                          ),
                          if (notes.contains(line['id']) || line['note'] != '')
                            EditText(
                              'brewNotes',
                              line['note'] as String,
                              (v) {
                                line['note'] = v;
                                changed();
                              },
                              max: 1000,
                              lines: 2,
                            )
                          else
                            TextButton.icon(
                              icon: const Icon(Icons.add),
                              label: Text(context.t('brewNotes')),
                              onPressed: () => setState(
                                () => notes.add(line['id'] as String),
                              ),
                            ),
                          Align(
                            alignment: Alignment.centerRight,
                            child: IconButton(
                              tooltip: context.t('remove_ingredient'),
                              icon: const Icon(Icons.delete_outline),
                              onPressed: () => setState(() {
                                (recipe!['ingredients'] as List).remove(line);
                                changed();
                              }),
                            ),
                          ),
                          const Divider(),
                        ],
                      ),
                    TextButton.icon(
                      icon: const Icon(Icons.add),
                      label: Text(context.t('brewAddIngredient')),
                      onPressed: () => setState(() {
                        (recipe!['ingredients'] as List).add({
                          'id': newId('line-'),
                          'name': '',
                          'ingredient_id': null,
                          'quantity': null,
                          'phase_id': null,
                          'note': '',
                        });
                        changed();
                      }),
                    ),
                    SectionTitle(context.t('cookSteps')),
                    for (final phase in objects(recipe!['phases']))
                      BrewPhaseEditor(
                        phase,
                        key: ValueKey(object(phase['stage'])['id']),
                        recipe: recipe!,
                        catalogue: catalogue!,
                        changed: changed,
                        remove: () async {
                          if (!await confirm(context, 'brewRemovePhaseRefs')) {
                            return;
                          }
                          if (mounted) {
                            setState(() {
                              final id = object(phase['stage'])['id'];
                              (recipe!['phases'] as List).remove(phase);
                              for (final other in objects(recipe!['phases'])) {
                                (object(other['stage'])['dependencies'] as List)
                                    .remove(id);
                              }
                              for (final line in objects(
                                recipe!['ingredients'],
                              )) {
                                if (line['phase_id'] == id) {
                                  line['phase_id'] = null;
                                }
                              }
                              changed();
                            });
                          }
                        },
                      ),
                    TextButton.icon(
                      icon: const Icon(Icons.add),
                      label: Text(context.t('brewAddPhase')),
                      onPressed: () => setState(() {
                        (recipe!['phases'] as List).add(
                          brewPhase(
                            newId('phase-'),
                            context.t('brewPhaseSource'),
                          ),
                        );
                        changed();
                      }),
                    ),
                    if (recipe!['yeast_preparation'] == null)
                      TextButton.icon(
                        icon: const Icon(Icons.add),
                        label: Text(context.t('brewAddYeast')),
                        onPressed: () => setState(() {
                          recipe!['yeast_preparation'] = {
                            'days_before': 1,
                            'local_time': '12:00',
                            'fold': 'reject',
                            'instruction': '',
                          };
                          changed();
                        }),
                      )
                    else ...[
                      SectionTitle(context.t('brewYeast')),
                      Note(context.t('brewYeastHelp')),
                      EditText(
                        'brewDaysBefore',
                        '${object(recipe!['yeast_preparation'])['days_before']}',
                        (v) {
                          final n = int.tryParse(v);
                          if (n != null) {
                            object(
                              recipe!['yeast_preparation'],
                            )['days_before'] = n;
                          }
                          changed();
                        },
                        required: true,
                        integer: true,
                        numeric: true,
                        max: 2,
                      ),
                      EditText(
                        'brewLocalTime',
                        object(recipe!['yeast_preparation'])['local_time']
                            as String,
                        (v) {
                          object(recipe!['yeast_preparation'])['local_time'] =
                              v;
                          changed();
                        },
                        required: true,
                        max: 5,
                      ),
                      Choice(
                        'cookFoldChoice',
                        object(recipe!['yeast_preparation'])['fold'] as String,
                        {
                          'reject': context.t('cookFoldReject'),
                          'earlier': context.t('cookFoldEarlier'),
                          'later': context.t('cookFoldLater'),
                        },
                        (v) => setState(() {
                          object(recipe!['yeast_preparation'])['fold'] = v;
                          changed();
                        }),
                      ),
                      EditText(
                        'instruction',
                        object(recipe!['yeast_preparation'])['instruction']
                            as String,
                        (v) {
                          object(recipe!['yeast_preparation'])['instruction'] =
                              v;
                          changed();
                        },
                        required: true,
                        lines: 2,
                        max: 4000,
                      ),
                      TextButton(
                        onPressed: () => setState(() {
                          recipe!.remove('yeast_preparation');
                          changed();
                        }),
                        child: Text(context.t('remove')),
                      ),
                    ],
                    EditText(
                      'brewNotes',
                      recipe!['notes'] as String,
                      (v) {
                        recipe!['notes'] = v;
                        changed();
                      },
                      lines: 3,
                      max: 4000,
                    ),
                    FilledButton(
                      onPressed: busy || !widget.c.writable
                          ? null
                          : () async {
                              if (!form.currentState!.validate()) return;
                              setState(() => busy = true);
                              try {
                                final result = await reviewBrewing(
                                  context,
                                  widget.c,
                                  {
                                    'operation': 'save_recipe',
                                    'id': widget.record?['id'],
                                    'expected_recipe_revision':
                                        widget.record?['revision'],
                                    'recipe': clone(recipe!),
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

class BrewPhaseEditor extends StatefulWidget {
  const BrewPhaseEditor(
    this.phase, {
    super.key,
    required this.recipe,
    required this.catalogue,
    required this.changed,
    required this.remove,
  });
  final Json phase, recipe, catalogue;
  final VoidCallback changed;
  final Future<void> Function() remove;
  @override
  State<BrewPhaseEditor> createState() => _BrewPhaseEditorState();
}

class _BrewPhaseEditorState extends State<BrewPhaseEditor> {
  Json? rememberedTiming;
  @override
  Widget build(BuildContext context) {
    final phase = widget.phase, stage = object(phase['stage']);
    final keys = objects(widget.catalogue['templates'])
        .where((t) => t['beverage'] == widget.recipe['beverage'])
        .expand((t) => (t['phases'] as List).cast<String>())
        .toSet();
    final kinds = <String, String>{
      for (final entry in object(
        widget.catalogue['phases'],
      ).entries.where((e) => keys.contains(e.key)))
        object(entry.value)['kind'] as String: brewLocalized(
          object(entry.value),
          Localizations.localeOf(context).languageCode,
        ),
      'custom': context.t('brewOther'),
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Choice(
          'brewPhase',
          phase['kind'] as String,
          kinds,
          (v) => setState(() {
            phase['kind'] = v;
            widget.changed();
          }),
        ),
        CheckboxListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(context.t('brewKnownTiming')),
          value: phase['timing_known'] == true,
          onChanged: (v) => setState(() {
            phase['timing_known'] = v == true;
            if (v != true) {
              rememberedTiming = {
                for (final key in [
                  'active_seconds',
                  'passive_seconds',
                  'passive_min_seconds',
                  'passive_max_seconds',
                ])
                  key: stage[key],
              };
              for (final key in rememberedTiming!.keys) {
                stage[key] = 0;
              }
            } else if (rememberedTiming != null) {
              stage.addAll(rememberedTiming!);
            }
            widget.changed();
          }),
        ),
        if (phase['timing_known'] != true) Note(context.t('brewUnknownTiming')),
        if (phase['temperature'] != null) ...[
          BrewMeasurementField(
            'brewTemperature',
            object(phase['temperature']),
            widget.changed,
            temperature: true,
          ),
          TextButton(
            onPressed: () => setState(() {
              phase['temperature'] = null;
              widget.changed();
            }),
            child: Text(context.t('remove')),
          ),
        ] else
          TextButton.icon(
            icon: const Icon(Icons.add),
            label: Text(context.t('brewAddTemperature')),
            onPressed: () => setState(() {
              phase['temperature'] = {'value': '', 'unit': 'celsius'};
              widget.changed();
            }),
          ),
        if (phase['volume'] != null) ...[
          BrewQuantityField(
            'brewPhaseVolume',
            object(phase['volume']),
            widget.changed,
            volume: true,
          ),
          TextButton(
            onPressed: () => setState(() {
              phase['volume'] = null;
              widget.changed();
            }),
            child: Text(context.t('remove')),
          ),
        ] else
          TextButton.icon(
            icon: const Icon(Icons.add),
            label: Text(context.t('brewPhaseVolume')),
            onPressed: () => setState(() {
              phase['volume'] = {'amount': '', 'unit': 'l'};
              widget.changed();
            }),
          ),
        ProcessStageEditor(
          stage,
          stages: objects(widget.recipe['phases'])
              .map((p) => object(p['stage']))
              .toList(),
          changed: widget.changed,
          remove: widget.remove,
          knownTiming: phase['timing_known'] == true,
          maximum: 31536000,
          allowAnchor: false,
          allowEquipment: false,
          completionWait: false,
        ),
      ],
    );
  }
}
