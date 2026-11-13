import 'dart:async';
import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';

Json activityGuard(Json data) => {
  'expected_generation': data['generation'],
  'expected_access_revision': data['access_revision'],
  'expected_revision': data['revision'],
};

class ActivityScreen extends StatefulWidget {
  const ActivityScreen(
    this.api,
    this.person,
    this.profileRevision, {
    super.key,
  });
  final HouseholdApi api;
  final Json person;
  final int profileRevision;
  @override
  State<ActivityScreen> createState() => _ActivityScreenState();
}

class _ActivityScreenState extends State<ActivityScreen> {
  int version = 0, page = 1;
  String get path => '/people/${widget.person['id']}/activity';
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(widget.person['name'] as String),
      actions: [
        if (widget.api.owner)
          IconButton(
            tooltip: context.t('activity_access'),
            icon: const Icon(Icons.manage_accounts_outlined),
            onPressed: () async {
              await openPage<void>(
                context,
                ActivityAccess(widget.api, widget.person),
              );
              if (mounted) setState(() => version++);
            },
          ),
      ],
    ),
    body: Resource<Json>(
      key: ValueKey((version, page)),
      sensitive: true,
      load: () => widget.api.household('$path?page=$page&per_page=50'),
      builder: (context, data, reload) => ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Note(context.t('activity_private')),
          Wrap(
            spacing: 8,
            children: [
              if (data['can_edit'] == true)
                FilledButton.icon(
                  onPressed: () async {
                    await openPage<void>(
                      context,
                      ActivityEditor(widget.api, path, data),
                    );
                    await reload();
                  },
                  icon: const Icon(Icons.add),
                  label: Text(context.t('new_activity')),
                ),
              OutlinedButton.icon(
                onPressed: () => runAction(context, () async {
                  final value = await widget.api.household('$path/export');
                  if (!widget.api.current) return;
                  await const MethodChannel('org.mealshop/documents')
                      .invokeMethod<bool>('export', {
                        'name': 'mealshop-activities.json',
                        'value': jsonEncode(value),
                      });
                }),
                icon: const Icon(Icons.file_download_outlined),
                label: Text(context.t('export')),
              ),
            ],
          ),
          if (objects(data['entries']).isEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 40),
              child: Text(
                context.t('nothing_yet'),
                textAlign: TextAlign.center,
              ),
            ),
          for (final record in objects(data['entries']))
            Builder(
              builder: (context) {
                final entry = object(record['entry']);
                final seconds = entry['duration_seconds'] as int;
                return ListTile(
                  contentPadding: EdgeInsets.zero,
                  leading: Icon(switch (entry['kind']) {
                    'cycling' => Icons.directions_bike,
                    'swimming' => Icons.pool,
                    'strength' => Icons.fitness_center,
                    'walking' => Icons.directions_walk,
                    _ => Icons.directions_run,
                  }),
                  title: Text(context.t(entry['kind'] as String)),
                  subtitle: Text(
                    '${entry['date']} · ${seconds ~/ 60} min${seconds % 60 == 0 ? '' : ' ${seconds % 60} s'}${entry['intensity'] == null ? '' : ' · ${context.t(entry['intensity'] as String)}'}',
                  ),
                  trailing: data['can_edit'] == true
                      ? const Icon(Icons.chevron_right)
                      : null,
                  onTap: data['can_edit'] == true
                      ? () async {
                          await openPage<void>(
                            context,
                            ActivityEditor(
                              widget.api,
                              path,
                              data,
                              record: record,
                            ),
                          );
                          await reload();
                        }
                      : () => openPage<void>(
                          context,
                          ActivityRead(widget.api, '$path/${record['id']}'),
                        ),
                );
              },
            ),
          if ((data['total'] as int) > 50)
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                IconButton(
                  tooltip: context.t('back'),
                  onPressed: page > 1 ? () => setState(() => page--) : null,
                  icon: const Icon(Icons.chevron_left),
                ),
                Text('$page / ${((data['total'] as int) + 49) ~/ 50}'),
                IconButton(
                  tooltip: context.t('next'),
                  onPressed: page * 50 < (data['total'] as int)
                      ? () => setState(() => page++)
                      : null,
                  icon: const Icon(Icons.chevron_right),
                ),
              ],
            ),
          if (data['can_edit'] == true && (data['total'] as int) > 0)
            TextButton.icon(
              onPressed: () => runAction(context, () async {
                if (!await confirm(context, 'delete_all')) return;
                await widget.api.household(
                  path,
                  method: 'DELETE',
                  body: activityGuard(data),
                );
                await reload();
              }),
              icon: const Icon(Icons.delete_sweep_outlined),
              label: Text(context.t('delete_all')),
            ),
        ],
      ),
    ),
  );
}

class ActivityAccess extends StatefulWidget {
  const ActivityAccess(this.api, this.person, {super.key});
  final HouseholdApi api;
  final Json person;
  @override
  State<ActivityAccess> createState() => _ActivityAccessState();
}

class _ActivityAccessState extends State<ActivityAccess> {
  String? failure;
  Json? data;
  Json grants = {};
  int profileRevision = 0;
  bool busy = false, confirmed = false;
  String get path => '/people/${widget.person['id']}/activity/access';
  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  Future<void> load() async {
    try {
      final profiles = await widget.api.household('/profiles');
      final value = await widget.api.household(path);
      if (!mounted) return;
      setState(() {
        profileRevision = profiles['revision'] as int;
        data = value;
        grants = {
          for (final g in objects(value['grants']))
            if (g['active'] == true) g['user'] as String: g['permission'],
        };
      });
    } catch (e) {
      if (mounted) {
        setState(() => failure = e is ApiFailure ? e.code : 'request_failed');
      }
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('activity_access'))),
    body: data == null
        ? failure == null
              ? const Center(child: CircularProgressIndicator())
              : EmptyContent(
                  icon: Icons.cloud_off_outlined,
                  title: failure!,
                  action: 'retry',
                  onAction: load,
                )
        : ListView(
            padding: const EdgeInsets.all(20),
            children: [
              Text(
                widget.person['name'] as String,
                style: Theme.of(context).textTheme.titleLarge,
              ),
              Note(context.t('activity_access_note')),
              for (final user in object(data!['accounts']).keys)
                Choice(
                  user,
                  grants[user] as String? ?? 'none',
                  {
                    'none': context.t('no_access'),
                    'read': context.t('read'),
                    'edit': context.t('edit_access'),
                  },
                  (v) => setState(() {
                    if (v == 'none') {
                      grants.remove(user);
                    } else {
                      grants[user] = v;
                    }
                  }),
                ),
              if (grants.containsKey(widget.api.credential.user) &&
                  data!['can_read'] != true)
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  value: confirmed,
                  onChanged: (v) => setState(() => confirmed = v == true),
                  title: Text(context.t('self_access')),
                ),
              const SizedBox(height: 20),
              FilledButton(
                onPressed: busy
                    ? null
                    : () async {
                        setState(() => busy = true);
                        try {
                          await widget.api.household(
                            path,
                            method: 'PUT',
                            body: {
                              'expected_profile_revision': profileRevision,
                              'expected_generation': data!['generation'],
                              'expected_access_revision':
                                  data!['access_revision'],
                              'grants': grants,
                              'confirm_self_access': confirmed,
                            },
                          );
                          if (context.mounted) Navigator.pop(context);
                        } catch (e) {
                          if (context.mounted) await showFailure(context, e);
                        } finally {
                          if (mounted) setState(() => busy = false);
                        }
                      },
                child: Text(context.t('save')),
              ),
            ],
          ),
  );
}

class ActivityEditor extends StatefulWidget {
  const ActivityEditor(
    this.api,
    this.path,
    this.state, {
    super.key,
    this.record,
  });
  final HouseholdApi api;
  final String path;
  final Json state;
  final Json? record;
  @override
  State<ActivityEditor> createState() => _ActivityEditorState();
}

class _ActivityEditorState extends State<ActivityEditor>
    with WidgetsBindingObserver {
  final form = GlobalKey<FormState>();
  late Json entry;
  late String id, minutes, seconds;
  bool busy = false, dirty = false, visible = true, foreground = true;
  Timer? timer;
  @override
  void initState() {
    super.initState();
    id = widget.record?['id'] as String? ?? newId('activity-');
    entry = clone(
      widget.record == null
          ? {
              'date': DateTime.now().toIso8601String().substring(0, 10),
              'kind': 'walking',
              'duration_seconds': 0,
              'intensity': null,
              'notes': '',
            }
          : object(widget.record!['entry']),
    );
    final duration = entry['duration_seconds'] as int;
    minutes = '${duration ~/ 60}';
    seconds = '${duration % 60}';
    WidgetsBinding.instance.addObserver(this);
    timer = Timer.periodic(const Duration(seconds: 20), (_) {
      if (foreground) unawaited(check());
    });
  }

  void deny() {
    entry.clear();
    setState(() {
      dirty = false;
      visible = false;
    });
    if (Navigator.canPop(context)) {
      Navigator.of(context).removeRoute(ModalRoute.of(context)!);
    }
  }

  Future<void> check() async {
    try {
      final state = await widget.api.household(widget.path);
      if (state['can_edit'] != true ||
          state['generation'] != widget.state['generation']) {
        throw const ApiFailure('activity_access_denied', 403);
      }
      if (mounted) setState(() => visible = true);
    } catch (e) {
      if (!mounted) return;
      if (e is ApiFailure && (e.denied || e.status == 404)) {
        deny();
      } else {
        setState(() => visible = false);
      }
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    foreground = state == AppLifecycleState.resumed;
    if (!foreground) {
      setState(() => visible = false);
    } else {
      unawaited(check());
    }
  }

  @override
  void dispose() {
    timer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  void change(VoidCallback action) => setState(() {
    action();
    dirty = true;
  });
  Future<void> save({bool remove = false}) async {
    if (!remove && !form.currentState!.validate()) return;
    setState(() => busy = true);
    try {
      final min = int.tryParse(minutes), sec = int.tryParse(seconds);
      if (!remove &&
          (min == null ||
              sec == null ||
              min < 0 ||
              sec < 0 ||
              sec > 59 ||
              min * 60 + sec < 1 ||
              min * 60 + sec > 604800)) {
        throw const ApiFailure('invalid_activity');
      }
      if (!remove) entry['duration_seconds'] = min! * 60 + sec!;
      await widget.api.household(
        '${widget.path}/$id',
        method: remove ? 'DELETE' : 'PUT',
        body: {...activityGuard(widget.state), if (!remove) 'entry': entry},
      );
      dirty = false;
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) {
        if (e is ApiFailure && (e.denied || e.status == 404)) {
          deny();
        } else {
          await showFailure(context, e);
        }
      }
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: dirty,
    child: Scaffold(
      appBar: AppBar(
        title: Text(context.t('new_activity')),
        actions: [
          if (widget.record != null)
            IconButton(
              tooltip: context.t('delete'),
              onPressed: busy
                  ? null
                  : () async {
                      if (await confirm(context, 'confirm_delete')) {
                        await save(remove: true);
                      }
                    },
              icon: const Icon(Icons.delete_outline),
            ),
        ],
      ),
      body: !visible
          ? EmptyContent(
              icon: Icons.lock_outline,
              title: 'attention',
              action: 'retry',
              onAction: check,
            )
          : Form(
              key: form,
              child: ListView(
                padding: const EdgeInsets.all(20),
                children: [
                  Note(context.t('activity_private')),
                  Choice('activity_kind', entry['kind'] as String, {
                    for (final kind in [
                      'walking',
                      'running',
                      'cycling',
                      'swimming',
                      'strength',
                      'other',
                    ])
                      kind: context.t(kind),
                  }, (v) => change(() => entry['kind'] = v)),
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('date')),
                    subtitle: Text(entry['date'] as String),
                    trailing: const Icon(Icons.calendar_today_outlined),
                    onTap: () async {
                      final selected = await showDatePicker(
                        context: context,
                        initialDate: DateTime.parse(entry['date'] as String),
                        firstDate: DateTime(1900),
                        lastDate: DateTime(2200),
                      );
                      if (selected != null) {
                        change(
                          () => entry['date'] = selected
                              .toIso8601String()
                              .substring(0, 10),
                        );
                      }
                    },
                  ),
                  EditText(
                    'duration_minutes',
                    minutes,
                    (v) => change(() => minutes = v),
                    numeric: true,
                    integer: true,
                    required: true,
                  ),
                  EditText(
                    'duration_seconds',
                    seconds,
                    (v) => change(() => seconds = v),
                    numeric: true,
                    integer: true,
                    required: true,
                  ),
                  Choice(
                    'intensity',
                    entry['intensity'] as String? ?? 'none',
                    {
                      'none': context.t('unknown_value'),
                      for (final i in ['easy', 'moderate', 'hard'])
                        i: context.t(i),
                    },
                    (v) => change(
                      () => entry['intensity'] = v == 'none' ? null : v,
                    ),
                  ),
                  EditText(
                    'notes',
                    entry['notes'] as String,
                    (v) => change(() => entry['notes'] = v),
                    lines: 3,
                    max: 1000,
                  ),
                  const SizedBox(height: 20),
                  FilledButton(
                    onPressed: busy ? null : save,
                    child: Text(context.t('save')),
                  ),
                ],
              ),
            ),
    ),
  );
}

class ActivityRead extends StatelessWidget {
  const ActivityRead(this.api, this.path, {super.key});
  final HouseholdApi api;
  final String path;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('activities'))),
    body: Resource<Json>(
      sensitive: true,
      load: () => api.household(path),
      builder: (context, data, reload) {
        final entry = object(data['entry']);
        return ListView(
          padding: const EdgeInsets.all(20),
          children: [
            Note(context.t('activity_private')),
            Text(
              context.t(entry['kind'] as String),
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            Text(entry['date'] as String),
            Text(
              '${(entry['duration_seconds'] as int) ~/ 60} min ${(entry['duration_seconds'] as int) % 60} s',
            ),
            if (entry['intensity'] != null)
              Text(context.t(entry['intensity'] as String)),
            if ((entry['notes'] as String).isNotEmpty)
              Text(entry['notes'] as String),
          ],
        );
      },
    ),
  );
}
