import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'cooking_scope.dart';
import '../planning/data.dart';
import 'content_common.dart';
import 'common.dart';
import 'messages.dart';
import 'plan_common.dart';
import 'process_common.dart';
import 'history.dart';

class HistoryEditor extends StatefulWidget {
  const HistoryEditor(this.api, {super.key, this.entry, this.run});
  final HouseholdApi api;
  final Json? entry, run;
  @override
  State<HistoryEditor> createState() => _HistoryEditorState();
}

class _HistoryEditorState extends State<HistoryEditor> {
  final form = GlobalKey<FormState>();
  final zone = TextEditingController(), servings = TextEditingController();
  Json? profiles, source;
  List<Json> recipes = [];
  int revision = 0;
  String date = isoDay(DateTime.now());
  final Set<String> attendees = {};
  final Map<String, dynamic> ratings = {};
  int? householdRating, effort;
  bool knownEffort = false,
      confirmed = false,
      loading = true,
      busy = false,
      dirty = false,
      saved = false,
      uncertain = false;
  String? error;
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    zone.dispose();
    servings.dispose();
    super.dispose();
  }

  Future<void> load() async {
    try {
      final values = await Future.wait([
        widget.api.household(
          widget.entry == null ? '/history' : '/history/${widget.entry!['id']}',
        ),
        widget.api.household('/profiles'),
        widget.api.recipes(),
      ]);
      revision = values[0]['revision'] as int;
      profiles = values[1];
      recipes = objects(values[2]['recipes']);
      final entry = widget.entry;
      if (entry != null) {
        // The snapshot the user opened remains the correction base. A concurrent correction is rejected.
        if (object(values[0]['entry']).toString() != entry.toString()) {
          throw const ApiFailure('history_revision_conflict');
        }
        final input = object(entry['input']);
        date = input['date'] as String;
        zone.text = input['timezone'] as String;
        attendees.addAll((input['attendees'] as List).cast<String>());
        ratings.addAll(object(input['ratings']));
        householdRating = input['household_rating'] as int?;
        effort = input['active_seconds'] as int?;
        knownEffort = effort != null;
        servings.text = '${entry['servings']}';
        source = object(entry['source'])['kind'] == 'session'
            ? clone(object(entry['source']))
            : {
                'kind': 'manual',
                'recipe_id': object(entry['recipe'])['id'],
                'revision': object(entry['recipe'])['revision'],
                'servings': entry['servings'],
              };
      } else {
        zone.text = mounted
            ? CookingScope.maybeOf(context)?.clock.sample?.timezone ?? 'UTC'
            : 'UTC';
        final run = widget.run;
        if (run != null) {
          source = {'kind': 'session', 'run_id': run['id']};
          servings.text = '${run['servings']}';
        } else if (recipes.isNotEmpty) {
          source = {
            'kind': 'manual',
            'recipe_id': recipes.first['id'],
            'revision': recipes.first['revision'],
            'servings': recipes.first['servings'],
          };
          servings.text = '${recipes.first['servings']}';
        }
      }
    } catch (e) {
      error = e is ApiFailure ? e.code : 'request_failed';
    }
    if (mounted) setState(() => loading = false);
  }

  Future<void> save() async {
    if (!widget.api.writable ||
        !widget.api.controller.online ||
        source == null ||
        !confirmed ||
        !form.currentState!.validate() ||
        attendees.isEmpty ||
        uncertain) {
      return;
    }
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final body = {
        'expected_revision': revision,
        'expected_profile_revision': profiles!['revision'],
        'confirmed_completed': true,
        'source': {
          ...source!,
          if (source!['kind'] == 'manual') 'servings': int.parse(servings.text),
        },
        'input': {
          'date': date,
          'timezone': zone.text.trim(),
          'attendees': attendees.toList(),
          'active_seconds': knownEffort ? effort : null,
          'household_rating': householdRating,
          'ratings': ratings,
        },
      };
      await widget.api.household(
        widget.entry == null ? '/history' : '/history/${widget.entry!['id']}',
        method: widget.entry == null ? 'POST' : 'PUT',
        body: body,
      );
      saved = true;
      if (mounted) Navigator.pop(context);
    } on ApiFailure catch (e) {
      if (e.status == 0 || e.status >= 500) {
        uncertain = true;
        error = 'history_response_unknown';
      } else {
        error = e.code;
      }
    } catch (_) {
      uncertain = true;
      error = 'history_response_unknown';
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(widget.entry == null ? 'historyNew' : 'historyEdit'),
        ),
      ),
      body: loading
          ? const Center(child: CircularProgressIndicator())
          : profiles == null
          ? EmptyContent(
              icon: Icons.cloud_off_outlined,
              title: error ?? 'request_failed',
              action: 'retry',
              onAction: () {
                setState(() => loading = true);
                load();
              },
            )
          : Form(
              key: form,
              onChanged: () => dirty = true,
              child: ListView(
                padding: const EdgeInsets.all(16),
                children: [
                  Note(context.t('historyVisibility')),
                  if (error != null) Note(context.t(error!), warning: true),
                  if (uncertain)
                    OutlinedButton(
                      onPressed: () =>
                          openPage<void>(context, HistoryScreen(widget.api)),
                      child: Text(context.t('historyReload')),
                    ),
                  if (!widget.api.controller.online)
                    Note(context.t('online_required'), warning: true),
                  if (source?['kind'] == 'session') ...[
                    Text(
                      '${widget.run?['name'] ?? object(widget.entry!['recipe'])['title']} · ${servings.text} ${context.t('servings')}',
                    ),
                    Note(context.t('historyPinned')),
                  ] else ...[
                    DropdownButtonFormField<String>(
                      key: ValueKey(source?['recipe_id']),
                      initialValue: source?['recipe_id'] as String?,
                      isExpanded: true,
                      decoration: InputDecoration(
                        labelText: context.t('historyRecipe'),
                      ),
                      items: [
                        for (final r in recipes)
                          DropdownMenuItem(
                            value: r['id'] as String,
                            child: Text(
                              '${r['title']} · v${r['revision']}',
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        if (source != null &&
                            !recipes.any(
                              (r) => r['id'] == source!['recipe_id'],
                            ))
                          DropdownMenuItem(
                            value: source!['recipe_id'] as String,
                            child: Text(
                              object(widget.entry!['recipe'])['title']
                                  as String,
                            ),
                          ),
                      ],
                      onChanged: busy || uncertain
                          ? null
                          : (id) {
                              final r = recipes.firstWhere(
                                (r) => r['id'] == id,
                              );
                              setState(() {
                                dirty = true;
                                source = {
                                  'kind': 'manual',
                                  'recipe_id': id,
                                  'revision': r['revision'],
                                };
                                servings.text = '${r['servings']}';
                              });
                            },
                    ),
                    if (source != null) Text('v${source!['revision']}'),
                    TextFormField(
                      controller: servings,
                      keyboardType: TextInputType.number,
                      decoration: InputDecoration(
                        labelText: context.t('servings'),
                      ),
                      validator: (v) {
                        final n = int.tryParse(v ?? '');
                        return n == null || n < 1 || n > 1000000
                            ? context.t('required')
                            : null;
                      },
                    ),
                  ],
                  DayButton(
                    'historyDate',
                    date,
                    (v) => setState(() {
                      date = v;
                      dirty = true;
                    }),
                  ),
                  TextFormField(
                    controller: zone,
                    decoration: InputDecoration(
                      labelText: context.t('historyZone'),
                    ),
                    validator: (v) => v == null || v.trim().isEmpty
                        ? context.t('required')
                        : null,
                    maxLength: 80,
                  ),
                  SectionTitle(context.t('historyPeople')),
                  if (objects(profiles!['members']).isEmpty)
                    Note(context.t('historyPeopleRequired'), warning: true),
                  if (attendees.any(
                    (id) =>
                        !objects(profiles!['members'])
                            .any((p) => p['id'] == id),
                  ))
                    Note(context.t('historyRemovedHelp'), warning: true),
                  for (final id
                      in attendees
                          .where(
                            (id) =>
                                !objects(profiles!['members'])
                                    .any((p) => p['id'] == id),
                          )
                          .toList())
                    CheckboxListTile(
                      value: true,
                      title: Text('${context.t('historyRemovedPerson')} ($id)'),
                      onChanged: busy
                          ? null
                          : (_) => setState(() {
                              attendees.remove(id);
                              ratings.remove(id);
                              dirty = true;
                            }),
                    ),
                  for (final p in objects(profiles!['members'])) ...[
                    CheckboxListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(p['name'] as String),
                      value: attendees.contains(p['id']),
                      onChanged: busy || uncertain
                          ? null
                          : (v) => setState(() {
                              dirty = true;
                              if (v == true) {
                                attendees.add(p['id'] as String);
                              } else {
                                attendees.remove(p['id']);
                                ratings.remove(p['id']);
                              }
                            }),
                    ),
                    if (attendees.contains(p['id']))
                      RatingField(
                        'historyPersonalRating',
                        ratings[p['id']] as int?,
                        (v) => setState(() {
                          dirty = true;
                          if (v == null) {
                            ratings.remove(p['id']);
                          } else {
                            ratings[p['id'] as String] = v;
                          }
                        }),
                      ),
                  ],
                  Note(context.t('historyRatingHelp')),
                  RatingField(
                    'historyHouseholdRating',
                    householdRating,
                    (v) => setState(() {
                      householdRating = v;
                      dirty = true;
                    }),
                  ),
                  CheckboxListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('historyEffort')),
                    subtitle: Text(context.t('historyEffortHelp')),
                    value: knownEffort,
                    onChanged: busy || uncertain
                        ? null
                        : (v) => setState(() {
                            knownEffort = v == true;
                            effort ??= 0;
                            dirty = true;
                          }),
                  ),
                  if (knownEffort)
                    DurationField(
                      'historyEffort',
                      effort ?? 0,
                      (v) => effort = v,
                    ),
                  CheckboxListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('historyConfirm')),
                    value: confirmed,
                    onChanged: busy || uncertain
                        ? null
                        : (v) => setState(() => confirmed = v == true),
                  ),
                  FilledButton(
                    onPressed:
                        busy ||
                            uncertain ||
                            !confirmed ||
                            attendees.isEmpty ||
                            source == null ||
                            !widget.api.writable ||
                            !widget.api.controller.online
                        ? null
                        : save,
                    child: Text(context.t('save')),
                  ),
                  if (widget.entry != null)
                    TextButton(
                      onPressed: busy || uncertain
                          ? null
                          : () async {
                              if (!await confirm(
                                context,
                                'history_rebase',
                                body: 'history_rebase_help',
                              )) {
                                return;
                              }
                              setState(() {
                                loading = true;
                                error = null;
                              });
                              final current = await widget.api.household(
                                '/history/${widget.entry!['id']}',
                              );
                              if (!context.mounted) return;
                              await Navigator.pushReplacement<void, void>(
                                context,
                                MaterialPageRoute(
                                  builder: (_) => HistoryEditor(
                                    widget.api,
                                    entry: object(current['entry']),
                                  ),
                                ),
                              );
                            },
                      child: Text(context.t('historyReload')),
                    ),
                ],
              ),
            ),
    ),
  );
}

class RatingField extends StatelessWidget {
  const RatingField(this.label, this.value, this.changed, {super.key});
  final String label;
  final int? value;
  final ValueChanged<int?> changed;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: DropdownButtonFormField<int>(
      key: ValueKey('$label-$value'),
      initialValue: value ?? 0,
      isExpanded: true,
      decoration: InputDecoration(labelText: context.t(label)),
      items: [
        DropdownMenuItem(value: 0, child: Text(context.t('historyNoRating'))),
        for (var i = 1; i <= 5; i++)
          DropdownMenuItem(value: i, child: Text('$i / 5')),
      ],
      onChanged: (v) => changed(v == 0 ? null : v),
    ),
  );
}
