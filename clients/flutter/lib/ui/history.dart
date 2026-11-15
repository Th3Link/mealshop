import 'package:material_ui/material_ui.dart';

import '../content/api.dart';
import '../connection/nextcloud.dart';
import 'content_common.dart';
import 'common.dart';
import 'messages.dart';
import 'recipes.dart';
import 'history_editor.dart';
import 'learning.dart';

class HistoryScreen extends StatefulWidget {
  const HistoryScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<HistoryScreen> createState() => _HistoryScreenState();
}

class _HistoryScreenState extends State<HistoryScreen> {
  final search = TextEditingController();
  String query = '';
  int page = 1, version = 0;
  @override
  void dispose() {
    search.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(context.t('history')),
      actions: [
        IconButton(
          tooltip: context.t('historyLearning'),
          icon: const Icon(Icons.tune),
          onPressed: () => openPage<void>(context, LearningScreen(widget.api)),
        ),
      ],
    ),
    floatingActionButton: widget.api.writable
        ? FloatingActionButton.extended(
            icon: const Icon(Icons.add),
            label: Text(context.t('historyNew')),
            onPressed: () async {
              await openPage<void>(context, HistoryEditor(widget.api));
              if (mounted) setState(() => version++);
            },
          )
        : null,
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            controller: search,
            maxLength: 100,
            decoration: InputDecoration(
              labelText: context.t('historySearch'),
              suffixIcon: IconButton(
                tooltip: context.t('historySearch'),
                icon: const Icon(Icons.search),
                onPressed: () {
                  FocusScope.of(context).unfocus();
                  setState(() {
                    query = search.text.trim();
                    page = 1;
                    version++;
                  });
                },
              ),
            ),
            onSubmitted: (v) => setState(() {
              query = v.trim();
              page = 1;
              version++;
            }),
          ),
        ),
        Expanded(
          child: Resource<Json>(
            key: ValueKey('$page-$version'),
            sensitive: true,
            pollInterval: null,
            load: () => widget.api.household(
              '/history?page=$page&q=${Uri.encodeComponent(query)}',
            ),
            builder: (context, data, reload) => ListView(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 110),
              children: [
                Note(context.t('historyVisibility')),
                if (!widget.api.writable)
                  Note(context.t('household_read_only')),
                if (objects(data['entries']).isEmpty)
                  Note(
                    context.t(
                      query.isEmpty ? 'historyEmptyHelp' : 'no_results',
                    ),
                  ),
                for (final entry in objects(data['entries']))
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: const Icon(Icons.history),
                    title: Text(object(entry['recipe'])['title'] as String),
                    subtitle: Text(
                      '${object(entry['input'])['date']} · ${entry['servings']} ${context.t('servings')} · ${object(entry['input'])['household_rating'] ?? context.t('historyNoRating')}',
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () async {
                      await openPage<void>(
                        context,
                        HistoryDetail(widget.api, entry['id'] as String),
                      );
                      await reload();
                    },
                  ),
                Wrap(
                  alignment: WrapAlignment.spaceBetween,
                  children: [
                    if (page > 1)
                      TextButton(
                        onPressed: () => setState(() => page--),
                        child: Text(context.t('previous_page')),
                      ),
                    Text('$page · ${data['total']}'),
                    if (page * 20 < (data['total'] as int))
                      TextButton(
                        onPressed: () => setState(() => page++),
                        child: Text(context.t('next_page')),
                      ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ],
    ),
  );
}

class HistoryDetail extends StatelessWidget {
  const HistoryDetail(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('history'))),
    body: Resource<Json>(
      sensitive: true,
      pollInterval: null,
      load: () async {
        final values = await Future.wait([
          api.household('/history/$id'),
          api.household('/profiles'),
        ]);
        return {...values[0], 'profiles': values[1]};
      },
      builder: (context, data, reload) {
        final e = object(data['entry']), input = object(e['input']);
        final people = objects(object(data['profiles'])['members']);
        String person(String id) =>
            people.where((p) => p['id'] == id).firstOrNull?['name']
                as String? ??
            '${context.t('historyRemovedPerson')} ($id)';
        return ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(
              object(e['recipe'])['title'] as String,
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            Text(
              'v${object(e['recipe'])['revision']} · ${e['servings']} ${context.t('servings')}',
            ),
            Text(
              context.t(
                object(e['source'])['kind'] == 'session'
                    ? 'historySession'
                    : 'historyManual',
              ),
            ),
            Text('${input['date']} · ${input['timezone']}'),
            Note(context.t('historyVisibility')),
            Text(
              '${context.t('historyPeople')}: ${(input['attendees'] as List).cast<String>().map(person).join(', ')}',
            ),
            Text(
              '${context.t('historyEffort')}: ${input['active_seconds'] == null ? context.t('historyEffortUnknown') : '${input['active_seconds']} ${context.t('cookSeconds')}'}',
            ),
            Text(
              '${context.t('historyHouseholdRating')}: ${input['household_rating'] ?? context.t('historyNoRating')}',
            ),
            for (final rating in object(input['ratings']).entries)
              Text('${person(rating.key)}: ${rating.value} / 5'),
            Text(
              '${context.t('historyRecordedBy')}: ${e['recorded_by']} · ${DateTime.fromMillisecondsSinceEpoch((e['recorded_at'] as int) * 1000, isUtc: true).toIso8601String()}',
            ),
            Text(
              '${context.t('historyUpdatedBy')}: ${e['updated_by']} · ${DateTime.fromMillisecondsSinceEpoch((e['updated_at'] as int) * 1000, isUtc: true).toIso8601String()}',
            ),
            TextButton(
              onPressed: () => openPage<void>(
                context,
                RecipeDetail(
                  api,
                  object(e['recipe'])['id'] as String,
                  revision: object(e['recipe'])['revision'] as int,
                ),
              ),
              child: Text(context.t('historySourceRecipe')),
            ),
            if (api.writable) ...[
              FilledButton(
                onPressed: () async {
                  await openPage<void>(context, HistoryEditor(api, entry: e));
                  await reload();
                },
                child: Text(context.t('historyEdit')),
              ),
              TextButton(
                onPressed: () async {
                  if (!await confirm(
                        context,
                        'historyDelete',
                        body: 'historyDeleteConfirm',
                      ) ||
                      !context.mounted) {
                    return;
                  }
                  await runAction(context, () async {
                    await api.household(
                      '/history/$id',
                      method: 'DELETE',
                      body: {
                        'expected_revision': data['revision'],
                        'confirmed': true,
                      },
                    );
                    if (context.mounted) Navigator.pop(context);
                  });
                },
                child: Text(context.t('historyDelete')),
              ),
            ],
          ],
        );
      },
    ),
  );
}
