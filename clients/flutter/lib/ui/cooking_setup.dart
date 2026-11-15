import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import 'common.dart';
import 'content_common.dart';
import 'cooking_review.dart';
import 'messages.dart';
import 'process_common.dart';
import 'process_stage_editor.dart';
import 'recipe_editor.dart';

Future<Json?> pickCookingRecipe(BuildContext context, HouseholdApi api) =>
    openPage<Json>(context, _CookingRecipePicker(api));

class _CookingRecipePicker extends StatefulWidget {
  const _CookingRecipePicker(this.api);
  final HouseholdApi api;
  @override
  State<_CookingRecipePicker> createState() => _CookingRecipePickerState();
}

class _CookingRecipePickerState extends State<_CookingRecipePicker> {
  String query = '';
  int version = 0;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('recipes'))),
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            decoration: InputDecoration(
              labelText: context.t('browse'),
              prefixIcon: const Icon(Icons.search),
            ),
            onChanged: (v) => setState(() => query = v.toLowerCase().trim()),
          ),
        ),
        Expanded(
          child: Resource<Json>(
            key: ValueKey(version),
            load: () => widget.api.recipes(),
            builder: (context, data, reload) {
              final recipes = objects(data['recipes'])
                  .where(
                    (r) => (r['title'] as String).toLowerCase().contains(query),
                  )
                  .toList();
              return ListView(
                children: [
                  if (recipes.isEmpty) Note(context.t('cookNoRecipes')),
                  for (final recipe in recipes)
                    ListTile(
                      title: Text(recipe['title'] as String),
                      subtitle: Text('v${recipe['revision']}'),
                      trailing: const Icon(Icons.chevron_right),
                      onTap: () => Navigator.pop(context, recipe),
                    ),
                ],
              );
            },
          ),
        ),
        if (widget.api.writable)
          Padding(
            padding: const EdgeInsets.all(16),
            child: OutlinedButton.icon(
              icon: const Icon(Icons.add),
              label: Text(context.t('new_recipe')),
              onPressed: () async {
                await openPage<String>(context, RecipeEditor(widget.api));
                if (mounted) setState(() => version++);
              },
            ),
          ),
      ],
    ),
  );
}

class CookingSetupScreen extends StatefulWidget {
  const CookingSetupScreen(
    this.c,
    this.recipe, {
    super.key,
    this.planSlot,
    this.servings,
  });
  final CookingController c;
  final Json recipe;
  final Json? planSlot;
  final int? servings;
  @override
  State<CookingSetupScreen> createState() => _CookingSetupScreenState();
}

class _CookingSetupScreenState extends State<CookingSetupScreen> {
  final form = GlobalKey<FormState>();
  Json? draft;
  String? error;
  bool busy = true, dirty = false, saved = false;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    try {
      await widget.c.initialize();
      if (widget.c.list == null) await widget.c.refresh();
      if (!widget.c.protocol) throw const ApiFailure('mobile_upgrade_required');
      final record = await widget.c.api.recipes(
        '/${widget.recipe['id']}?revision=${widget.recipe['revision']}',
      );
      final timezone = widget.c.clock.sample?.timezone ?? 'UTC';
      final query = Uri(
        queryParameters: {
          'recipe_id': widget.recipe['id'] as String,
          'revision': '${widget.recipe['revision']}',
          'servings':
              '${widget.servings ?? object(record['recipe'])['servings']}',
          'timezone': timezone,
        },
      ).query;
      final value = await widget.c.api.household('/cooking/template?$query');
      draft = clone(object(value['draft']));
      if (widget.planSlot != null) {
        draft!['plan_slot'] = clone(widget.planSlot!);
      }
    } catch (e) {
      error = e is ApiFailure ? e.code : 'request_failed';
    }
    if (mounted) setState(() => busy = false);
  }

  void changed() {
    dirty = true;
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('cookNew'))),
      body: draft == null
          ? busy
                ? const Center(child: CircularProgressIndicator())
                : EmptyContent(
                    icon: Icons.error_outline,
                    title: error ?? 'request_failed',
                    action: 'retry',
                    onAction: () => setState(() {
                      busy = true;
                      error = null;
                      load();
                    }),
                  )
          : Form(
              key: form,
              child: ListView(
                padding: const EdgeInsets.all(16),
                children: [
                  Text(
                    widget.recipe['title'] as String,
                    style: Theme.of(context).textTheme.titleLarge,
                  ),
                  Note(context.t('cookStagesHelp')),
                  if (error != null) Note(context.t(error!), warning: true),
                  EditText('name', draft!['name'] as String, (v) {
                    draft!['name'] = v;
                    changed();
                  }, required: true),
                  EditText(
                    'servings',
                    '${draft!['servings']}',
                    (v) {
                      final n = int.tryParse(v);
                      if (n != null) draft!['servings'] = n;
                      changed();
                    },
                    required: true,
                    integer: true,
                    numeric: true,
                  ),
                  EditText(
                    'cookTimezone',
                    draft!['timezone'] as String,
                    (v) {
                      draft!['timezone'] = v;
                      if (draft!['target'] != null) {
                        object(draft!['target'])['timezone'] = v;
                      }
                      for (final stage in objects(draft!['stages'])) {
                        if (stage['start_at'] != null) {
                          object(stage['start_at'])['timezone'] = v;
                        }
                      }
                      changed();
                    },
                    required: true,
                    hint: 'cookTimeZoneHelp',
                  ),
                  Choice(
                    'cookKind',
                    draft!['kind'] as String,
                    {
                      'cooking': context.t('cookKindCooking'),
                      'bread': context.t('cookKindBread'),
                      'rolls': context.t('cookKindRolls'),
                      'pizza': context.t('cookKindPizza'),
                      'baguette': context.t('cookKindBaguette'),
                    },
                    (v) => setState(() {
                      draft!['kind'] = v;
                      if (v != 'cooking' && draft!['target'] == null) {
                        draft!['target'] = localMoment(
                          draft!['timezone'] as String,
                          DateTime.now().add(const Duration(days: 1)),
                        );
                      }
                      changed();
                    }),
                  ),
                  if (draft!['target'] != null)
                    LocalMomentEditor('cookTarget', object(draft!['target']), (
                      v,
                    ) {
                      setState(() => draft!['target'] = v);
                      changed();
                    }, zone: false)
                  else
                    TextButton.icon(
                      icon: const Icon(Icons.add),
                      label: Text(context.t('cookTarget')),
                      onPressed: () => setState(() {
                        draft!['target'] = localMoment(
                          draft!['timezone'] as String,
                          DateTime.now().add(const Duration(days: 1)),
                        );
                        changed();
                      }),
                    ),
                  if (draft!['kind'] == 'cooking' && draft!['target'] != null)
                    TextButton(
                      onPressed: () => setState(() {
                        draft!['target'] = null;
                        changed();
                      }),
                      child: Text(context.t('remove')),
                    ),
                  SectionTitle(context.t('cookSteps')),
                  for (final stage in objects(draft!['stages']))
                    ProcessStageEditor(
                      stage,
                      key: ValueKey(stage['id']),
                      stages: objects(draft!['stages']),
                      timezone: draft!['timezone'] as String,
                      changed: changed,
                      remove: () => setState(() {
                        final id = stage['id'];
                        (draft!['stages'] as List).removeWhere(
                          (s) => object(s)['id'] == id,
                        );
                        for (final other in objects(draft!['stages'])) {
                          (other['dependencies'] as List).remove(id);
                        }
                        changed();
                      }),
                    ),
                  TextButton.icon(
                    icon: const Icon(Icons.add),
                    label: Text(context.t('cookAddStage')),
                    onPressed: () => setState(() {
                      (draft!['stages'] as List).add(
                        newProcessStage(newId('stage-')),
                      );
                      changed();
                    }),
                  ),
                  FilledButton(
                    onPressed:
                        busy ||
                            !widget.c.writable ||
                            widget.c.pending.isNotEmpty
                        ? null
                        : () async {
                            if (!form.currentState!.validate()) return;
                            setState(() => busy = true);
                            try {
                              await widget.c.refresh();
                              if (!widget.c.api.controller.online) {
                                throw const ApiFailure('online_required');
                              }
                              final preview = await widget.c.api.household(
                                '/cooking/preview',
                                method: 'POST',
                                body: {
                                  'expected_revision':
                                      widget.c.list!['revision'],
                                  'proposal': {
                                    'kind': 'create',
                                    'draft': draft,
                                  },
                                },
                              );
                              if (!context.mounted) return;
                              final id = await openPage<String>(
                                context,
                                CookingReviewScreen(widget.c, preview),
                              );
                              if (id != null && context.mounted) {
                                saved = true;
                                Navigator.pop(context, id);
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
                    child: Text(context.t('cookPreview')),
                  ),
                  if (busy) const LinearProgressIndicator(),
                ],
              ),
            ),
    ),
  );
}
