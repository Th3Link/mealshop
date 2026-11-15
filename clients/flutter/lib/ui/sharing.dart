import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'ingredients.dart';
import 'messages.dart';
import 'recipes.dart';

class AccountPicker extends StatefulWidget {
  const AccountPicker(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<AccountPicker> createState() => _AccountPickerState();
}

class _AccountPickerState extends State<AccountPicker> {
  Timer? timer;
  int epoch = 0;
  List<Json> people = [];
  String? error;
  bool busy = false;
  @override
  void initState() {
    super.initState();
    search('');
  }

  @override
  void dispose() {
    timer?.cancel();
    epoch++;
    super.dispose();
  }

  void search(String value) {
    timer?.cancel();
    final id = ++epoch;
    setState(() => busy = true);
    timer = Timer(const Duration(milliseconds: 300), () async {
      try {
        final uri = widget.api.credential.server
            .path('/ocs/v2.php/apps/files_sharing/api/v1/sharees')
            .replace(
              queryParameters: {
                'search': value,
                'itemType': 'file',
                'shareType[0]': '0',
                'perPage': '20',
                'lookup': 'false',
                'format': 'json',
              },
            );
        final envelope = object(
          await widget.api.controller.api.request(
            uri,
            credential: widget.api.credential,
          ),
        );
        final data = object(object(envelope['ocs'])['data']);
        final items = [
          ...objects(object(data['exact'])['users']),
          ...objects(data['users']),
        ];
        if (mounted && id == epoch && widget.api.current) {
          setState(() {
            people = {
              for (final p in items)
                object(p['value'])['shareWith'] as String: p,
            }.values.toList();
            error = null;
            busy = false;
          });
        }
      } catch (e) {
        if (mounted && id == epoch) {
          setState(() {
            people = [];
            error = e is ApiFailure ? e.code : 'request_failed';
            busy = false;
          });
        }
      }
    });
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('recipient'))),
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            autofocus: true,
            onChanged: search,
            decoration: InputDecoration(
              labelText: context.t('recipient_user'),
              prefixIcon: const Icon(Icons.search),
            ),
          ),
        ),
        if (busy) const LinearProgressIndicator(),
        if (error != null)
          Padding(
            padding: const EdgeInsets.all(16),
            child: Text(context.t(error!)),
          ),
        Expanded(
          child: ListView(
            children: [
              for (final person in people)
                ListTile(
                  leading: const Icon(Icons.person_outline),
                  title: Text(person['label'] as String),
                  subtitle: Text(
                    object(person['value'])['shareWith'] as String,
                  ),
                  onTap: () => Navigator.pop(
                    context,
                    object(person['value'])['shareWith'] as String,
                  ),
                ),
            ],
          ),
        ),
      ],
    ),
  );
}

class ShareEditor extends StatefulWidget {
  const ShareEditor(this.api, {super.key, required this.resource});
  final HouseholdApi api;
  final Json resource;
  @override
  State<ShareEditor> createState() => _ShareEditorState();
}

class _ShareEditorState extends State<ShareEditor> {
  Json? snapshot;
  String kind = 'account', user = '', home = '';
  bool busy = false;
  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  Future<void> load() async {
    try {
      final v = await widget.api.call('${widget.api.root}/shares?per_page=1');
      if (mounted) setState(() => snapshot = v);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('share'))),
    body: snapshot == null
        ? const Center(child: CircularProgressIndicator())
        : ListView(
            padding: const EdgeInsets.all(20),
            children: [
              Note(context.t('source_only')),
              Choice('recipient', kind, {
                for (final k in [
                  'account',
                  'household_invitation',
                  'household',
                ])
                  k: context.t('${k}_recipient'),
              }, (v) => setState(() => kind = v)),
              if (kind == 'household')
                Choice('household', home, {
                  for (final h in widget.api.controller.homes.where(
                    (h) => h.role == 'owner' && h.id != widget.api.home,
                  ))
                    h.id: h.name,
                }, (v) => setState(() => home = v))
              else
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  leading: const Icon(Icons.person_add_outlined),
                  title: Text(
                    user.isEmpty ? context.t('recipient_user') : user,
                  ),
                  trailing: const Icon(Icons.search),
                  onTap: () async {
                    final selected = await openPage<String>(
                      context,
                      AccountPicker(widget.api),
                    );
                    if (selected != null) setState(() => user = selected);
                  },
                ),
              const SizedBox(height: 20),
              FilledButton(
                onPressed:
                    busy || (kind == 'household' ? home.isEmpty : user.isEmpty)
                    ? null
                    : () async {
                        setState(() => busy = true);
                        try {
                          await widget.api.call(
                            '${widget.api.root}/shares',
                            method: 'POST',
                            body: {
                              'expected_revision': snapshot!['revision'],
                              'resource': widget.resource,
                              'recipient': kind == 'household'
                                  ? {'kind': 'household', 'home_id': home}
                                  : {'kind': kind, 'user': user},
                            },
                          );
                          if (context.mounted) Navigator.pop(context);
                        } catch (e) {
                          if (context.mounted) await showFailure(context, e);
                        } finally {
                          if (mounted) setState(() => busy = false);
                        }
                      },
                child: Text(context.t('share')),
              ),
            ],
          ),
  );
}

class SharesScreen extends StatefulWidget {
  const SharesScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<SharesScreen> createState() => _SharesScreenState();
}

class _SharesScreenState extends State<SharesScreen> {
  int page = 1;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('shares'))),
    body: Resource<Json>(
      key: ValueKey(page),
      load: () =>
          widget.api.call('${widget.api.root}/shares?page=$page&per_page=50'),
      builder: (context, data, reload) => ListView(
        padding: const EdgeInsets.all(20),
        children: [
          for (final item in objects(data['shares']))
            Builder(
              builder: (context) {
                final grant = object(item['grant']);
                final resource = object(grant['resource']);
                return ListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(item['recipient_label'] as String),
                  subtitle: Text(
                    '${context.t(resource['kind'] as String)} · ${resource['recipe_id'] ?? resource['collection_id']}${resource['revision'] == null ? '' : ' · ${context.t('revision')} ${resource['revision']}'}',
                  ),
                  trailing: PopupMenuButton<String>(
                    onSelected: (action) => runAction(context, () async {
                      if (action == 'revoke') {
                        if (!await confirm(context, 'revoke')) return;
                        await widget.api.call(
                          '${widget.api.root}/shares/${grant['id']}',
                          method: 'DELETE',
                          body: {'expected_revision': data['revision']},
                        );
                      } else {
                        final latest = await widget.api.recipes(
                          '/${resource['recipe_id']}',
                        );
                        if (!context.mounted) return;
                        final chosen = await askName(
                          context,
                          'publish',
                          initial: '${latest['revision']}',
                          field: 'revision',
                          integer: true,
                        );
                        if (chosen == null) return;
                        await widget.api.call(
                          '${widget.api.root}/shares/${grant['id']}',
                          method: 'PUT',
                          body: {
                            'expected_revision': data['revision'],
                            'resource': {
                              ...resource,
                              'revision': int.parse(chosen),
                            },
                          },
                        );
                      }
                      await reload();
                    }),
                    itemBuilder: (context) => [
                      if (resource['kind'] == 'recipe')
                        PopupMenuItem(
                          value: 'publish',
                          child: Text(context.t('publish')),
                        ),
                      PopupMenuItem(
                        value: 'revoke',
                        child: Text(context.t('revoke')),
                      ),
                    ],
                  ),
                );
              },
            ),
          if ((data['total'] as int) == 0) Text(context.t('nothing_yet')),
          PageControls(
            page,
            data['total'] as int,
            (v) => setState(() => page = v),
          ),
        ],
      ),
    ),
  );
}

class SharedScreen extends StatefulWidget {
  const SharedScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<SharedScreen> createState() => _SharedScreenState();
}

class _SharedScreenState extends State<SharedScreen> {
  int page = 1;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('shared'))),
    body: Resource<Json>(
      key: ValueKey(page),
      sensitive: true,
      load: () => widget.api.call('/api/shared?page=$page&per_page=50'),
      builder: (context, data, reload) => ListView(
        padding: const EdgeInsets.all(20),
        children: [
          if (objects(data['items']).isEmpty) Text(context.t('nothing_yet')),
          for (final item in objects(data['items']))
            Builder(
              builder: (context) {
                final share = object(item['share']),
                    content = object(item['content'] ?? <String, dynamic>{});
                final path =
                    '/api/shared/${share['source_home']}/${share['id']}';
                return ListTile(
                  contentPadding: EdgeInsets.zero,
                  leading: Icon(
                    content['kind'] == 'collection'
                        ? Icons.collections_bookmark_outlined
                        : Icons.menu_book_outlined,
                  ),
                  title: Text(
                    content['title'] as String? ??
                        content['name'] as String? ??
                        context.t(
                          share['pending'] == true
                              ? 'pending_household_share'
                              : 'source_only',
                        ),
                  ),
                  subtitle: Text(share['owner'] as String),
                  trailing: PopupMenuButton<String>(
                    onSelected: (action) => runAction(context, () async {
                      if (action == 'accept') {
                        final own = widget.api.controller.homes
                            .where((h) => h.role == 'owner')
                            .toList();
                        if (own.isEmpty) {
                          throw const ApiFailure('create_home_first');
                        }
                        final selected = await showModalBottomSheet<String>(
                          context: context,
                          builder: (context) => SafeArea(
                            child: ListView(
                              shrinkWrap: true,
                              children: [
                                for (final home in own)
                                  ListTile(
                                    title: Text(home.name),
                                    onTap: () =>
                                        Navigator.pop(context, home.id),
                                  ),
                              ],
                            ),
                          ),
                        );
                        if (selected == null) return;
                        await widget.api.call(
                          '$path/accept',
                          method: 'PUT',
                          body: {
                            'expected_version': share['version'],
                            'household_id': selected,
                          },
                        );
                      } else {
                        if (!await confirm(context, 'confirm_delete')) return;
                        await widget.api.call(
                          path,
                          method: 'DELETE',
                          body: {'expected_version': share['version']},
                        );
                      }
                      await reload();
                    }),
                    itemBuilder: (context) => [
                      if (share['pending'] == true)
                        PopupMenuItem(
                          value: 'accept',
                          child: Text(context.t('accept')),
                        ),
                      PopupMenuItem(
                        value: 'decline',
                        child: Text(context.t('decline')),
                      ),
                    ],
                  ),
                  onTap: share['pending'] == true
                      ? null
                      : () async {
                          if (content['kind'] == 'source_link') {
                            await runAction(
                              context,
                              () => openSource(content['source_url'] as String),
                            );
                          } else if (content['kind'] == 'collection') {
                            await openPage<void>(
                              context,
                              SharedCollection(widget.api, path),
                            );
                          } else {
                            await openPage<void>(
                              context,
                              RecipeDetail(widget.api, '', sharedPath: path),
                            );
                          }
                          await reload();
                        },
                );
              },
            ),
          PageControls(
            page,
            data['total'] as int,
            (v) => setState(() => page = v),
          ),
        ],
      ),
    ),
  );
}

class SharedCollection extends StatefulWidget {
  const SharedCollection(this.api, this.path, {super.key});
  final HouseholdApi api;
  final String path;
  @override
  State<SharedCollection> createState() => _SharedCollectionState();
}

class _SharedCollectionState extends State<SharedCollection> {
  int page = 1;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('collections'))),
    body: Resource<Json>(
      key: ValueKey(page),
      sensitive: true,
      load: () => widget.api.call('${widget.path}?page=$page&per_page=50'),
      builder: (context, data, reload) {
        final content = object(data['content']);
        return ListView(
          padding: const EdgeInsets.all(20),
          children: [
            Text(
              content['name'] as String,
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            Text(content['description'] as String),
            for (final item in objects(content['items']))
              ListTile(
                title: Text(
                  object(item['content'])['title'] as String? ??
                      context.t('source_only'),
                ),
                subtitle: Text('${context.t('revision')} ${item['revision']}'),
                onTap: () => openPage<void>(
                  context,
                  RecipeDetail(
                    widget.api,
                    item['recipe_id'] as String,
                    sharedPath: '${widget.path}/recipes/${item['recipe_id']}',
                  ),
                ),
              ),
            PageControls(
              page,
              content['count'] as int,
              (v) => setState(() => page = v),
            ),
          ],
        );
      },
    ),
  );
}

class PageControls extends StatelessWidget {
  const PageControls(this.page, this.total, this.changed, {super.key});
  final int page, total;
  final ValueChanged<int> changed;
  @override
  Widget build(BuildContext context) => total <= 50
      ? const SizedBox.shrink()
      : Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            IconButton(
              tooltip: context.t('back'),
              onPressed: page > 1 ? () => changed(page - 1) : null,
              icon: const Icon(Icons.chevron_left),
            ),
            Text('$page / ${(total + 49) ~/ 50}'),
            IconButton(
              tooltip: context.t('next'),
              onPressed: page * 50 < total ? () => changed(page + 1) : null,
              icon: const Icon(Icons.chevron_right),
            ),
          ],
        );
}
