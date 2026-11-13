import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'content_common.dart';
import 'common.dart';
import 'ingredient_picker.dart';
import 'messages.dart';
import 'photo.dart';

Json emptyPerson() => {
  'id': newId('person-'),
  'name': '',
  'allergies': <String>[],
  'allergen_ids': <String>[],
  'intolerances': <String>[],
  'excluded_ingredients': <String>[],
  'excluded_tags': <String>[],
  'disliked_ingredients': <String>[],
  'rules': {
    'version': 1,
    'preset': 'unrestricted',
    'exceptions': <Json>[],
    'availability': <Json>[],
    'max_active_seconds': null,
  },
};
const weekdays = [
  'monday',
  'tuesday',
  'wednesday',
  'thursday',
  'friday',
  'saturday',
  'sunday',
];
const meals = ['breakfast', 'lunch', 'snack', 'dinner'];

class PersonTour extends StatefulWidget {
  const PersonTour(this.api, {super.key, this.profiles, this.person});
  final HouseholdApi api;
  final Json? profiles, person;
  @override
  State<PersonTour> createState() => _PersonTourState();
}

class _PersonTourState extends State<PersonTour> {
  final form = GlobalKey<FormState>();
  late Json person, profiles;
  Catalogue? catalogue;
  List<Json> allergens = [];
  int step = 0;
  int? draftRevision;
  bool loaded = false,
      busy = false,
      dirty = false,
      finished = false,
      conflict = false;
  String? failure;
  Timer? timer;
  String get draftKey => 'person:${widget.person?['id'] ?? 'new'}';
  Json get rules => object(person['rules']);
  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  Future<void> load() async {
    try {
      final draft = widget.api.controller.draft(draftKey);
      profiles = await widget.api.household('/profiles');
      draftRevision =
          draft?['profile_revision'] as int? ?? profiles['revision'] as int;
      conflict = draftRevision != profiles['revision'];
      person = draft?['person'] is Json
          ? clone(object(draft!['person']))
          : clone(widget.person ?? emptyPerson());
      if (draft == null && widget.person == null) {
        person['name'] = 'Person ${objects(profiles['members']).length + 1}';
      }
      person['rules'] ??= emptyPerson()['rules'];
      step = draft?['step'] as int? ?? (widget.person == null ? 0 : 4);
      catalogue = Catalogue(await widget.api.household('/ingredients'));
      allergens = objects((await widget.api.household('/allergens'))['groups']);
      if (mounted) setState(() => loaded = true);
    } catch (e) {
      if (mounted) {
        setState(() => failure = e is ApiFailure ? e.code : 'request_failed');
      }
    }
  }

  void change(VoidCallback action) {
    action();
    dirty = true;
    timer?.cancel();
    timer = Timer(
      const Duration(milliseconds: 350),
      () => unawaited(persist()),
    );
    setState(() {});
  }

  Future<void> persist() async {
    if (!widget.api.writable) return;
    try {
      await widget.api.controller.saveDraft(draftKey, {
        'person': person,
        'profile_revision': draftRevision,
        'step': step,
      });
    } catch (e) {
      if (mounted) await showFailure(context, e);
    }
  }

  @override
  void dispose() {
    timer?.cancel();
    if (loaded && dirty && !finished) unawaited(persist());
    super.dispose();
  }

  Future<void> save() async {
    if (conflict || !form.currentState!.validate()) return;
    if ((person['name'] as String).trim().isEmpty) {
      setState(() => step = 0);
      return;
    }
    setState(() => busy = true);
    try {
      final members = objects(
        profiles['members'],
      ).where((m) => m['id'] != person['id']).toList()..add(person);
      await widget.api.household(
        '/profiles',
        method: 'PUT',
        body: {'expected_revision': profiles['revision'], 'members': members},
      );
      finished = true;
      timer?.cancel();
      await widget.api.controller.saveDraft(draftKey, null);
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  String label(String id) => catalogue!.find(id) == null
      ? id
      : catalogue!.name(catalogue!.find(id)!, widget.api.controller.language);
  Widget ingredientList(String key) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      SectionTitle(context.t(key)),
      for (final id in List<String>.from(person[key] as List))
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(label(id)),
          trailing: IconButton(
            tooltip: context.t('delete'),
            icon: const Icon(Icons.delete_outline),
            onPressed: () => change(() => (person[key] as List).remove(id)),
          ),
        ),
      OutlinedButton.icon(
        onPressed: () async {
          final item = await pickIngredient(context, widget.api);
          if (item != null && !(person[key] as List).contains(item['id'])) {
            change(() => (person[key] as List).add(item['id']));
          }
        },
        icon: const Icon(Icons.add),
        label: Text(context.t('find_ingredient')),
      ),
    ],
  );
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(context.t(widget.person == null ? 'new_person' : 'people')),
      actions: [
        if (loaded && widget.api.writable)
          IconButton(
            tooltip: context.t('clear_draft'),
            icon: const Icon(Icons.delete_sweep_outlined),
            onPressed: () async {
              if (await confirm(context, 'clear_draft')) {
                finished = true;
                timer?.cancel();
                await widget.api.controller.saveDraft(draftKey, null);
                if (context.mounted) Navigator.pop(context);
              }
            },
          ),
      ],
    ),
    body: !loaded
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
            child: Column(
              children: [
                LinearProgressIndicator(value: (step + 1) / 5),
                Padding(
                  padding: const EdgeInsets.fromLTRB(20, 16, 20, 0),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Text(
                      '${step + 1} / 5 · ${context.t(['person_name', 'rules', 'allergies', 'availability', 'summary'][step])}',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                  ),
                ),
                Expanded(
                  child: ListView(
                    padding: const EdgeInsets.all(20),
                    children: [
                      if (conflict) ...[
                        Note(
                          context.t('profile_revision_conflict'),
                          warning: true,
                        ),
                        OutlinedButton(
                          onPressed: () async {
                            final current = objects(profiles['members'])
                                .where((m) => m['id'] == person['id'])
                                .firstOrNull;
                            final accepted = await showDialog<bool>(
                              context: context,
                              builder: (context) => AlertDialog(
                                title: Text(context.t('compare_draft')),
                                content: SingleChildScrollView(
                                  child: Column(
                                    crossAxisAlignment:
                                        CrossAxisAlignment.start,
                                    mainAxisSize: MainAxisSize.min,
                                    children: [
                                      Text(context.t('server_version')),
                                      if (current != null)
                                        Text(
                                          personDescription(
                                            current,
                                            catalogue!,
                                            widget.api.controller.language,
                                          ),
                                        ),
                                      const Divider(),
                                      Text(context.t('your_draft')),
                                      Text(
                                        personDescription(
                                          person,
                                          catalogue!,
                                          widget.api.controller.language,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                                actions: [
                                  TextButton(
                                    onPressed: () =>
                                        Navigator.pop(context, false),
                                    child: Text(context.t('cancel')),
                                  ),
                                  FilledButton(
                                    onPressed: () =>
                                        Navigator.pop(context, true),
                                    child: Text(context.t('review')),
                                  ),
                                ],
                              ),
                            );
                            if (accepted == true && mounted) {
                              setState(() {
                                conflict = false;
                                draftRevision = profiles['revision'] as int;
                              });
                            }
                          },
                          child: Text(context.t('compare_draft')),
                        ),
                      ],
                      if (!widget.api.writable) Note(context.t('reader')),
                      if (step == 0) ...[
                        Note(context.t('tour_intro')),
                        EditText(
                          'person_name',
                          person['name'] as String,
                          (v) => change(() => person['name'] = v),
                          required: true,
                        ),
                        PhotoField(
                          widget.api,
                          (person['avatar'] as Json?)?['image_id'] as String?,
                          (id) =>
                              change(() => person['avatar'] = {'image_id': id}),
                        ),
                      ],
                      if (step == 1) ...[
                        Choice('preset', rules['preset'] as String, {
                          for (final key in [
                            'unrestricted',
                            'vegetarian',
                            'vegan',
                          ])
                            key: context.t(key),
                        }, (v) => change(() => rules['preset'] = v)),
                        Note(context.t('exceptions_note')),
                        SectionTitle(context.t('exceptions')),
                        for (final exception in objects(rules['exceptions']))
                          Column(
                            children: [
                              ListTile(
                                contentPadding: EdgeInsets.zero,
                                title: Text(
                                  label(exception['ingredient_id'] as String),
                                ),
                                trailing: IconButton(
                                  tooltip: context.t('delete'),
                                  icon: const Icon(Icons.delete_outline),
                                  onPressed: () => change(
                                    () => (rules['exceptions'] as List).remove(
                                      exception,
                                    ),
                                  ),
                                ),
                              ),
                              SwitchListTile(
                                contentPadding: EdgeInsets.zero,
                                title: Text(context.t('household_only')),
                                value:
                                    exception['household_produced_only']
                                        as bool,
                                onChanged: (v) => change(
                                  () =>
                                      exception['household_produced_only'] = v,
                                ),
                              ),
                            ],
                          ),
                        OutlinedButton.icon(
                          onPressed: () async {
                            final item = await pickIngredient(
                              context,
                              widget.api,
                            );
                            if (item != null &&
                                !objects(rules['exceptions']).any(
                                  (e) => e['ingredient_id'] == item['id'],
                                )) {
                              change(
                                () => (rules['exceptions'] as List).add({
                                  'ingredient_id': item['id'],
                                  'household_produced_only': false,
                                }),
                              );
                            }
                          },
                          icon: const Icon(Icons.add),
                          label: Text(context.t('find_ingredient')),
                        ),
                        ingredientList('excluded_ingredients'),
                        ingredientList('disliked_ingredients'),
                      ],
                      if (step == 2) ...[
                        Note(context.t('allergen_review'), warning: true),
                        for (final group in allergens)
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            title: Text(
                              (group['translations'] as Json?)?[widget
                                          .api
                                          .controller
                                          .language]
                                      as String? ??
                                  group['name'] as String,
                            ),
                            value: (person['allergen_ids'] as List).contains(
                              group['id'],
                            ),
                            onChanged: (v) => change(() {
                              final ids = person['allergen_ids'] as List;
                              v == true
                                  ? ids.add(group['id'])
                                  : ids.remove(group['id']);
                            }),
                          ),
                        EditText(
                          'allergy_notes',
                          (person['allergies'] as List).join('\n'),
                          (v) => change(() => person['allergies'] = lines(v)),
                          lines: 3,
                          max: 4000,
                        ),
                        EditText(
                          'intolerance_notes',
                          (person['intolerances'] as List).join('\n'),
                          (v) =>
                              change(() => person['intolerances'] = lines(v)),
                          lines: 3,
                          max: 4000,
                        ),
                        EditText(
                          'tags',
                          (person['excluded_tags'] as List? ?? []).join('\n'),
                          (v) =>
                              change(() => person['excluded_tags'] = lines(v)),
                          lines: 2,
                          max: 2400,
                        ),
                      ],
                      if (step == 3) ...[
                        EditText(
                          'max_active',
                          rules['max_active_seconds']?.toString() ?? '',
                          (v) => change(
                            () => rules['max_active_seconds'] = v.isEmpty
                                ? null
                                : int.tryParse(v),
                          ),
                          numeric: true,
                          integer: true,
                        ),
                        Note(context.t('availability_note')),
                        for (final window in objects(rules['availability']))
                          Column(
                            children: [
                              Choice('availability', window['meal'] as String, {
                                for (final m in meals) m: context.t(m),
                              }, (v) => change(() => window['meal'] = v)),
                              Wrap(
                                spacing: 6,
                                children: [
                                  for (var d = 0; d < 7; d++)
                                    FilterChip(
                                      label: Text(context.t(weekdays[d])),
                                      selected: (window['weekdays'] as List)
                                          .contains(d),
                                      onSelected: (v) => change(() {
                                        final days = window['weekdays'] as List;
                                        v ? days.add(d) : days.remove(d);
                                      }),
                                    ),
                                ],
                              ),
                              Align(
                                alignment: Alignment.centerRight,
                                child: IconButton(
                                  tooltip: context.t('delete'),
                                  icon: const Icon(Icons.delete_outline),
                                  onPressed: () => change(
                                    () => (rules['availability'] as List)
                                        .remove(window),
                                  ),
                                ),
                              ),
                            ],
                          ),
                        OutlinedButton.icon(
                          onPressed: () => change(
                            () => (rules['availability'] as List).add({
                              'weekdays': [0, 1, 2, 3, 4, 5, 6],
                              'meal': 'dinner',
                            }),
                          ),
                          icon: const Icon(Icons.add),
                          label: Text(context.t('add_window')),
                        ),
                      ],
                      if (step == 4) ...[
                        Text(
                          person['name'] as String,
                          style: Theme.of(context).textTheme.headlineSmall,
                        ),
                        ListTile(
                          contentPadding: EdgeInsets.zero,
                          title: Text(context.t('rules')),
                          subtitle: Text(
                            '${context.t(rules['preset'] as String)}${(rules['exceptions'] as List).isEmpty ? '' : ' · ${context.t('exceptions')}: ${(rules['exceptions'] as List).length}'}',
                          ),
                        ),
                        for (final e in objects(rules['exceptions']))
                          Text(
                            '${label(e['ingredient_id'] as String)}${e['household_produced_only'] == true ? ' · ${context.t('household_only')}' : ''}',
                          ),
                        Note(context.t('exceptions_note')),
                        for (final key in [
                          'allergen_ids',
                          'allergies',
                          'intolerances',
                          'excluded_ingredients',
                          'disliked_ingredients',
                          'excluded_tags',
                        ])
                          if ((person[key] as List? ?? []).isNotEmpty)
                            ListTile(
                              contentPadding: EdgeInsets.zero,
                              title: Text(
                                context.t(switch (key) {
                                  'allergen_ids' => 'allergens',
                                  'excluded_ingredients' => 'exclusions',
                                  'disliked_ingredients' => 'dislikes',
                                  _ => key,
                                }),
                              ),
                              subtitle: Text(
                                (person[key] as List)
                                    .map(
                                      (v) => key.endsWith('_ingredients')
                                          ? label(v as String)
                                          : v,
                                    )
                                    .join(', '),
                              ),
                            ),
                        if (rules['max_active_seconds'] != null)
                          Text(
                            '${context.t('max_active')}: ${rules['max_active_seconds']}',
                          ),
                        for (final w in objects(rules['availability']))
                          Text(
                            '${(w['weekdays'] as List).map((d) => context.t(weekdays[d as int])).join(', ')} · ${context.t(w['meal'] as String)}',
                          ),
                        Note(context.t('activity_access_note')),
                        if (widget.person != null && widget.api.owner)
                          TextButton.icon(
                            onPressed: busy
                                ? null
                                : () async {
                                    if (!await confirm(
                                      context,
                                      'delete_person',
                                    )) {
                                      return;
                                    }
                                    if (!context.mounted) return;
                                    setState(() => busy = true);
                                    await runAction(context, () async {
                                      await widget.api.household(
                                        '/profiles',
                                        method: 'PUT',
                                        body: {
                                          'expected_revision':
                                              profiles['revision'],
                                          'members':
                                              objects(profiles['members'])
                                                  .where(
                                                    (m) =>
                                                        m['id'] != person['id'],
                                                  )
                                                  .toList(),
                                        },
                                      );
                                      await widget.api.controller.saveDraft(
                                        draftKey,
                                        null,
                                      );
                                      if (context.mounted) {
                                        Navigator.pop(context);
                                      }
                                    });
                                    if (mounted) setState(() => busy = false);
                                  },
                            icon: const Icon(Icons.delete_outline),
                            label: Text(context.t('delete_person')),
                          ),
                      ],
                    ],
                  ),
                ),
                SafeArea(
                  top: false,
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(20, 8, 20, 12),
                    child: Row(
                      children: [
                        TextButton(
                          onPressed: busy
                              ? null
                              : () async {
                                  await persist();
                                  if (!context.mounted) return;
                                  if (step == 0) {
                                    Navigator.pop(context);
                                  } else {
                                    setState(() => step--);
                                  }
                                },
                          child: Text(context.t(step == 0 ? 'skip' : 'back')),
                        ),
                        const Spacer(),
                        FilledButton(
                          onPressed: busy || conflict || !widget.api.writable
                              ? null
                              : () async {
                                  if (step == 4) {
                                    await save();
                                    return;
                                  }
                                  if (!form.currentState!.validate()) return;
                                  setState(() => step++);
                                  await persist();
                                },
                          child: Text(context.t(step == 4 ? 'save' : 'next')),
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
  );
}
